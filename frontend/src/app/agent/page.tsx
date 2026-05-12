"use client";

import { useRouter } from "next/navigation";
import Link from "next/link";
import { useState, useMemo } from "react";
import {
  ZapIcon, RefreshCwIcon, Loader2Icon,
  PlusIcon, SearchIcon, TrendingUpIcon,
} from "lucide-react";
import { useAllOpenTasks, LiveTask } from "@/hooks/useAllOpenTasks";
import { useMarketplaceStats } from "@/hooks/useMarketplaceStats";
import { shortenPubkey } from "@/lib/pdas";
import { PublicKey } from "@solana/web3.js";

const GRADIENTS = [
  "from-violet-500 to-purple-700",
  "from-emerald-400 to-teal-600",
  "from-orange-400 to-red-600",
  "from-sky-400 to-blue-600",
  "from-pink-400 to-rose-600",
  "from-amber-400 to-orange-600",
  "from-cyan-400 to-blue-500",
  "from-lime-400 to-green-600",
];

function cardGradient(pubkey: PublicKey): string {
  const sum = pubkey.toBytes().slice(0, 4).reduce((a, b) => a + b, 0);
  return GRADIENTS[sum % GRADIENTS.length];
}

function TaskCard({ task, onClick }: { task: LiveTask; onClick: () => void }) {
  const gradient = cardGradient(task.pubkey);
  const advancePct = Math.round(task.advanceBps / 100);
  const advanceSol = ((task.reward * task.advanceBps) / 10_000).toFixed(3);
  const letter = task.pubkey.toBase58()[0].toUpperCase();

  return (
    <div
      onClick={onClick}
      className="group flex flex-col rounded-lg border border-white/[0.08] bg-surface-card hover:border-accent-teal/40 hover:bg-surface-card transition-all duration-200 cursor-pointer overflow-hidden"
    >
      {/* Gradient header */}
      <div className={`relative h-28 bg-gradient-to-br ${gradient}`}>
        <div className="absolute inset-0 bg-black/20" />
        <div className="absolute bottom-0 left-0 p-4">
          <div className="h-12 w-12 rounded-lg bg-black/30 backdrop-blur-sm border border-white/20 flex items-center justify-center">
            <span className="text-xl font-bold text-white">{letter}</span>
          </div>
        </div>
        <div className="absolute top-3 right-3">
          <span className="rounded-full bg-black/30 backdrop-blur-sm border border-white/20 px-2 py-0.5 text-xs font-semibold text-white">
            Open
          </span>
        </div>
      </div>

      {/* Body */}
      <div className="flex flex-col flex-1 p-4 gap-3">
        <div>
          <p className="font-mono text-[11px] text-gray-500">Task #{task.taskId.slice(-8)}</p>
          <p className="font-mono text-[11px] text-gray-600">{task.pubkey.toBase58().slice(0, 16)}…</p>
        </div>

        <div>
          <p className="font-mono text-2xl font-bold text-white">◎ {task.reward.toFixed(3)}</p>
          <p className="text-[11px] text-gray-500 mt-0.5">total reward</p>
        </div>

        <div className="grid grid-cols-2 gap-2">
          <div className="rounded-lg border border-white/10 bg-white/[0.02] px-3 py-2">
            <p className="text-[10px] text-gray-500 mb-0.5">Advance</p>
            <p className="font-mono text-xs font-bold text-white">
              {advancePct}% · ◎{advanceSol}
            </p>
          </div>
          <div className="rounded-lg border border-white/10 bg-white/[0.02] px-3 py-2">
            <p className="text-[10px] text-gray-500 mb-0.5">Stake req.</p>
            <p className="font-mono text-xs font-bold text-white">◎{task.requiredStake.toFixed(3)}</p>
          </div>
        </div>

        <div className="flex items-center justify-between mt-auto pt-1">
          <p className="text-[11px] text-gray-600">
            <span className="font-mono text-gray-400">{shortenPubkey(task.client)}</span>
          </p>
          <button
            onClick={(e) => { e.stopPropagation(); onClick(); }}
            className="flex items-center gap-1 rounded-lg bg-white px-3 py-1.5 text-xs font-semibold text-black hover:bg-neutral-200 transition-colors"
          >
            <ZapIcon className="h-3 w-3" /> Claim
          </button>
        </div>
      </div>
    </div>
  );
}

type SortKey = "reward" | "advance" | "stake";

export default function AgentPage() {
  const router = useRouter();
  const { tasks, loading, refresh } = useAllOpenTasks();
  const stats = useMarketplaceStats();
  const [search, setSearch] = useState("");
  const [sort, setSort] = useState<SortKey>("reward");

  const filtered = useMemo(() => {
    let list = [...tasks];
    if (search.trim()) {
      const q = search.toLowerCase();
      list = list.filter(
        (t) =>
          t.pubkey.toBase58().toLowerCase().includes(q) ||
          t.client.toBase58().toLowerCase().includes(q) ||
          t.taskId.includes(q)
      );
    }
    list.sort((a, b) => {
      if (sort === "reward") return b.reward - a.reward;
      if (sort === "advance") return b.advanceBps - a.advanceBps;
      return b.requiredStake - a.requiredStake;
    });
    return list;
  }, [tasks, search, sort]);

  return (
    <div className="min-h-screen">
      {/* Stats ticker */}
      <div className="border-b border-white/10 bg-white/[0.02] backdrop-blur-sm">
        <div className="mx-auto max-w-7xl px-4 py-2.5 flex items-center gap-8 overflow-x-auto text-sm">
          <div className="flex items-center gap-2 shrink-0">
            <TrendingUpIcon className="h-3.5 w-3.5 text-neutral-400" />
            <span className="text-gray-500">Tasks Posted</span>
            <span className="font-mono font-bold text-white">{stats?.totalTasks ?? "—"}</span>
          </div>
          <div className="flex items-center gap-2 shrink-0">
            <span className="h-2 w-2 rounded-full bg-accent-green animate-pulse" />
            <span className="text-gray-500">Open</span>
            <span className="font-mono font-bold text-white">{stats?.openTasks ?? "—"}</span>
          </div>
          <div className="flex items-center gap-2 shrink-0">
            <span className="text-gray-500">SOL in Escrow</span>
            <span className="font-mono font-bold text-white">◎ {stats?.tvlSol.toFixed(3) ?? "—"}</span>
          </div>
          <div className="ml-auto shrink-0 flex items-center gap-1.5 text-xs text-gray-600">
            <span className="h-1.5 w-1.5 rounded-full bg-accent-green animate-pulse" />
            Solana Devnet
          </div>
        </div>
      </div>

      <div className="mx-auto max-w-7xl px-4 py-8">
        {/* Page header */}
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-8">
          <div>
            <h1 className="text-2xl font-bold text-white">Task Marketplace</h1>
            <p className="text-sm text-gray-400 mt-1">
              Stake SOL to claim, earn upfront, settle on-chain.
            </p>
          </div>
          <Link
            href="/post"
            className="inline-flex items-center gap-2 rounded-lg bg-white px-5 py-2.5 text-sm font-semibold text-black hover:bg-neutral-200 transition-colors"
          >
            <PlusIcon className="h-4 w-4" /> Post a Task
          </Link>
        </div>

        {/* Search + sort bar */}
        <div className="flex flex-col sm:flex-row gap-3 mb-8">
          <div className="flex-1 flex items-center gap-2 rounded-lg border border-white/10 bg-white/[0.02] px-4 py-2.5">
            <SearchIcon className="h-4 w-4 text-gray-500 shrink-0" />
            <input
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="Search by address or task ID…"
              className="flex-1 bg-transparent text-sm text-white placeholder-gray-600 outline-none font-mono"
            />
          </div>
          <div className="flex items-center gap-2">
            <span className="text-xs text-gray-500 shrink-0">Sort:</span>
            {(["reward", "advance", "stake"] as SortKey[]).map((key) => (
              <button
                key={key}
                onClick={() => setSort(key)}
                className={`rounded-lg px-3 py-2 text-xs font-semibold capitalize transition-colors ${
                  sort === key
                    ? "bg-white text-black"
                    : "border border-white/10 bg-white/[0.02] text-gray-400 hover:text-white hover:bg-white/[0.05]"
                }`}
              >
                {key}
              </button>
            ))}
            <button
              onClick={refresh}
              title="Refresh"
              className="flex items-center rounded-lg border border-white/10 bg-white/[0.02] p-2 text-gray-400 hover:text-white hover:bg-white/[0.05] transition-colors"
            >
              <RefreshCwIcon className="h-3.5 w-3.5" />
            </button>
          </div>
        </div>

        {/* Content */}
        {loading ? (
          <div className="flex flex-col items-center justify-center py-40 gap-3 text-gray-500">
            <Loader2Icon className="h-8 w-8 animate-spin" />
            <p className="text-sm">Fetching on-chain tasks…</p>
          </div>
        ) : filtered.length === 0 ? (
          <div className="rounded-lg border border-white/10 bg-white/[0.02] py-40 text-center">
            <ZapIcon className="mx-auto h-10 w-10 text-neutral-600 mb-4" />
            <p className="text-gray-400 font-semibold text-lg">
              {search ? "No tasks match your search" : "No open tasks yet"}
            </p>
            <p className="text-sm text-gray-600 mt-2 mb-8">
              {search
                ? "Try a different search"
                : "Post the first task and lock a reward in escrow."}
            </p>
            {!search && (
              <Link
                href="/post"
                className="inline-flex items-center gap-2 rounded-lg bg-white px-6 py-3 text-sm font-semibold text-black hover:bg-neutral-200 transition-colors"
              >
                <PlusIcon className="h-4 w-4" /> Post a Task
              </Link>
            )}
          </div>
        ) : (
          <>
            <p className="text-xs text-gray-600 mb-4">{filtered.length} task{filtered.length !== 1 ? "s" : ""} found</p>
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
              {filtered.map((task) => (
                <TaskCard
                  key={task.pubkey.toBase58()}
                  task={task}
                  onClick={() => router.push(`/task/${task.pubkey.toBase58()}`)}
                />
              ))}
            </div>
          </>
        )}
      </div>
    </div>
  );
}
