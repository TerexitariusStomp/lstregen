import React, { useEffect, useState } from 'react';
import { CosmWasmClient, SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { GasPrice } from '@cosmjs/stargate';

declare global {
  interface Window {
    keplr?: any;
  }
}

interface StakingInterfaceProps {
  contractAddress: string;
  rpcEndpoint: string;
  chainId?: string; // default regen-1
}

interface StakingState {
  totalStaked: string;
  totalSupply: string;
  exchangeRate: string;
  userBalance: string;
  userStaked: string;
}

const StakingInterface: React.FC<StakingInterfaceProps> = ({
  contractAddress,
  rpcEndpoint,
  chainId = 'regen-1',
}) => {
  const [client, setClient] = useState<CosmWasmClient | null>(null);
  const [signingClient, setSigningClient] = useState<SigningCosmWasmClient | null>(null);
  const [walletAddress, setWalletAddress] = useState<string>('');
  const [stakingState, setStakingState] = useState<StakingState>({
    totalStaked: '0',
    totalSupply: '0',
    exchangeRate: '1.0',
    userBalance: '0',
    userStaked: '0',
  });
  const [stakeAmount, setStakeAmount] = useState<string>('');
  const [unbondAmount, setUnbondAmount] = useState<string>('');
  const [loading, setLoading] = useState<boolean>(false);

  useEffect(() => {
    initializeClients();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (client) {
      fetchStakingState();
      const interval = setInterval(fetchStakingState, 30000); // Update every 30 seconds
      return () => clearInterval(interval);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [client, walletAddress]);

  const initializeClients = async () => {
    try {
      const queryClient = await CosmWasmClient.connect(rpcEndpoint);
      setClient(queryClient);

      if (window.keplr) {
        await window.keplr.enable(chainId);
        const offlineSigner = window.keplr.getOfflineSigner(chainId);
        const signingClient = await SigningCosmWasmClient.connectWithSigner(rpcEndpoint, offlineSigner, {
          gasPrice: GasPrice.fromString('0.025uregen'),
        });
        setSigningClient(signingClient);

        const accounts = await offlineSigner.getAccounts();
        setWalletAddress(accounts[0].address);
      }
    } catch (error) {
      console.error('Failed to initialize clients:', error);
    }
  };

  const fetchStakingState = async () => {
    if (!client) return;

    try {
      const stateResponse = await client.queryContractSmart(contractAddress, { state: {} });
      const exchangeRateResponse = await client.queryContractSmart(contractAddress, { exchange_rate: {} });

      let userBalance = '0';
      if (walletAddress) {
        const balanceResponse = await client.getBalance(walletAddress, 'uregen');
        userBalance = balanceResponse.amount;
      }

      setStakingState({
        totalStaked: stateResponse.total_regen_staked,
        totalSupply: stateResponse.total_dregen_supply,
        exchangeRate: exchangeRateResponse.rate,
        userBalance,
        userStaked: '0', // Optionally: query CW20 dREGEN balance if available
      });
    } catch (error) {
      console.error('Failed to fetch staking state:', error);
    }
  };

  const handleStake = async () => {
    if (!signingClient || !walletAddress || !stakeAmount) return;

    setLoading(true);
    try {
      const amountUregen = Math.round(parseFloat(stakeAmount) * 1_000_000);
      const funds = [{ denom: 'uregen', amount: amountUregen.toString() }];

      const result = await signingClient.execute(
        walletAddress,
        contractAddress,
        { stake: {} },
        'auto',
        undefined,
        funds
      );

      console.log('Staking successful:', result);
      setStakeAmount('');
      await fetchStakingState();
    } catch (error) {
      console.error('Staking failed:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleUnbond = async () => {
    if (!signingClient || !walletAddress || !unbondAmount) return;

    setLoading(true);
    try {
      const amount = Math.round(parseFloat(unbondAmount) * 1_000_000);

      const result = await signingClient.execute(
        walletAddress,
        contractAddress,
        { unbond: { dregen_amount: amount.toString() } },
        'auto'
      );

      console.log('Unbonding successful:', result);
      setUnbondAmount('');
      await fetchStakingState();
    } catch (error) {
      console.error('Unbonding failed:', error);
    } finally {
      setLoading(false);
    }
  };

  const formatAmount = (amount: string, decimals: number = 6) => {
    const num = parseFloat(amount) / Math.pow(10, decimals);
    if (Number.isNaN(num)) return '0';
    return num.toLocaleString(undefined, { maximumFractionDigits: 2 });
  };

  const expectedDregen = () => {
    const rate = parseFloat(stakingState.exchangeRate || '1');
    const stake = parseFloat(stakeAmount || '0') * 1_000_000;
    if (rate <= 0 || !isFinite(rate)) return '0';
    return formatAmount((stake / rate).toString());
  };

  const expectedRegenFromUnbond = () => {
    const rate = parseFloat(stakingState.exchangeRate || '1');
    const dreg = parseFloat(unbondAmount || '0') * 1_000_000;
    return formatAmount((dreg * rate).toString());
  };

  return (
    <div className="max-w-4xl mx-auto p-6 bg-white rounded-lg shadow-lg">
      <h1 className="text-3xl font-bold text-gray-800 mb-8">Regen Liquid Staking</h1>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
        <div className="bg-green-50 p-4 rounded-lg">
          <h3 className="text-sm font-medium text-gray-500">Total Staked</h3>
          <p className="text-2xl font-bold text-green-600">
            {formatAmount(stakingState.totalStaked)} REGEN
          </p>
        </div>
        <div className="bg-blue-50 p-4 rounded-lg">
          <h3 className="text-sm font-medium text-gray-500">Exchange Rate</h3>
          <p className="text-2xl font-bold text-blue-600">
            {parseFloat(stakingState.exchangeRate).toFixed(4)}
          </p>
        </div>
        <div className="bg-purple-50 p-4 rounded-lg">
          <h3 className="text-sm font-medium text-gray-500">APR</h3>
          <p className="text-2xl font-bold text-purple-600">—</p>
        </div>
      </div>

      {!walletAddress ? (
        <div className="text-center mb-8">
          <button
            onClick={initializeClients}
            className="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-lg"
          >
            Connect Keplr Wallet
          </button>
        </div>
      ) : (
        <div className="mb-8">
          <p className="text-sm text-gray-600">
            Connected: {walletAddress.slice(0, 10)}...{walletAddress.slice(-6)}
          </p>
          <p className="text-sm text-gray-600">
            Balance: {formatAmount(stakingState.userBalance)} REGEN
          </p>
        </div>
      )}

      {walletAddress && (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
          <div className="bg-green-50 p-6 rounded-lg">
            <h2 className="text-xl font-bold text-green-800 mb-4">Stake REGEN</h2>
            <div className="mb-4">
              <label className="block text-sm font-medium text-gray-700 mb-2">Amount (REGEN)</label>
              <input
                type="number"
                value={stakeAmount}
                onChange={(e) => setStakeAmount(e.target.value)}
                className="w-full p-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500"
                placeholder="Enter amount to stake"
              />
            </div>
            {stakeAmount && (
              <p className="text-sm text-gray-600 mb-4">You will receive ~{expectedDregen()} dREGEN</p>
            )}
            <button
              onClick={handleStake}
              disabled={loading || !stakeAmount}
              className="w-full bg-green-600 hover:bg-green-700 disabled:bg-gray-400 text-white font-bold py-3 px-4 rounded-lg"
            >
              {loading ? 'Staking...' : 'Stake REGEN'}
            </button>
          </div>

          <div className="bg-red-50 p-6 rounded-lg">
            <h2 className="text-xl font-bold text-red-800 mb-4">Unbond dREGEN</h2>
            <div className="mb-4">
              <label className="block text-sm font-medium text-gray-700 mb-2">Amount (dREGEN)</label>
              <input
                type="number"
                value={unbondAmount}
                onChange={(e) => setUnbondAmount(e.target.value)}
                className="w-full p-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-red-500"
                placeholder="Enter dREGEN amount to unbond"
              />
            </div>
            {unbondAmount && (
              <p className="text-sm text-gray-600 mb-4">
                You will receive ~{expectedRegenFromUnbond()} REGEN after unbonding period
              </p>
            )}
            <button
              onClick={handleUnbond}
              disabled={loading || !unbondAmount}
              className="w-full bg-red-600 hover:bg-red-700 disabled:bg-gray-400 text-white font-bold py-3 px-4 rounded-lg"
            >
              {loading ? 'Unbonding...' : 'Unbond dREGEN'}
            </button>
          </div>
        </div>
      )}
    </div>
  );
};

export default StakingInterface;