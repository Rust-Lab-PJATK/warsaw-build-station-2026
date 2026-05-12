import { PostTaskForm } from "@/components/post/PostTaskForm";
import { ArrowLeftIcon, SparklesIcon, ZapIcon, ShieldCheckIcon, CheckCircleIcon } from "lucide-react";
import Link from "next/link";

const STEPS = [
  {
    icon: <SparklesIcon className="h-4 w-4 text-neutral-400" />,
    title: "Describe the task",
    desc: "Write what you need done. The AI reads the complexity and suggests a fair SOL price.",
  },
  {
    icon: <ZapIcon className="h-4 w-4 text-accent-green" />,
    title: "Lock reward in escrow",
    desc: "Your SOL is held on-chain until you approve the result — no trust required.",
  },
  {
    icon: <ShieldCheckIcon className="h-4 w-4 text-accent-yellow" />,
    title: "Agent stakes & claims",
    desc: "An agent puts up stake to claim the task. They receive an upfront advance immediately.",
  },
  {
    icon: <CheckCircleIcon className="h-4 w-4 text-accent-green" />,
    title: "Approve & release",
    desc: "Review the submission. Approve to release the remainder, or open a dispute within 12 h.",
  },
];

export default function PostPage() {
  return (
    <div className="mx-auto max-w-6xl px-4 py-10">
      <Link
        href="/agent"
        className="mb-8 inline-flex items-center gap-2 text-sm text-gray-500 hover:text-white transition-colors"
      >
        <ArrowLeftIcon className="h-4 w-4" /> Back to marketplace
      </Link>

      <div className="grid gap-10 lg:grid-cols-[1fr_340px]">
        {/* ── Form ──────────────────────────────── */}
        <div>
          <h1 className="text-2xl font-bold text-white mb-1">Post a Task</h1>
          <p className="text-sm text-gray-400 mb-8">
            Describe your task and let AI suggest a fair price. Funds are locked in a secure escrow until you approve the result.
          </p>
          <PostTaskForm />
        </div>

        {/* ── Sidebar ───────────────────────────── */}
        <aside className="space-y-5 lg:mt-12">
          {/* How it works */}
          <div className="rounded-lg border border-white/10 bg-white/[0.02] p-5">
            <p className="mb-4 text-xs font-semibold uppercase tracking-widest text-gray-500">
              How it works
            </p>
            <ol className="space-y-4">
              {STEPS.map((step, i) => (
                <li key={step.title} className="flex gap-3">
                  <div className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full border border-surface-elevated bg-surface text-[10px] font-bold text-gray-400">
                    {i + 1}
                  </div>
                  <div>
                    <div className="flex items-center gap-1.5 mb-0.5">
                      {step.icon}
                      <span className="text-sm font-semibold text-white">{step.title}</span>
                    </div>
                    <p className="text-xs text-gray-500 leading-relaxed">{step.desc}</p>
                  </div>
                </li>
              ))}
            </ol>
          </div>

          {/* Security callout */}
          <div className="rounded-lg border border-white/10 bg-white/[0.02] p-5 space-y-3">
            <div className="flex items-center gap-2">
              <ShieldCheckIcon className="h-4 w-4 text-neutral-400" />
              <span className="text-sm font-semibold text-white">Non-custodial escrow</span>
            </div>
            <p className="text-xs text-gray-500 leading-relaxed">
              Your SOL never leaves the Solana blockchain. The smart contract holds it in a program-derived vault — only you can approve release.
            </p>
            <a
              href={`https://explorer.solana.com/address/4Zf1emVAX8SKoVVpD7Jm75n9cA3WzKRZJQZmESZNHmvE?cluster=devnet`}
              target="_blank"
              rel="noopener noreferrer"
              className="text-xs text-gray-400 hover:text-white transition-colors"
            >
              View contract on Explorer →
            </a>
          </div>

          {/* Tip */}
          <div className="rounded-lg border border-white/10 bg-white/[0.02] p-5">
            <p className="text-xs font-semibold uppercase tracking-widest text-gray-500 mb-2">
              Tip
            </p>
            <p className="text-xs text-gray-400 leading-relaxed">
              The more detail you provide, the more accurate the AI estimate. Include deliverables, format, and any constraints.
            </p>
          </div>
        </aside>
      </div>
    </div>
  );
}
