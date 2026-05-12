"use client";

import { useState } from "react";
import { PublicKey } from "@solana/web3.js";
import { FileTextIcon, Loader2Icon, XIcon } from "lucide-react";
import { cn } from "@/lib/cn";
import { sha256Bytes } from "@/types/marketplace";
import { useSubmitResult } from "@/hooks/useMarketplace";
import { submitPreview, getJobId } from "@/lib/backendApi";

interface Props {
  taskPubkey: PublicKey;
  onSuccess?: () => void;
  onClose: () => void;
}

export function SubmitPreviewModal({ taskPubkey, onSuccess, onClose }: Props) {
  const [input, setInput] = useState("");
  const { submitResult, loading, error } = useSubmitResult();

  async function handleSubmit() {
    if (!input.trim()) return;
    const hash = await sha256Bytes(input.trim());
    const sig = await submitResult(taskPubkey, hash);
    if (sig) {
      // Notify backend that proof was submitted
      const jobId = getJobId(taskPubkey.toBase58());
      if (jobId) {
        try {
          await submitPreview(jobId);
        } catch (e) {
          console.warn("submitPreview backend call failed:", e);
        }
      }
      onSuccess?.();
      onClose();
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4">
      <div className="w-full max-w-md rounded-2xl border border-surface-elevated bg-surface-card p-6 space-y-5 shadow-2xl">
        {/* Header */}
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-2">
            <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-blue-500/10">
              <FileTextIcon className="h-5 w-5 text-blue-400" />
            </div>
            <div>
              <p className="font-semibold text-white">Submit Preview</p>
              <p className="text-xs text-gray-500">Share your work-in-progress with the client</p>
            </div>
          </div>
          <button onClick={onClose} className="text-gray-600 hover:text-white transition-colors">
            <XIcon className="h-5 w-5" />
          </button>
        </div>

        {/* Input */}
        <div className="space-y-2">
          <label className="text-xs font-semibold uppercase tracking-widest text-gray-500">
            Preview URL or IPFS CID
          </label>
          <textarea
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder="ipfs://Qm... or https://..."
            rows={3}
            className="w-full rounded-xl border border-surface-elevated bg-surface px-4 py-3 text-sm text-white placeholder-gray-600 outline-none focus:border-blue-500/50 transition-colors resize-none font-mono"
          />
          <p className="text-xs text-gray-600">
            This will be SHA-256 hashed and committed on-chain. The client can verify your preview matches this hash.
          </p>
        </div>

        {error && (
          <p className="rounded-lg bg-accent-red/10 border border-accent-red/30 px-3 py-2 text-sm text-accent-red">
            {error}
          </p>
        )}

        <div className="flex gap-3">
          <button
            onClick={onClose}
            className="flex-1 rounded-xl border border-surface-elevated py-3 text-sm text-gray-400 hover:text-white transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={handleSubmit}
            disabled={!input.trim() || loading}
            className={cn(
              "flex flex-1 items-center justify-center gap-2 rounded-xl py-3 text-sm font-semibold transition-all",
              input.trim() && !loading
                ? "bg-blue-500 text-white hover:bg-blue-400"
                : "bg-surface-elevated text-gray-600 cursor-not-allowed"
            )}
          >
            {loading ? <Loader2Icon className="h-4 w-4 animate-spin" /> : null}
            {loading ? "Submitting…" : "Submit to Chain"}
          </button>
        </div>
      </div>
    </div>
  );
}
