import { PublicKey } from "@solana/web3.js";
import BN from "bn.js";

// Anchor represents enum variants as { variantName: {} }
export interface TaskStatus {
  open?: Record<string, never>;
  claimed?: Record<string, never>;
  submitted?: Record<string, never>;
  approved?: Record<string, never>;
  disputed?: Record<string, never>;
  slashed?: Record<string, never>;
}

export type TaskStatusVariant =
  | "open"
  | "claimed"
  | "submitted"
  | "approved"
  | "disputed"
  | "slashed";

export interface TaskAccount {
  client: PublicKey;
  agent: PublicKey | null;
  taskId: BN;
  reward: BN;
  requiredStake: BN;
  advanceBps: number;
  advancePaid: BN;
  descriptionHash: number[];
  resultHash: number[] | null;
  status: TaskStatus;
  submitDeadline: BN;
  disputeDeadline: BN;
  taskBump: number;
  vaultBump: number;
  stakeBump: number;
}

export function getStatusVariant(status: TaskStatus): TaskStatusVariant {
  if ("open" in status) return "open";
  if ("claimed" in status) return "claimed";
  if ("submitted" in status) return "submitted";
  if ("approved" in status) return "approved";
  if ("disputed" in status) return "disputed";
  return "slashed";
}

export const STATUS_LABELS: Record<TaskStatusVariant, string> = {
  open: "Open",
  claimed: "Claimed",
  submitted: "Submitted",
  approved: "Approved",
  disputed: "Disputed",
  slashed: "Slashed",
};

export const LAMPORTS_PER_SOL = 1_000_000_000;

export function lamportsToSol(lamports: BN | number): number {
  const n = typeof lamports === "number" ? lamports : lamports.toNumber();
  return n / LAMPORTS_PER_SOL;
}

export function bytesToHex(bytes: number[]): string {
  return bytes.map((b) => b.toString(16).padStart(2, "0")).join("");
}

export async function sha256Bytes(text: string): Promise<number[]> {
  const data = new TextEncoder().encode(text);
  const hash = await crypto.subtle.digest("SHA-256", data);
  return Array.from(new Uint8Array(hash));
}

export function advanceLamports(task: TaskAccount): number {
  return task.advancePaid.toNumber();
}

export function remainingLamports(task: TaskAccount): number {
  return task.reward.toNumber() - task.advancePaid.toNumber();
}
