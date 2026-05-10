import { Keypair, PublicKey } from "@solana/web3.js";
import BN from "bn.js";
import { TaskAccount } from "@/types/marketplace";

function zeros(len: number) {
  return Array.from({ length: len }).map(() => 0);
}

export function makeMockTask(id: number, statusVariant: keyof TaskAccount['status']): { task: TaskAccount; pubkey: PublicKey } {
  const keypair = Keypair.generate();
  const client = Keypair.generate().publicKey;
  const agent = statusVariant === 'open' ? null : Keypair.generate().publicKey;
  const task: TaskAccount = {
    client,
    agent,
    taskId: new BN(id),
    reward: new BN(2_000_000_000),
    requiredStake: new BN(500_000_000),
    advanceBps: 2500,
    advancePaid: new BN(250_000_000),
    descriptionHash: zeros(32),
    resultHash: statusVariant === 'submitted' ? zeros(32) : null,
    status: { [statusVariant]: {} } as TaskAccount['status'],
    submitDeadline: new BN(Math.floor(Date.now() / 1000) + 3600),
    disputeDeadline: new BN(Math.floor(Date.now() / 1000) + 7200),
    taskBump: 1,
    vaultBump: 1,
    stakeBump: 1,
  };

  return { task, pubkey: keypair.publicKey };
}

export function makeMockTasks() {
  return [
    makeMockTask(1001, 'open'),
    makeMockTask(1002, 'claimed'),
    makeMockTask(1003, 'submitted'),
  ];
}

export default makeMockTasks;
