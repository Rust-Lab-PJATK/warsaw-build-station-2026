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
    <div className="flex items-center justify-between py-2.5 border-b border-surface-elevated last:border-0">
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

  return (
    <div className="mx-auto max-w-5xl space-y-6 px-4 py-8">
      {/* ── Header ───────────────────────────────────────────── */}
      <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
        <div>
          <div className="flex items-center gap-3">
            <h1 className="text-2xl font-bold text-white">
              Task #{task.taskId.toString()}
            </h1>
            <StatusBadge variant={status} />
          </div>
          <div className="mt-1 flex items-center gap-1.5">
            <span className="text-xs text-gray-600">Task PDA:</span>
            <CopyableAddress pubkey={taskPubkey} />
            <a
              href={`https://explorer.solana.com/address/${taskPubkey.toBase58()}?cluster=devnet`}
              target="_blank"
              rel="noopener noreferrer"
              className="text-gray-600 hover:text-gray-300 transition-colors"
            >
              <ExternalLinkIcon className="h-3 w-3" />
            </a>
          </div>
        </div>

        <div className="flex items-center gap-2 rounded-xl border border-surface-elevated bg-surface-card px-4 py-2">
          <span className="text-xs text-gray-500">Reward</span>
          <span className="font-mono text-xl font-bold text-accent-green">
            ◎ {lamportsToSol(task.reward).toFixed(4)}
          </span>
        </div>
      </div>

      {/* ── Main grid ────────────────────────────────────────── */}
      <div className="grid gap-5 lg:grid-cols-3">
        {/* Left column — metadata */}
        <div className="space-y-5 lg:col-span-2">
          {/* Task info */}
          <div className="rounded-xl border border-surface-elevated bg-surface-card p-5">
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
              <span className="font-mono text-accent-purple">
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
          <div className="rounded-xl border border-surface-elevated bg-surface-card p-5">
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
              "flex items-center justify-center gap-2 rounded-xl border border-surface-elevated",
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
      <div className="flex h-64 items-center justify-center">
        <div className="h-8 w-8 animate-spin rounded-full border-2 border-accent-purple border-t-transparent" />
      </div>
    );
  }

  if (error || !task) {
    return (
      <div className="rounded-xl border border-accent-red/30 bg-accent-red/5 p-6 text-center">
        <p className="font-semibold text-accent-red">Failed to load task</p>
        <p className="mt-1 text-sm text-gray-500">{error ?? "Account not found"}</p>
      </div>
    );
  }

  return <TaskDetailView task={task} taskPubkey={taskPubkey} onSuccess={refresh} />;
}
