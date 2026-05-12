import { PublicKey } from "@solana/web3.js";
import BN from "bn.js";

const PROGRAM_ID_STR =
  process.env.NEXT_PUBLIC_PROGRAM_ID ?? "4Zf1emVAX8SKoVVpD7Jm75n9cA3WzKRZJQZmESZNHmvE";
export const PROGRAM_ID = new PublicKey(PROGRAM_ID_STR);

const TASK_SEED = Buffer.from("task");
const VAULT_SEED = Buffer.from("vault");
const STAKE_SEED = Buffer.from("stake");

export function findTaskPda(
  client: PublicKey,
  taskId: BN
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [TASK_SEED, client.toBuffer(), taskId.toArrayLike(Buffer, "le", 8)],
    PROGRAM_ID
  );
}

export function findVaultPda(task: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [VAULT_SEED, task.toBuffer()],
    PROGRAM_ID
  );
}

export function findStakePda(
  task: PublicKey,
  agent: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [STAKE_SEED, task.toBuffer(), agent.toBuffer()],
    PROGRAM_ID
  );
}

export function shortenPubkey(pk: PublicKey | string, chars = 4): string {
  const s = pk.toString();
  return `${s.slice(0, chars)}…${s.slice(-chars)}`;
}
