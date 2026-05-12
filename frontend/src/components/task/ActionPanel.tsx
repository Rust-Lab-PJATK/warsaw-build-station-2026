"use client";

import { useState } from "react";
import { PublicKey } from "@solana/web3.js";
import { useWallet } from "@solana/wallet-adapter-react";
import { cn } from "@/lib/cn";
import {
  TaskAccount,
  getStatusVariant,
  lamportsToSol,
  remainingLamports,
  advanceLamports,
} from "@/types/marketplace";
import {
  useClaimTask,
  useApproveResult,
  useDisputeResult,
  useSlashTimeout,
} from "@/hooks/useMarketplace";
import { SubmitPreviewModal } from "./SubmitPreviewModal";
import {
  CheckCircleIcon,
  AlertTriangleIcon,
  ZapIcon,
  ScissorsIcon,
  FileTextIcon,
} from "lucide-react";

interface Props {
  task: TaskAccount;
  taskPubkey: PublicKey;
  onSuccess?: () => void;
}

function ActionButton({
  onClick,
  loading,
  disabled,
  variant,
  icon: Icon,
  label,
  sub,
}: {
  onClick: () => void;
  loading: boolean;
  disabled: boolean;
  variant: "green" | "purple" | "blue" | "yellow" | "red";
  icon: typeof CheckCircleIcon;
  label: string;
  sub: string;
}) {
  const colors = {
    green: "border-accent-green/30 bg-accent-green/10 text-accent-green hover:bg-accent-green/20 disabled:opacity-40",
    purple: "border-accent-teal/30 bg-accent-teal/10 text-accent-teal hover:bg-accent-teal/20 disabled:opacity-40",
    blue: "border-blue-500/30 bg-blue-500/10 text-blue-400 hover:bg-blue-500/20 disabled:opacity-40",
    yellow: "border-accent-yellow/30 bg-accent-yellow/10 text-accent-yellow hover:bg-accent-yellow/20 disabled:opacity-40",
    red: "border-accent-red/30 bg-accent-red/10 text-accent-red hover:bg-accent-red/20 disabled:opacity-40",
  };

  return (
    <button
      onClick={onClick}
      disabled={disabled || loading}
      className={cn(
        "flex w-full items-center gap-3 rounded-lg border px-4 py-3 transition-all",
        colors[variant]
      )}
    >
      <Icon className="h-5 w-5 shrink-0" />
      <div className="text-left">
        <p className="font-semibold">{loading ? "Sending transaction…" : label}</p>
        <p className="text-xs opacity-70">{sub}</p>
      </div>
    </button>
  );
}

export function ActionPanel({ task, taskPubkey, onSuccess }: Props) {
  const { publicKey } = useWallet();
  const status = getStatusVariant(task.status);
  const [showPreviewModal, setShowPreviewModal] = useState(false);

  const isClient = publicKey && task.client.equals(publicKey);
  const isAgent = publicKey && task.agent !== null && task.agent.equals(publicKey);

  const { claimTask, loading: claimLoading } = useClaimTask();
  const { approveResult, loading: approveLoading } = useApproveResult();
  const { disputeResult, loading: disputeLoading } = useDisputeResult();
  const { slashTimeout, loading: slashLoading } = useSlashTimeout();

  const now = Math.floor(Date.now() / 1000);
  const submitDeadline = task.submitDeadline.toNumber();
  const disputeDeadline = task.disputeDeadline.toNumber();

  const canSlash =
    (status === "claimed" && now > submitDeadline) ||
    (status === "disputed" && now > disputeDeadline + 86_400);

  const advanceSol = lamportsToSol(advanceLamports(task));
  const remainingSol = lamportsToSol(remainingLamports(task));
  const stakeSol = lamportsToSol(task.requiredStake);

  const actions: JSX.Element[] = [];

  // ── Agent: claim open task ───────────────────────────────────────────────────
  if (status === "open" && !isClient && publicKey) {
    actions.push(
      <ActionButton
        key="claim"
        onClick={async () => { await claimTask(taskPubkey); onSuccess?.(); }}
        loading={claimLoading}
        disabled={false}
        variant="purple"
        icon={ZapIcon}
        label={`Claim — stake ◎ ${stakeSol.toFixed(3)}, receive ◎ ${advanceSol.toFixed(3)} now`}
        sub={`◎ ${remainingSol.toFixed(3)} released when client approves your preview`}
      />
    );
  }

  // ── Agent: submit preview ────────────────────────────────────────────────────
  if (status === "claimed" && isAgent) {
    actions.push(
      <ActionButton
        key="preview"
        onClick={() => setShowPreviewModal(true)}
        loading={false}
        disabled={now > submitDeadline}
        variant="blue"
        icon={FileTextIcon}
        label="Submit Preview"
        sub="Share your work-in-progress IPFS CID or URL"
      />
    );
  }

  // ── Client: approve or dispute ───────────────────────────────────────────────
  if (status === "submitted" && isClient && task.agent) {
    actions.push(
      <ActionButton
        key="approve"
        onClick={async () => { await approveResult(taskPubkey, task.agent!); onSuccess?.(); }}
        loading={approveLoading}
        disabled={false}
        variant="green"
        icon={CheckCircleIcon}
        label={`Approve Preview — release ◎ ${remainingSol.toFixed(3)}`}
        sub="Agent also gets their stake returned"
      />,
      <ActionButton
        key="dispute"
        onClick={async () => { await disputeResult(taskPubkey); onSuccess?.(); }}
        loading={disputeLoading}
        disabled={now > disputeDeadline}
        variant="yellow"
        icon={AlertTriangleIcon}
        label="Dispute"
        sub="Challenge within the dispute window"
      />
    );
  }

  // ── Anyone: slash timed-out agent ────────────────────────────────────────────
  if (canSlash) {
    actions.push(
      <ActionButton
        key="slash"
        onClick={async () => { await slashTimeout(taskPubkey, task.client, task.agent!); onSuccess?.(); }}
        loading={slashLoading}
        disabled={false}
        variant="red"
        icon={ScissorsIcon}
        label="Slash Agent"
        sub="Agent missed their deadline — anyone can trigger this"
      />
    );
  }

  return (
    <>
      {showPreviewModal && (
        <SubmitPreviewModal
          taskPubkey={taskPubkey}
          onSuccess={onSuccess}
          onClose={() => setShowPreviewModal(false)}
        />
      )}

      {actions.length === 0 ? (
        <div className="rounded-lg border border-white/[0.08] bg-surface-card px-4 py-5 text-center">
          <p className="text-sm text-gray-500">
            {!publicKey
              ? "Connect your wallet to interact"
              : "No actions available for your role at this stage"}
          </p>
        </div>
      ) : (
        <div className="space-y-3">{actions}</div>
      )}
    </>
  );
}
