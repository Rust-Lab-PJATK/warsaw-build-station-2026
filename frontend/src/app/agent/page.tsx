"use client";

// ─── UPDATE THESE after `anchor deploy` ──────────────────────────────────────
// Run: anchor deploy --provider.cluster devnet
// Then paste the task PDAs created by your demo script here.
const DEMO_TASK_PDAS: Array<{ pda: string; title: string; reward: string; advance: string; stake: string; complexity: "low" | "medium" | "high" }> = [
  // Example — replace with real devnet PDAs:
  // {
  //   pda: "AbCdEf...",
  //   title: "Build a Python script to scrape Solana DEX prices",
  //   reward: "0.8",
  //   advance: "20%",
  //   stake: "0.08",
  //   complexity: "medium",
  // },
];
// ─────────────────────────────────────────────────────────────────────────────

import { useRouter } from "next/navigation";
import Link from "next/link";
import { ArrowLeftIcon, ZapIcon, SearchIcon } from "lucide-react";
import { cn } from "@/lib/cn";

const COMPLEXITY_STYLES = {
  low: "text-accent-green bg-accent-green/10 border-accent-green/30",
  medium: "text-accent-yellow bg-accent-yellow/10 border-accent-yellow/30",
  high: "text-accent-red bg-accent-red/10 border-accent-red/30",
};

export default function AgentPage() {
  const router = useRouter();

  return (
    <div className="mx-auto max-w-3xl px-4 py-10">
      <Link
        href="/"
        className="mb-6 inline-flex items-center gap-2 text-sm text-gray-500 hover:text-white transition-colors"
      >
        <ArrowLeftIcon className="h-4 w-4" /> Back
      </Link>

      <div className="mb-8 flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">Open Tasks</h1>
          <p className="text-sm text-gray-400">
            Claim a task, receive an advance, deliver your preview.
          </p>
        </div>
        <span className="rounded-full border border-surface-elevated bg-surface-card px-3 py-1 text-sm text-gray-400">
          {DEMO_TASK_PDAS.length} open
        </span>
      </div>

      {DEMO_TASK_PDAS.length === 0 ? (
        <div className="rounded-xl border border-dashed border-surface-elevated py-20 text-center">
          <SearchIcon className="mx-auto h-8 w-8 text-gray-700 mb-3" />
          <p className="text-gray-500 font-semibold">No tasks yet</p>
          <p className="text-sm text-gray-600 mt-1 mb-6">
            Post the first task or add devnet PDAs to this page.
          </p>
          <Link
            href="/post"
            className="inline-flex items-center gap-2 rounded-xl bg-accent-purple px-5 py-2.5 text-sm font-semibold text-white hover:bg-accent-purple/80 transition-colors"
          >
            Post a Task
          </Link>
        </div>
      ) : (
        <div className="space-y-4">
          {DEMO_TASK_PDAS.map((t) => (
            <div
              key={t.pda}
              className="rounded-xl border border-surface-elevated bg-surface-card p-5 hover:border-accent-purple/30 transition-all cursor-pointer"
              onClick={() => router.push(`/task/${t.pda}`)}
            >
              <div className="flex items-start justify-between gap-4">
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 mb-2">
                    <span
                      className={cn(
                        "rounded-full border px-2 py-0.5 text-xs font-semibold capitalize",
                        COMPLEXITY_STYLES[t.complexity]
                      )}
                    >
                      {t.complexity}
                    </span>
                  </div>
                  <p className="font-semibold text-white leading-snug">{t.title}</p>
                  <p className="mt-1 font-mono text-xs text-gray-600 truncate">{t.pda}</p>
                </div>

                <div className="text-right shrink-0">
                  <p className="font-mono text-xl font-bold text-accent-green">◎ {t.reward}</p>
                  <p className="text-xs text-gray-500">{t.advance} advance</p>
                </div>
              </div>

              <div className="mt-4 flex items-center justify-between border-t border-surface-elevated pt-3">
                <span className="text-xs text-gray-500">
                  Required stake: <span className="font-mono text-gray-300">◎ {t.stake}</span>
                </span>
                <button
                  onClick={(e) => { e.stopPropagation(); router.push(`/task/${t.pda}`); }}
                  className="flex items-center gap-1.5 rounded-lg bg-accent-purple/10 border border-accent-purple/30 px-3 py-1.5 text-xs font-semibold text-accent-purple hover:bg-accent-purple/20 transition-colors"
                >
                  <ZapIcon className="h-3.5 w-3.5" /> Claim
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
