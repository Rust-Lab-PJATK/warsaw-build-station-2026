"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { useWallet } from "@solana/wallet-adapter-react";
import { BN } from "@coral-xyz/anchor";
import { SparklesIcon, Loader2Icon, SendIcon } from "lucide-react";
import { cn } from "@/lib/cn";
import { LAMPORTS_PER_SOL, sha256Bytes } from "@/types/marketplace";
import { usePostTask } from "@/hooks/useMarketplace";
import { findTaskPda } from "@/lib/pdas";
import { PriceEstimateCard } from "./PriceEstimator";
import type { BackendEstimateResponse } from "@/lib/backendApi";
import { linkTask, storeJobId } from "@/lib/backendApi";
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";

export function PostTaskForm() {
  const router = useRouter();
  const { publicKey } = useWallet();
  const { postTask, loading: txLoading, error: txError } = usePostTask();

  const [description, setDescription] = useState("");
  const [estimate, setEstimate] = useState<BackendEstimateResponse | null>(null);
  const [estimating, setEstimating] = useState(false);
  const [estimateError, setEstimateError] = useState("");

  const [rewardSol, setRewardSol] = useState("");
  const [advancePct, setAdvancePct] = useState(20);
  const [stakeSol, setStakeSol] = useState("");

  async function handleEstimate() {
    if (description.trim().length < 10) return;
    setEstimating(true);
    setEstimateError("");
    setEstimate(null);
    try {
      const res = await fetch("/api/estimate-price", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ description }),
      });
      if (!res.ok) throw new Error("Failed");
      const data: BackendEstimateResponse = await res.json();
      setEstimate(data);
      setRewardSol(data.total_price_sol.toFixed(4));
      setStakeSol((data.total_price_sol * 0.1).toFixed(4));
    } catch {
      setEstimateError("Could not estimate. Fill in price manually below.");
    } finally {
      setEstimating(false);
    }
  }

  async function handlePost() {
    if (!publicKey || !rewardSol || !stakeSol) return;
    const taskId = new BN(Date.now());
    const rewardLamports = new BN(Math.round(parseFloat(rewardSol) * LAMPORTS_PER_SOL));
    const stakeLamports = new BN(Math.round(parseFloat(stakeSol) * LAMPORTS_PER_SOL));
    const advanceBps = advancePct * 100;
    const descHash = await sha256Bytes(description || "task");

    const result = await postTask(taskId, rewardLamports, stakeLamports, descHash, advanceBps);
    if (result) {
      const [taskPda] = findTaskPda(publicKey, taskId);
      const taskPubkey = taskPda.toBase58();

      // Link on-chain pubkey to the MongoDB job if we have a job_id
      if (estimate?.job_id) {
        try {
          await linkTask(estimate.job_id, taskPubkey);
          storeJobId(taskPubkey, estimate.job_id);
        } catch (e) {
          console.warn("link-task failed:", e);
        }
      }

      router.push(`/task/${taskPubkey}`);
    }
  }

  const canEstimate = description.trim().length >= 10 && !estimating;
  const canPost = publicKey && rewardSol && stakeSol && !txLoading;

  return (
    <div className="space-y-6">
      {/* Description */}
      <div className="space-y-2">
        <label className="text-xs font-semibold uppercase tracking-widest text-[var(--text-muted)]">
          Task Description
        </label>
        <textarea
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          placeholder="Describe what you need the AI agent to do in detail…"
          rows={5}
          className="w-full rounded-[var(--radius-lg)] border border-[var(--border-subtle)] bg-[var(--bg-surface)] px-4 py-3 text-sm text-[var(--text-primary)] placeholder-[var(--text-muted)] outline-none focus:border-[var(--border-default)] transition-colors resize-none"
        />
        <div className="flex items-center justify-between">
          <span className="text-xs text-[var(--text-muted)]">{description.length} chars — min 10</span>
          <button
            onClick={handleEstimate}
            disabled={!canEstimate}
            className={cn(
              "flex items-center gap-2 rounded-[var(--radius-md)] px-4 py-2 text-sm font-semibold transition-all",
              canEstimate
                ? "bg-[var(--accent-primary)] text-[var(--text-inverse)] hover:bg-[var(--accent-primary-hover)]"
                : "bg-[var(--bg-elevated)] text-[var(--text-muted)] cursor-not-allowed"
            )}
          >
            {estimating ? (
              <Loader2Icon className="h-4 w-4 animate-spin" />
            ) : (
              <SparklesIcon className="h-4 w-4" />
            )}
            {estimating ? "Analyzing…" : "Get AI Price"}
          </button>
        </div>
        {estimateError && (
          <p className="text-xs text-[var(--accent-amber)]">{estimateError}</p>
        )}
      </div>

      {/* AI Estimate card */}
      {estimate && (
        <PriceEstimateCard
          estimate={estimate}
          rewardSol={rewardSol}
          advancePct={advancePct}
          onRewardChange={setRewardSol}
          onAdvanceChange={setAdvancePct}
        />
      )}

      {/* Manual inputs shown when no estimate yet */}
      {!estimate && (
        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-2">
            <label className="text-xs font-semibold uppercase tracking-widest text-[var(--text-muted)]">
              Reward (SOL)
            </label>
            <div className="flex items-center gap-2 rounded-[var(--radius-lg)] border border-[var(--border-subtle)] bg-[var(--bg-surface)] px-3 py-3">
              <span className="font-bold text-[var(--accent-primary)]">◎</span>
              <input
                type="number"
                value={rewardSol}
                onChange={(e) => setRewardSol(e.target.value)}
                placeholder="0.00"
                step="0.01"
                className="flex-1 bg-transparent font-mono text-[var(--text-primary)] outline-none"
              />
            </div>
          </div>
          <div className="space-y-2">
            <label className="text-xs font-semibold uppercase tracking-widest text-[var(--text-muted)]">
              Required Stake (SOL)
            </label>
            <div className="flex items-center gap-2 rounded-[var(--radius-lg)] border border-[var(--border-subtle)] bg-[var(--bg-surface)] px-3 py-3">
              <span className="font-bold text-[var(--accent-amber)]">◎</span>
              <input
                type="number"
                value={stakeSol}
                onChange={(e) => setStakeSol(e.target.value)}
                placeholder="0.00"
                step="0.01"
                className="flex-1 bg-transparent font-mono text-[var(--text-primary)] outline-none"
              />
            </div>
          </div>
        </div>
      )}

      {/* Stake input shown below estimate */}
      {estimate && (
        <div className="space-y-2">
          <label className="text-xs font-semibold uppercase tracking-widest text-[var(--text-muted)]">
            Required Agent Stake (SOL)
          </label>
          <div className="flex items-center gap-2 rounded-[var(--radius-lg)] border border-[var(--border-subtle)] bg-[var(--bg-surface)] px-3 py-3">
            <span className="font-bold text-[var(--accent-amber)]">◎</span>
            <input
              type="number"
              value={stakeSol}
              onChange={(e) => setStakeSol(e.target.value)}
              step="0.01"
              className="flex-1 bg-transparent font-mono text-[var(--text-primary)] outline-none"
            />
          </div>
          <p className="text-xs text-[var(--text-muted)]">
            Slashed to you if the agent misses their deadline
          </p>
        </div>
      )}

      {/* Submit */}
      {txError && (
        <p className="rounded-[var(--radius-md)] border border-[var(--accent-red-dim)] bg-[var(--accent-red-dim)] px-3 py-2 text-sm text-[var(--accent-red)]">
          {txError}
        </p>
      )}

      {!publicKey ? (
        <div className="flex justify-center">
          <WalletMultiButton />
        </div>
      ) : (
        <button
          onClick={handlePost}
          disabled={!canPost}
          className={cn(
            "flex w-full items-center justify-center gap-2 rounded-[var(--radius-md)] py-4 font-semibold transition-all",
            canPost
              ? "bg-[var(--accent-primary)] text-[var(--text-inverse)] hover:bg-[var(--accent-primary-hover)]"
              : "bg-[var(--bg-elevated)] text-[var(--text-muted)] cursor-not-allowed"
          )}
        >
          {txLoading ? (
            <Loader2Icon className="h-5 w-5 animate-spin" />
          ) : (
            <SendIcon className="h-5 w-5" />
          )}
          {txLoading ? "Posting to Solana…" : "Post Task & Lock Escrow"}
        </button>
      )}
    </div>
  );
}
