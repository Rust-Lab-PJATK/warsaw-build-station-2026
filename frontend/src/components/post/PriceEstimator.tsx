"use client";

import { cn } from "@/lib/cn";
import { SparklesIcon, ChevronRightIcon } from "lucide-react";
import type { BackendEstimateResponse, BackendJobTask } from "@/lib/backendApi";

export type { BackendEstimateResponse as PriceEstimate };

const COMPLEXITY_COLORS: Record<number, string> = {
  1: "text-accent-teal bg-accent-teal/[0.08]",
  2: "text-accent-teal bg-accent-teal/[0.08]",
  3: "text-accent-yellow bg-accent-yellow/[0.08]",
  4: "text-accent-yellow bg-accent-yellow/[0.08]",
  5: "text-accent-red bg-accent-red/[0.08]",
};

const COMPLEXITY_LABEL: Record<number, string> = {
  1: "Trivial",
  2: "Easy",
  3: "Medium",
  4: "High",
  5: "Very High",
};

function ComplexityDots({ level }: { level: number }) {
  return (
    <div className="flex gap-0.5">
      {Array.from({ length: 5 }).map((_, i) => (
        <div
          key={i}
          className={cn(
            "h-1.5 w-1.5 rounded-full",
            i < level ? "bg-accent-yellow" : "bg-white/[0.05]"
          )}
        />
      ))}
    </div>
  );
}

function TaskRow({ task }: { task: BackendJobTask }) {
  return (
    <div className="flex items-start gap-3 rounded-lg border border-white/[0.08] bg-surface px-3 py-2.5">
      <ChevronRightIcon className="mt-0.5 h-3.5 w-3.5 shrink-0 text-gray-500" />
      <div className="min-w-0 flex-1 space-y-1">
        <div className="flex items-center justify-between gap-2">
          <span className="truncate text-sm font-semibold text-white">
            {task.title}
          </span>
          <span className="shrink-0 font-mono text-sm font-bold text-accent-teal">
            ◎ {task.price_sol.toFixed(2)}
          </span>
        </div>
        <p className="text-xs text-gray-400">{task.description}</p>
        <div className="flex items-center gap-2">
          <ComplexityDots level={task.complexity} />
          <span
            className={cn(
              "rounded px-1.5 py-0.5 text-xs font-semibold",
              COMPLEXITY_COLORS[task.complexity] ?? COMPLEXITY_COLORS[3]
            )}
          >
            {COMPLEXITY_LABEL[task.complexity] ?? `L${task.complexity}`}
          </span>
        </div>
      </div>
    </div>
  );
}

interface Props {
  estimate: BackendEstimateResponse;
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
    <div className="rounded-lg border border-white/[0.12] bg-surface-card p-5 space-y-5">
      {/* Header */}
      <div className="flex items-center gap-2">
        <SparklesIcon className="h-4 w-4 text-accent-teal" />
        <span className="text-sm font-semibold text-accent-teal">
          AI Price Estimate
        </span>
        <span
          className={cn(
            "ml-auto rounded px-2 py-0.5 text-xs font-semibold",
            COMPLEXITY_COLORS[estimate.overall_complexity] ?? COMPLEXITY_COLORS[3]
          )}
        >
          Complexity {estimate.overall_complexity}/5
        </span>
      </div>

      {/* Rationale */}
      <p className="text-sm italic text-gray-400">
        &ldquo;{estimate.rationale}&rdquo;
      </p>

      {/* Subtasks */}
      <div className="space-y-2">
        <p className="text-xs font-semibold uppercase tracking-widest text-gray-500">
          Subtasks breakdown
        </p>
        {estimate.tasks.map((task, i) => (
          <TaskRow key={i} task={task} />
        ))}
      </div>

      {/* Total */}
      <div className="flex items-center justify-between rounded-lg border border-white/[0.12] bg-surface-elevated px-4 py-3">
        <span className="text-sm text-gray-400">Suggested total</span>
        <span className="font-mono text-xl font-bold text-accent-teal">
          ◎ {estimate.total_price_sol.toFixed(2)}
        </span>
      </div>

      {/* Reward input */}
      <div className="space-y-2">
        <label className="text-xs font-semibold uppercase tracking-widest text-gray-500">
          Your reward (SOL)
        </label>
        <div className="flex items-center gap-2 rounded-lg border border-white/[0.08] bg-surface px-3 py-2">
          <span className="font-bold text-accent-teal">◎</span>
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
            Advance payment
          </label>
          <span className="font-mono text-xs text-accent-teal">
            {advancePct}%
          </span>
        </div>
        <input
          type="range"
          min={0}
          max={50}
          step={5}
          value={advancePct}
          onChange={(e) => onAdvanceChange(parseInt(e.target.value))}
          className="w-full accent-[#44bcc3]"
        />
        <div className="grid grid-cols-2 gap-2 text-xs">
          <div className="rounded-lg border border-white/[0.08] bg-accent-yellow/[0.08] px-3 py-2">
            <p className="text-accent-yellow mb-1 font-semibold">On claim</p>
            <span className="font-mono font-bold text-white">
              ◎ {advanceSol.toFixed(4)}
            </span>
          </div>
          <div className="rounded-lg border border-white/[0.08] bg-accent-teal/[0.08] px-3 py-2">
            <p className="text-accent-teal mb-1 font-semibold">On approval</p>
            <span className="font-mono font-bold text-white">
              ◎ {remainingSol.toFixed(4)}
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}
