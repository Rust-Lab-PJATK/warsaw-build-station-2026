import { TaskAccount, lamportsToSol, advanceLamports, remainingLamports } from "@/types/marketplace";
import { ZapIcon, CheckCircleIcon, Check } from "lucide-react";

interface Props {
  task: TaskAccount;
}

export function AdvanceBreakdown({ task }: Props) {
  const total = lamportsToSol(task.reward);
  const advance = lamportsToSol(advanceLamports(task));
  const remaining = lamportsToSol(remainingLamports(task));
  const advancePct = task.advanceBps / 100;

  // Bar fill width for the advance portion
  const fillPct = Math.min(task.advanceBps / 50, 100); // max 50% = full bar

  return (
    <div className="rounded-xl border border-surface-elevated bg-surface-card p-5 space-y-4">
      <p className="text-xs font-semibold uppercase tracking-widest text-gray-500">
        Payment Schedule
      </p>

      {/* Progress bar */}
      <div className="h-2 w-full rounded-full bg-surface overflow-hidden">
        <div
          className="h-full rounded-full bg-gradient-to-r from-accent-purple to-accent-green transition-all"
          style={{ width: `${fillPct}%` }}
        />
      </div>

      <div className="grid grid-cols-2 gap-3">
        {/* Advance */}
        <div className="rounded-lg bg-accent-purple/10 border border-accent-purple/20 px-4 py-3">
          <div className="flex items-center gap-1.5 text-accent-purple mb-2">
            <ZapIcon className="h-3.5 w-3.5" />
            <span className="text-xs font-semibold">On Claim ({advancePct}%)</span>
          </div>
          <p className="font-mono text-xl font-bold text-white">◎ {advance.toFixed(4)}</p>
          <p className="text-[10px] text-gray-500 mt-1">
            {task.advancePaid.toNumber() > 0 ? (
              <span className="inline-flex items-center gap-1"><Check className="h-3 w-3 text-accent-green" />Paid</span>
            ) : (
              "Paid immediately on claim"
            )}
          </p>
        </div>

        {/* Remaining */}
        <div className="rounded-lg bg-accent-green/10 border border-accent-green/20 px-4 py-3">
          <div className="flex items-center gap-1.5 text-accent-green mb-2">
            <CheckCircleIcon className="h-3.5 w-3.5" />
            <span className="text-xs font-semibold">On Approval ({(100 - advancePct)}%)</span>
          </div>
          <p className="font-mono text-xl font-bold text-white">◎ {remaining.toFixed(4)}</p>
          <p className="text-[10px] text-gray-500 mt-1">Released after preview approved</p>
        </div>
      </div>

      <div className="flex items-center justify-between border-t border-surface-elevated pt-3">
        <span className="text-sm text-gray-500">Total reward</span>
        <span className="font-mono text-lg font-bold text-white">◎ {total.toFixed(4)}</span>
      </div>
    </div>
  );
}
