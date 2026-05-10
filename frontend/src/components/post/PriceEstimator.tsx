"use client";

import { cn } from "@/lib/cn";
import { SparklesIcon, ClockIcon, ZapIcon, Check } from "lucide-react";

export interface PriceEstimate {
  price_sol: number;
  advance_bps: number;
  complexity: "low" | "medium" | "high";
  reasoning: string;
  estimated_duration: string;
  similar: Array<{ description: string; price_sol: number; complexity: string }>;
}

const COMPLEXITY_STYLES = {
  low: "text-accent-green bg-accent-green/10 border-accent-green/30",
  medium: "text-accent-yellow bg-accent-yellow/10 border-accent-yellow/30",
  high: "text-accent-red bg-accent-red/10 border-accent-red/30",
};

interface Props {
  estimate: PriceEstimate;
  rewardSol: string;
  advancePct: number;
  onRewardChange: (v: string) => void;
  onAdvanceChange: (v: number) => void;
}

export function PriceEstimateCard({
  estimate,
  rewardSol,
  advancePct,
  onRewardChange,
  onAdvanceChange,
}: Props) {
  const advanceSol = (parseFloat(rewardSol || "0") * advancePct) / 100;
  const remainingSol = parseFloat(rewardSol || "0") - advanceSol;

  return (
    <div className="rounded-xl border border-accent-purple/30 bg-accent-purple/5 p-5 space-y-5">
      {/* AI badge */}
      <div className="flex items-center gap-2">
        <SparklesIcon className="h-4 w-4 text-accent-purple" />
        <span className="text-sm font-semibold text-accent-purple">AI Price Estimate</span>
        <span
          className={cn(
            "ml-auto rounded-full border px-2 py-0.5 text-xs font-semibold capitalize",
            COMPLEXITY_STYLES[estimate.complexity]
          )}
        >
          {estimate.complexity} complexity
        </span>
      </div>

      {/* Reasoning */}
      <p className="text-sm text-gray-300 italic">"{estimate.reasoning}"</p>

      {/* Duration */}
      <div className="flex items-center gap-2 text-sm text-gray-400">
        <ClockIcon className="h-4 w-4" />
        Estimated duration: <span className="text-white font-semibold">{estimate.estimated_duration}</span>
      </div>

      {/* Reward input */}
      <div className="space-y-2">
        <label className="text-xs font-semibold uppercase tracking-widest text-gray-500">
          Reward (SOL)
        </label>
        <div className="flex items-center gap-2 rounded-lg border border-surface-elevated bg-surface px-3 py-2">
          <span className="text-accent-green font-bold">◎</span>
          <input
            type="number"
            value={rewardSol}
            onChange={(e) => onRewardChange(e.target.value)}
            step="0.01"
            min="0.01"
            className="flex-1 bg-transparent font-mono text-lg text-white outline-none"
          />
        </div>
      </div>

      {/* Advance slider */}
      <div className="space-y-2">
        <div className="flex justify-between">
          <label className="text-xs font-semibold uppercase tracking-widest text-gray-500">
            Advance Payment
          </label>
          <span className="text-xs font-mono text-accent-purple">{advancePct}%</span>
        </div>
        <input
          type="range"
          min={0}
          max={50}
          step={5}
          value={advancePct}
          onChange={(e) => onAdvanceChange(parseInt(e.target.value))}
          className="w-full accent-[#3B82F6]"
        />
        {/* Split preview */}
        <div className="flex gap-2 text-xs">
          <div className="flex-1 rounded-lg bg-accent-purple/10 border border-accent-purple/20 px-3 py-2">
            <div className="flex items-center gap-1 text-accent-purple mb-1">
              <ZapIcon className="h-3 w-3" /> On claim
            </div>
            <span className="font-mono font-bold text-white">◎ {advanceSol.toFixed(4)}</span>
          </div>
          <div className="flex-1 rounded-lg bg-accent-green/10 border border-accent-green/20 px-3 py-2">
            <div className="flex items-center gap-1 text-accent-green mb-1">
              <Check className="h-3 w-3" />
              <span>On approval</span>
            </div>
            <span className="font-mono font-bold text-white">◎ {remainingSol.toFixed(4)}</span>
          </div>
        </div>
      </div>

      {/* Similar tasks */}
      {estimate.similar.length > 0 && (
        <div className="space-y-2">
          <p className="text-xs font-semibold uppercase tracking-widest text-gray-500">
            Similar completed tasks
          </p>
          {estimate.similar.map((t, i) => (
            <div
              key={i}
              className="flex items-center justify-between rounded-lg bg-surface px-3 py-2"
            >
              <span className="text-xs text-gray-400 truncate max-w-[70%]">{t.description}</span>
              <span className="font-mono text-xs text-white">◎ {t.price_sol}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
