"use client";

import { cn } from "@/lib/cn";
import { SparklesIcon, ChevronRightIcon } from "lucide-react";
import type { BackendEstimateResponse, BackendJobTask } from "@/lib/backendApi";

export type { BackendEstimateResponse as PriceEstimate };

const COMPLEXITY_COLORS: Record<number, string> = {
  1: "text-[var(--accent-primary)] bg-[var(--accent-primary-dim)]",
  2: "text-[var(--accent-primary)] bg-[var(--accent-primary-dim)]",
  3: "text-[var(--accent-amber)] bg-[var(--accent-amber-dim)]",
  4: "text-[var(--accent-amber)] bg-[var(--accent-amber-dim)]",
  5: "text-[var(--accent-red)] bg-[var(--accent-red-dim)]",
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
            i < level ? "bg-[var(--accent-amber)]" : "bg-[var(--bg-overlay)]"
          )}
        />
      ))}
    </div>
  );
}

function TaskRow({ task }: { task: BackendJobTask }) {
  return (
    <div className="flex items-start gap-3 rounded-[var(--radius-md)] border border-[var(--border-subtle)] bg-[var(--bg-base)] px-3 py-2.5">
      <ChevronRightIcon className="mt-0.5 h-3.5 w-3.5 shrink-0 text-[var(--text-muted)]" />
      <div className="min-w-0 flex-1 space-y-1">
        <div className="flex items-center justify-between gap-2">
          <span className="truncate text-sm font-semibold text-[var(--text-primary)]">
            {task.title}
          </span>
          <span className="shrink-0 font-mono text-sm font-bold text-[var(--accent-primary)]">
            ◎ {task.price_sol.toFixed(2)}
          </span>
        </div>
        <p className="text-xs text-[var(--text-secondary)]">{task.description}</p>
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
    <div className="rounded-[var(--radius-lg)] border border-[var(--border-default)] bg-[var(--bg-surface)] p-5 space-y-5">
      {/* Header */}
      <div className="flex items-center gap-2">
        <SparklesIcon className="h-4 w-4 text-[var(--accent-primary)]" />
        <span className="text-sm font-semibold text-[var(--accent-primary)]">
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
      <p className="text-sm italic text-[var(--text-secondary)]">
        &ldquo;{estimate.rationale}&rdquo;
      </p>

      {/* Subtasks */}
      <div className="space-y-2">
        <p className="text-xs font-semibold uppercase tracking-widest text-[var(--text-muted)]">
          Subtasks breakdown
        </p>
        {estimate.tasks.map((task, i) => (
          <TaskRow key={i} task={task} />
        ))}
      </div>

      {/* Total */}
      <div className="flex items-center justify-between rounded-[var(--radius-md)] border border-[var(--border-default)] bg-[var(--bg-elevated)] px-4 py-3">
        <span className="text-sm text-[var(--text-secondary)]">Suggested total</span>
        <span className="font-mono text-xl font-bold text-[var(--accent-primary)]">
          ◎ {estimate.total_price_sol.toFixed(2)}
        </span>
      </div>

      {/* Reward input */}
      <div className="space-y-2">
        <label className="text-xs font-semibold uppercase tracking-widest text-[var(--text-muted)]">
          Your reward (SOL)
        </label>
        <div className="flex items-center gap-2 rounded-[var(--radius-md)] border border-[var(--border-subtle)] bg-[var(--bg-base)] px-3 py-2">
          <span className="font-bold text-[var(--accent-primary)]">◎</span>
          <input
            type="number"
            value={rewardSol}
            onChange={(e) => onRewardChange(e.target.value)}
            step="0.01"
            min="0.01"
            className="flex-1 bg-transparent font-mono text-lg text-[var(--text-primary)] outline-none"
          />
        </div>
      </div>

      {/* Advance slider */}
      <div className="space-y-2">
        <div className="flex justify-between">
          <label className="text-xs font-semibold uppercase tracking-widest text-[var(--text-muted)]">
            Advance payment
          </label>
          <span className="font-mono text-xs text-[var(--accent-primary)]">
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
          className="w-full accent-[var(--accent-primary)]"
        />
        <div className="grid grid-cols-2 gap-2 text-xs">
          <div className="rounded-[var(--radius-md)] border border-[var(--border-subtle)] bg-[var(--accent-amber-dim)] px-3 py-2">
            <p className="text-[var(--accent-amber)] mb-1 font-semibold">On claim</p>
            <span className="font-mono font-bold text-[var(--text-primary)]">
              ◎ {advanceSol.toFixed(4)}
            </span>
          </div>
          <div className="rounded-[var(--radius-md)] border border-[var(--border-subtle)] bg-[var(--accent-primary-dim)] px-3 py-2">
            <p className="text-[var(--accent-primary)] mb-1 font-semibold">On approval</p>
            <span className="font-mono font-bold text-[var(--text-primary)]">
              ◎ {remainingSol.toFixed(4)}
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}
