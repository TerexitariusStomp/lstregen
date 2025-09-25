import { Entity, PrimaryGeneratedColumn, Column } from 'typeorm';

@Entity({ name: 'stake_events' })
export class StakeEvent {
  @PrimaryGeneratedColumn()
  id!: number;

  @Column({ type: 'varchar', length: 128 })
  txHash!: string;

  @Column({ type: 'integer' })
  height!: number;

  @Column({ type: 'timestamptz' })
  timestamp!: Date;

  @Column({ type: 'varchar', length: 128 })
  staker!: string;

  @Column({ type: 'varchar', length: 64 })
  regenAmount!: string;

  @Column({ type: 'varchar', length: 64 })
  dregenAmount!: string;

  @Column({ type: 'double precision', nullable: true })
  exchangeRate!: number;
}

@Entity({ name: 'unbond_events' })
export class UnbondEvent {
  @PrimaryGeneratedColumn()
  id!: number;

  @Column({ type: 'varchar', length: 128 })
  txHash!: string;

  @Column({ type: 'integer' })
  height!: number;

  @Column({ type: 'timestamptz' })
  timestamp!: Date;

  @Column({ type: 'varchar', length: 128 })
  user!: string;

  @Column({ type: 'varchar', length: 64 })
  dregenAmount!: string;

  @Column({ type: 'varchar', length: 64 })
  regenAmount!: string;

  @Column({ type: 'integer' })
  unbondingId!: number;

  @Column({ type: 'timestamptz' })
  completionTime!: Date;
}

@Entity({ name: 'reward_events' })
export class RewardEvent {
  @PrimaryGeneratedColumn()
  id!: number;

  @Column({ type: 'varchar', length: 128 })
  txHash!: string;

  @Column({ type: 'integer' })
  height!: number;

  @Column({ type: 'timestamptz' })
  timestamp!: Date;

  @Column({ type: 'varchar', length: 128, nullable: true })
  claimer!: string;
}