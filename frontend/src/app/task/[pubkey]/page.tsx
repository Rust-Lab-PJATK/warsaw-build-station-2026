"use client";

import { useMemo } from "react";
import { PublicKey } from "@solana/web3.js";
import { TaskDetail } from "@/components/task/TaskDetail";
import { ArrowLeftIcon } from "lucide-react";
import Link from "next/link";

interface Props {
  params: { pubkey: string };
}

export default function TaskPage({ params }: Props) {
  const pubkey = useMemo(() => {
    try {
      return new PublicKey(params.pubkey);
    } catch {
      return null;
    }
  }, [params.pubkey]);

  if (!pubkey) {
    return (
      <div className="mx-auto max-w-2xl px-4 py-20 text-center">
        <p className="text-accent-red font-semibold">Invalid task address</p>
        <Link
          href="/"
          className="mt-4 inline-flex items-center gap-2 text-sm text-gray-400 hover:text-white transition-colors"
        >
          <ArrowLeftIcon className="h-4 w-4" />
          Back to marketplace
        </Link>
      </div>
    );
  }

  return (
    <>
      <div className="mx-auto max-w-5xl px-4 pt-6">
        <Link
          href="/"
          className="inline-flex items-center gap-2 text-sm text-gray-500 hover:text-white transition-colors"
        >
          <ArrowLeftIcon className="h-4 w-4" />
          All tasks
        </Link>
      </div>
      <TaskDetail taskPubkey={pubkey} />
    </>
  );
}
