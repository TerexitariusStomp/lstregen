import 'reflect-metadata';
import { CosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { createConnection, Connection, Repository } from 'typeorm';
import { StakeEvent, UnbondEvent, RewardEvent } from './entities';

interface IndexerConfig {
  rpcEndpoint: string;
  contractAddress: string;
  startHeight: number;
  dbConnection: string;
}

class LiquidStakingIndexer {
  private client!: CosmWasmClient;
  private connection!: Connection;
  private stakeEventRepo!: Repository<StakeEvent>;
  private unbondEventRepo!: Repository<UnbondEvent>;
  private rewardEventRepo!: Repository<RewardEvent>;
  private lastProcessedHeight = 0;

  constructor(private config: IndexerConfig) {}

  async initialize() {
    // Connect to Cosmos client
    this.client = await CosmWasmClient.connect(this.config.rpcEndpoint);

    // Connect to database
    this.connection = await createConnection({
      type: 'postgres',
      url: this.config.dbConnection,
      entities: [StakeEvent, UnbondEvent, RewardEvent],
      synchronize: true,
      logging: false,
    });

    this.stakeEventRepo = this.connection.getRepository(StakeEvent);
    this.unbondEventRepo = this.connection.getRepository(UnbondEvent);
    this.rewardEventRepo = this.connection.getRepository(RewardEvent);

    this.lastProcessedHeight = this.config.startHeight || (await this.client.getHeight());
  }

  async startIndexing() {
    console.log('Starting liquid staking indexer...');

    // Backfill from startHeight to current
    const latestHeight = await this.client.getHeight();
    let currentHeight = Math.max(this.config.startHeight, 1);

    while (currentHeight <= latestHeight) {
      await this.processBlock(currentHeight);
      this.lastProcessedHeight = currentHeight;
      currentHeight++;

      if (currentHeight % 500 === 0) {
        console.log(`Processed block ${currentHeight}/${latestHeight}`);
      }
    }

    // Start real-time monitoring
    await this.startRealTimeMonitoring();
  }

  private async startRealTimeMonitoring() {
    console.log('Starting real-time monitoring...');
    // Poll new blocks
    setInterval(async () => {
      try {
        const head = await this.client.getHeight();
        if (head > this.lastProcessedHeight) {
          for (let h = this.lastProcessedHeight + 1; h <= head; h++) {
            await this.processBlock(h);
            this.lastProcessedHeight = h;
          }
        }
      } catch (err) {
        console.error('Realtime monitor error:', err);
      }
    }, 30_000);
  }

  private async processBlock(height: number) {
    try {
      const block = await this.client.getBlock(height);
      // The @cosmjs client does not expose txs at block level in all RPCs;
      // you may need to query txs by height via Tendermint RPC. This is a simplified example.
      const search = await this.client.searchTx({ height });
      for (const tx of search) {
        if (tx.code === 0) {
          await this.processTxEvents(tx, height, block.header.time);
        }
      }
    } catch (error) {
      console.error(`Error processing block ${height}:`, error);
    }
  }

  private async processTxEvents(tx: any, height: number, blockTime: string) {
    for (const event of tx.events || []) {
      if (
        event.type === 'wasm' &&
        event.attributes?.find(
          (attr: any) => attr.key === '_contract_address' && attr.value === this.config.contractAddress,
        )
      ) {
        const action = event.attributes.find((attr: any) => attr.key === 'action')?.value;

        switch (action) {
          case 'stake':
            await this.indexStakeEvent(event, tx, height, blockTime);
            break;
          case 'unbond':
            await this.indexUnbondEvent(event, tx, height, blockTime);
            break;
          case 'claim_rewards':
            await this.indexRewardEvent(event, tx, height, blockTime);
            break;
          default:
            break;
        }
      }
    }
  }

  private getAttributeValue(event: any, key: string): string {
    return event.attributes.find((attr: any) => attr.key === key)?.value || '';
  }

  private async indexStakeEvent(event: any, tx: any, height: number, blockTime: string) {
    const staker = this.getAttributeValue(event, 'staker');
    const regenAmount = this.getAttributeValue(event, 'regen_amount');
    const dregenAmount = this.getAttributeValue(event, 'dregen_amount');
    const exchangeRate = this.getAttributeValue(event, 'exchange_rate');

    const stakeEvent = new StakeEvent();
    stakeEvent.txHash = tx.hash || tx.txhash;
    stakeEvent.height = height;
    stakeEvent.timestamp = new Date(blockTime);
    stakeEvent.staker = staker;
    stakeEvent.regenAmount = regenAmount;
    stakeEvent.dregenAmount = dregenAmount;
    stakeEvent.exchangeRate = parseFloat(exchangeRate || '0');

    await this.stakeEventRepo.save(stakeEvent);
  }

  private async indexUnbondEvent(event: any, tx: any, height: number, blockTime: string) {
    const user = this.getAttributeValue(event, 'user');
    const dregenAmount = this.getAttributeValue(event, 'dregen_amount');
    const regenAmount = this.getAttributeValue(event, 'regen_amount');
    const unbondingId = this.getAttributeValue(event, 'unbonding_id');
    const completionTime = this.getAttributeValue(event, 'completion_time');

    const unbondEvent = new UnbondEvent();
    unbondEvent.txHash = tx.hash || tx.txhash;
    unbondEvent.height = height;
    unbondEvent.timestamp = new Date(blockTime);
    unbondEvent.user = user;
    unbondEvent.dregenAmount = dregenAmount;
    unbondEvent.regenAmount = regenAmount;
    unbondEvent.unbondingId = parseInt(unbondingId || '0');
    unbondEvent.completionTime = new Date((parseInt(completionTime || '0')) * 1000);

    await this.unbondEventRepo.save(unbondEvent);
  }

  private async indexRewardEvent(event: any, tx: any, height: number, blockTime: string) {
    const claimer = this.getAttributeValue(event, 'claimer');
    const rewardEvent = new RewardEvent();
    rewardEvent.txHash = tx.hash || tx.txhash;
    rewardEvent.height = height;
    rewardEvent.timestamp = new Date(blockTime);
    rewardEvent.claimer = claimer;

    await this.rewardEventRepo.save(rewardEvent);
  }
}

// Start the indexer
async function main() {
  // Load environment
  // eslint-disable-next-line @typescript-eslint/no-var-requires
  require('dotenv').config();

  const config: IndexerConfig = {
    rpcEndpoint: process.env.RPC_ENDPOINT || 'https://rpc.regen.network',
    contractAddress: process.env.CONTRACT_ADDRESS || '',
    startHeight: parseInt(process.env.START_HEIGHT || '0'),
    dbConnection: process.env.DATABASE_URL || 'postgres://localhost:5432/indexer',
  };

  if (!config.contractAddress) {
    throw new Error('CONTRACT_ADDRESS is required');
  }

  const indexer = new LiquidStakingIndexer(config);
  await indexer.initialize();
  await indexer.startIndexing();
}

if (require.main === module) {
  main().catch((err) => {
    console.error(err);
    process.exit(1);
  });
}