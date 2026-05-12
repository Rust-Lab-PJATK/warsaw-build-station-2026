"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { useWallet } from "@solana/wallet-adapter-react";
import { BN } from "@coral-xyz/anchor";
import { SparklesIcon, Loader2Icon, SendIcon, CheckCircleIcon, XCircleIcon } from "lucide-react";
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
  const [showModal, setShowModal] = useState(false);

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

      <div className="flex gap-3">
        <button
          onClick={() => setShowModal(true)}
          className="flex flex-1 items-center justify-center gap-2 rounded-[var(--radius-md)] border border-[var(--accent-primary)] bg-[var(--accent-primary-dim)] py-4 font-semibold text-[var(--accent-primary)] transition-all hover:bg-[var(--accent-primary)] hover:text-[var(--text-inverse)] active:scale-95"
        >
          <CheckCircleIcon className="h-5 w-5" />
          Accept
        </button>
        <button
          onClick={() => router.push("/")}
          className="flex flex-1 items-center justify-center gap-2 rounded-[var(--radius-md)] border border-[var(--accent-red-dim)] bg-[var(--accent-red-dim)] py-4 font-semibold text-[var(--accent-red)] transition-all hover:bg-[var(--accent-red)] hover:text-white active:scale-95"
        >
          <XCircleIcon className="h-5 w-5" />
          Reject
        </button>
      </div>

      {showModal && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
          onClick={() => setShowModal(false)}
        >
          <div
            className="mx-4 w-full max-w-sm rounded-[var(--radius-modal)] border border-[var(--accent-primary-dim)] bg-[var(--bg-surface)] p-8 text-center shadow-xl"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full border border-[var(--accent-primary-dim)] bg-[var(--accent-primary-dim)]">
              <CheckCircleIcon className="h-8 w-8 text-[var(--accent-primary)]" />
            </div>
            <h2 className="text-xl font-bold text-[var(--text-primary)]">Accepted!</h2>
            <p className="mt-2 text-sm text-[var(--text-secondary)]">
              The client has accepted the completed task.
              <br />
              Payment will be released from escrow.
            </p>
            <button
              onClick={() => setShowModal(false)}
              className="mt-6 w-full rounded-[var(--radius-md)] border border-[var(--accent-primary-dim)] bg-[var(--accent-primary-dim)] px-4 py-2.5 text-sm font-semibold text-[var(--accent-primary)] transition-all hover:bg-[var(--accent-primary)] hover:text-[var(--text-inverse)]"
            >
              Close
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
