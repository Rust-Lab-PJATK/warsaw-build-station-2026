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
        <label className="text-xs font-semibold uppercase tracking-widest text-gray-500">
          Task Description
        </label>
        <textarea
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          placeholder="Describe what you need the AI agent to do in detail…"
          rows={5}
          className="w-full rounded-lg border border-white/[0.08] bg-surface-card px-4 py-3 text-sm text-white placeholder-gray-500 outline-none focus:border-white/[0.12] transition-colors resize-none"
        />
        <div className="flex items-center justify-between">
          <span className="text-xs text-gray-500">{description.length} chars — min 10</span>
          <button
            onClick={handleEstimate}
            disabled={!canEstimate}
            className={cn(
              "flex items-center gap-2 rounded-lg px-4 py-2 text-sm font-semibold transition-all",
              canEstimate
                ? "bg-accent-teal text-black hover:bg-[var(--accent-primary-hover)]"
                : "bg-surface-elevated text-gray-500 cursor-not-allowed"
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
          <p className="text-xs text-accent-yellow">{estimateError}</p>
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
            <label className="text-xs font-semibold uppercase tracking-widest text-gray-500">
              Reward (SOL)
            </label>
            <div className="flex items-center gap-2 rounded-lg border border-white/[0.08] bg-surface-card px-3 py-3">
              <span className="font-bold text-accent-teal">◎</span>
              <input
                type="number"
                value={rewardSol}
                onChange={(e) => setRewardSol(e.target.value)}
                placeholder="0.00"
                step="0.01"
                className="flex-1 bg-transparent font-mono text-white outline-none"
              />
            </div>
          </div>
          <div className="space-y-2">
            <label className="text-xs font-semibold uppercase tracking-widest text-gray-500">
              Required Stake (SOL)
            </label>
            <div className="flex items-center gap-2 rounded-lg border border-white/[0.08] bg-surface-card px-3 py-3">
              <span className="font-bold text-accent-yellow">◎</span>
              <input
                type="number"
                value={stakeSol}
                onChange={(e) => setStakeSol(e.target.value)}
                placeholder="0.00"
                step="0.01"
                className="flex-1 bg-transparent font-mono text-white outline-none"
              />
            </div>
          </div>
        </div>
      )}

      {/* Stake input shown below estimate */}
      {estimate && (
        <div className="space-y-2">
          <label className="text-xs font-semibold uppercase tracking-widest text-gray-500">
            Required Agent Stake (SOL)
          </label>
          <div className="flex items-center gap-2 rounded-lg border border-white/[0.08] bg-surface-card px-3 py-3">
            <span className="font-bold text-accent-yellow">◎</span>
            <input
              type="number"
              value={stakeSol}
              onChange={(e) => setStakeSol(e.target.value)}
              step="0.01"
              className="flex-1 bg-transparent font-mono text-white outline-none"
            />
          </div>
          <p className="text-xs text-gray-500">
            Slashed to you if the agent misses their deadline
          </p>
        </div>
      )}

      {/* Submit */}
      {txError && (
        <p className="rounded-lg border border-accent-red/20 bg-accent-red/[0.08] px-3 py-2 text-sm text-accent-red">
          {txError}
        </p>
      )}

      {!publicKey ? (
        <p className="rounded-lg border border-white/[0.08] bg-white/[0.02] px-4 py-3 text-center text-sm text-gray-500">
          Connect your wallet to post a task
        </p>
      ) : (
        <button
          onClick={handlePost}
          disabled={!canPost}
          className={cn(
            "flex w-full items-center justify-center gap-2 rounded-lg py-4 font-semibold transition-all",
            canPost
              ? "bg-white text-black hover:bg-neutral-200"
              : "border border-white/[0.08] bg-white/[0.02] text-gray-600 cursor-not-allowed"
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
