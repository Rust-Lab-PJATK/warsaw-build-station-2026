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
import { PriceEstimateCard, PriceEstimate } from "./PriceEstimator";
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";

export function PostTaskForm() {
  const router = useRouter();
  const { publicKey } = useWallet();
  const { postTask, loading: txLoading, error: txError } = usePostTask();

  const [description, setDescription] = useState("");
  const [estimate, setEstimate] = useState<PriceEstimate | null>(null);
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
      const data: PriceEstimate = await res.json();
      setEstimate(data);
      setRewardSol(data.price_sol.toFixed(4));
      setAdvancePct(Math.round(data.advance_bps / 100));
      setStakeSol((data.price_sol * 0.1).toFixed(4));
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
      router.push(`/task/${taskPda.toBase58()}`);
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
          className="w-full rounded-lg border border-white/[0.08] bg-surface px-4 py-3 text-sm text-white placeholder-gray-600 outline-none focus:border-accent-teal/50 transition-colors resize-none"
        />
        <div className="flex items-center justify-between">
          <span className="text-xs text-gray-600">{description.length} chars — min 10</span>
          <button
            onClick={handleEstimate}
            disabled={!canEstimate}
            className={cn(
              "flex items-center gap-2 rounded-lg px-4 py-2 text-sm font-semibold transition-all",
              canEstimate
                ? "bg-white text-black hover:bg-neutral-200"
                : "border border-white/10 bg-white/[0.02] text-gray-600 cursor-not-allowed"
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

      {/* Manual inputs (shown always, pre-filled after estimate) */}
      {!estimate && (
        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-2">
            <label className="text-xs font-semibold uppercase tracking-widest text-gray-500">
              Reward (SOL)
            </label>
            <div className="flex items-center gap-2 rounded-lg border border-white/[0.08] bg-surface px-3 py-3">
              <span className="text-accent-green font-bold">◎</span>
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
            <div className="flex items-center gap-2 rounded-lg border border-white/[0.08] bg-surface px-3 py-3">
              <span className="text-accent-teal font-bold">◎</span>
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

      {/* Stake input shown below estimate too */}
      {estimate && (
        <div className="space-y-2">
          <label className="text-xs font-semibold uppercase tracking-widest text-gray-500">
            Required Agent Stake (SOL)
          </label>
          <div className="flex items-center gap-2 rounded-lg border border-white/[0.08] bg-surface px-3 py-3">
            <span className="text-accent-teal font-bold">◎</span>
            <input
              type="number"
              value={stakeSol}
              onChange={(e) => setStakeSol(e.target.value)}
              step="0.01"
              className="flex-1 bg-transparent font-mono text-white outline-none"
            />
          </div>
          <p className="text-xs text-gray-600">
            Stake is slashed to you if the agent misses their deadline
          </p>
        </div>
      )}

      {/* Submit */}
      {txError && (
        <p className="rounded-lg bg-accent-red/10 border border-accent-red/30 px-3 py-2 text-sm text-accent-red">
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
            "flex w-full items-center justify-center gap-2 rounded-lg py-4 font-semibold transition-all",
            canPost
              ? "bg-white text-black hover:bg-neutral-200"
              : "border border-white/10 bg-white/[0.02] text-gray-600 cursor-not-allowed"
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
