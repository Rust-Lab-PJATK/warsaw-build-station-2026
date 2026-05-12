"use client";

import { PublicKey } from "@solana/web3.js";
import { useTaskAccount } from "@/hooks/useTaskAccount";
import { EscrowVaultBalance } from "./EscrowVaultBalance";
import { DeadlineCountdown } from "./DeadlineCountdown";
import { ProofBadge, ProofState } from "./ProofBadge";
import { StatusBadge } from "./StatusBadge";
import { ActionPanel } from "./ActionPanel";
import { AdvanceBreakdown } from "./AdvanceBreakdown";
import {
  getStatusVariant,
  lamportsToSol,
  bytesToHex,
} from "@/types/marketplace";
import { shortenPubkey } from "@/lib/pdas";
import { ExternalLinkIcon, CopyIcon, Check } from "lucide-react";
import { useState } from "react";
import { cn } from "@/lib/cn";

const GRADIENTS = [
  "from-violet-500 to-purple-700",
  "from-emerald-400 to-teal-600",
  "from-orange-400 to-red-600",
  "from-sky-400 to-blue-600",
  "from-pink-400 to-rose-600",
  "from-amber-400 to-orange-600",
  "from-cyan-400 to-blue-500",
  "from-lime-400 to-green-600",
];

function pubkeyGradient(pubkey: PublicKey): string {
  const sum = pubkey.toBytes().slice(0, 4).reduce((a, b) => a + b, 0);
  return GRADIENTS[sum % GRADIENTS.length];
}

function CopyableAddress({ pubkey }: { pubkey: PublicKey }) {
  const [copied, setCopied] = useState(false);
  const copy = () => {
    navigator.clipboard.writeText(pubkey.toBase58());
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };
  return (
    <button
      onClick={copy}
      className="flex items-center gap-1.5 font-mono text-xs text-gray-400 hover:text-white transition-colors"
    >
      {shortenPubkey(pubkey)}
      {copied ? (
        <Check className="h-3 w-3 text-accent-green" />
      ) : (
        <CopyIcon className="h-3 w-3" />
      )}
    </button>
  );
}

function InfoRow({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex items-center justify-between py-2.5 border-b border-white/[0.06] last:border-0">
      <span className="text-sm text-gray-500">{label}</span>
      <div className="text-sm text-white">{children}</div>
    </div>
  );
}

function deriveProofState(
  status: ReturnType<typeof getStatusVariant>,
  resultHash: number[] | null
): ProofState {
  if (status === "approved") return "verified";
  if (status === "slashed") return "invalid";
  if (resultHash && resultHash.some((b) => b !== 0)) return "pending";
  return "none";
}

interface Props {
  taskPubkey: PublicKey;
}

export function TaskDetailView({
  task,
  taskPubkey,
  onSuccess,
}: {
  task: import("@/types/marketplace").TaskAccount;
  taskPubkey: PublicKey;
  onSuccess?: () => void;
}) {
  const status = getStatusVariant(task.status);
  const submitDeadline = task.submitDeadline.toNumber();
  const disputeDeadline = task.disputeDeadline.toNumber();
  const proofState = deriveProofState(status, task.resultHash);

  const showSubmitCountdown = status === "claimed" && submitDeadline > 0;
  const showDisputeCountdown = status === "submitted" && disputeDeadline > 0;

  const gradient = pubkeyGradient(taskPubkey);
  const letter = taskPubkey.toBase58()[0].toUpperCase();

  return (
    <div className="mx-auto max-w-5xl space-y-6 px-4 pb-8">
      {/* ── Gradient hero ────────────────────────────────────── */}
      <div className={`relative rounded-lg overflow-hidden bg-gradient-to-br ${gradient}`}>
        <div className="absolute inset-0 bg-black/20" />
        <div className="relative px-6 py-8 flex flex-col sm:flex-row sm:items-end justify-between gap-4">
          <div className="flex items-end gap-4">
            <div className="h-14 w-14 rounded-lg bg-black/30 backdrop-blur-sm border border-white/20 flex items-center justify-center shrink-0">
              <span className="text-2xl font-bold text-white">{letter}</span>
            </div>
            <div>
              <div className="flex items-center gap-2 flex-wrap">
                <h1 className="text-xl font-bold text-white">Task #{task.taskId.toString()}</h1>
                <StatusBadge variant={status} />
              </div>
              <div className="mt-1 flex items-center gap-1.5">
                <span className="text-xs text-white/60">PDA:</span>
                <CopyableAddress pubkey={taskPubkey} />
                <a
                  href={`https://explorer.solana.com/address/${taskPubkey.toBase58()}?cluster=devnet`}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-white/50 hover:text-white/90 transition-colors"
                >
                  <ExternalLinkIcon className="h-3 w-3" />
                </a>
              </div>
            </div>
          </div>
          <div className="rounded-lg bg-black/30 backdrop-blur-sm border border-white/20 px-5 py-3 text-right shrink-0">
            <p className="text-xs text-white/60 mb-0.5">Total Reward</p>
            <p className="font-mono text-2xl font-bold text-white">
              ◎ {lamportsToSol(task.reward).toFixed(4)}
            </p>
          </div>
        </div>
      </div>

      {/* ── Main grid ────────────────────────────────────────── */}
      <div className="grid gap-5 lg:grid-cols-3">
        {/* Left column — metadata */}
        <div className="space-y-5 lg:col-span-2">
          {/* Task info */}
          <div className="rounded-lg border border-white/[0.08] bg-surface-card p-5">
            <p className="mb-3 text-xs font-semibold uppercase tracking-widest text-gray-500">
              Task Details
            </p>
            <InfoRow label="Client">
              <CopyableAddress pubkey={task.client} />
            </InfoRow>
            <InfoRow label="Agent">
              {task.agent ? (
                <CopyableAddress pubkey={task.agent} />
              ) : (
                <span className="text-gray-600 italic">Unclaimed</span>
              )}
            </InfoRow>
            <InfoRow label="Required Stake">
              <span className="font-mono text-accent-teal">
                ◎ {lamportsToSol(task.requiredStake).toFixed(4)}
              </span>
            </InfoRow>
            <InfoRow label="Description Hash">
              <span className="font-mono text-xs text-gray-400">
                {bytesToHex(task.descriptionHash).slice(0, 20)}…
              </span>
            </InfoRow>
          </div>

          {/* ZK proof badge */}
          <ProofBadge state={proofState} resultHash={task.resultHash} />

          {/* Action buttons */}
          <div className="rounded-lg border border-white/[0.08] bg-surface-card p-5">
            <p className="mb-3 text-xs font-semibold uppercase tracking-widest text-gray-500">
              Actions
            </p>
            <ActionPanel
              task={task}
              taskPubkey={taskPubkey}
              onSuccess={onSuccess}
            />
          </div>
        </div>

        {/* Right column — live data */}
        <div className="space-y-5">
          <AdvanceBreakdown task={task} />

          <EscrowVaultBalance
            taskPubkey={taskPubkey}
            agentPubkey={task.agent}
          />

          {showSubmitCountdown && (
            <DeadlineCountdown
              label="Submit Deadline"
              deadlineUnix={submitDeadline}
            />
          )}

          {showDisputeCountdown && (
            <DeadlineCountdown
              label="Dispute Window"
              deadlineUnix={disputeDeadline}
            />
          )}

          {/* Explorer link */}
          <a
            href={`https://explorer.solana.com/address/${taskPubkey.toBase58()}?cluster=devnet`}
            target="_blank"
            rel="noopener noreferrer"
            className={cn(
              "flex items-center justify-center gap-2 rounded-lg border border-white/[0.08]",
              "bg-surface-card px-4 py-3 text-sm text-gray-400 hover:text-white hover:border-gray-600 transition-all"
            )}
          >
            <ExternalLinkIcon className="h-4 w-4" />
            View on Solana Explorer
          </a>
        </div>
      </div>
    </div>
  );
}

export function TaskDetail({ taskPubkey }: Props) {
  const { task, loading, error, refresh } = useTaskAccount(taskPubkey);

  if (loading) {
    return (
      <div className="mx-auto max-w-5xl px-4 pb-8 space-y-6 animate-pulse">
        <div className="h-36 rounded-lg bg-surface-elevated" />
        <div className="grid gap-5 lg:grid-cols-3">
          <div className="lg:col-span-2 space-y-5">
            <div className="h-48 rounded-lg bg-surface-elevated" />
            <div className="h-24 rounded-lg bg-surface-elevated" />
          </div>
          <div className="space-y-5">
            <div className="h-40 rounded-lg bg-surface-elevated" />
            <div className="h-24 rounded-lg bg-surface-elevated" />
          </div>
        </div>
      </div>
    );
  }

  if (error || !task) {
    return (
      <div className="mx-auto max-w-5xl px-4 pb-8">
        <div className="rounded-lg border border-accent-red/30 bg-accent-red/5 p-10 text-center">
          <p className="font-bold text-accent-red text-lg">Task not found</p>
          <p className="mt-2 text-sm text-gray-500">{error ?? "Account not found on devnet"}</p>
          <a href="/agent" className="mt-6 inline-flex items-center gap-2 rounded-lg border border-white/[0.08] bg-white/[0.02] px-5 py-2.5 text-sm text-gray-300 hover:text-white transition-colors">
            ← Back to marketplace
          </a>
        </div>
      </div>
    );
  }

  return <TaskDetailView task={task} taskPubkey={taskPubkey} onSuccess={refresh} />;
}
