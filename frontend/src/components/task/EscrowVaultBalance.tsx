"use client";

import { PublicKey } from "@solana/web3.js";
import { useVaultBalance, useStakeBalance } from "@/hooks/useVaultBalance";
import { lamportsToSol } from "@/types/marketplace";
import { cn } from "@/lib/cn";
import { LockIcon } from "lucide-react";

interface Props {
  taskPubkey: PublicKey;
  agentPubkey: PublicKey | null;
}

function BalanceRow({
  label,
  lamports,
  accent,
}: {
  label: string;
  lamports: number | null;
  accent: string;
}) {
  const sol = lamports !== null ? lamportsToSol(lamports) : null;

  return (
    <div className="flex items-center justify-between rounded-lg bg-surface px-4 py-3">
      <span className="text-sm text-gray-400">{label}</span>
      <div className="flex items-center gap-2">
        {sol !== null ? (
          <span className={cn("font-mono text-lg font-bold", accent)}>
            ◎ {sol.toFixed(4)}
          </span>
        ) : (
          <span className="font-mono text-lg text-gray-600 animate-pulse">
            ◎ ——
          </span>
        )}
      </div>
    </div>
  );
}

export function EscrowVaultBalance({ taskPubkey, agentPubkey }: Props) {
  const rewardBalance = useVaultBalance(taskPubkey);
  const stakeBalance = useStakeBalance(taskPubkey, agentPubkey);

  const totalLocked =
    rewardBalance !== null && stakeBalance !== null
      ? rewardBalance + stakeBalance
      : rewardBalance ?? null;

  return (
    <div className="rounded-lg border border-white/[0.08] bg-surface-card p-5 space-y-3">
      {/* Header */}
      <div className="flex items-center gap-2 mb-1">
        <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-accent-teal/10">
          <LockIcon className="h-4 w-4 text-accent-teal" />
        </div>
        <div>
          <p className="text-sm font-semibold text-white">Escrow Vault</p>
          <p className="text-xs text-gray-500">Live on-chain balances</p>
        </div>
        {/* Live indicator */}
        <span className="ml-auto flex items-center gap-1 rounded-full bg-accent-green/10 px-2 py-0.5 text-xs text-accent-green">
          <span className="h-1.5 w-1.5 rounded-full bg-accent-green animate-pulse" />
          LIVE
        </span>
      </div>

      <BalanceRow
        label="Reward (vault)"
        lamports={rewardBalance}
        accent="text-accent-green"
      />

      {agentPubkey && (
        <BalanceRow
          label="Agent stake"
          lamports={stakeBalance}
          accent="text-accent-teal"
        />
      )}

      {totalLocked !== null && agentPubkey && (
        <div className="flex items-center justify-between border-t border-white/[0.08] pt-3">
          <span className="text-sm font-semibold text-gray-300">
            Total locked
          </span>
          <span className="font-mono text-xl font-bold text-white">
            ◎ {lamportsToSol(totalLocked).toFixed(4)}
          </span>
        </div>
      )}
    </div>
  );
}
