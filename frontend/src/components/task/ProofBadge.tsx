import { cn } from "@/lib/cn";
import { ShieldCheckIcon, ShieldXIcon, ShieldIcon } from "lucide-react";
import { bytesToHex } from "@/types/marketplace";

export type ProofState = "none" | "pending" | "verified" | "invalid";

interface Props {
  state: ProofState;
  resultHash: number[] | null;
}

const STATE_CONFIG: Record<
  ProofState,
  { label: string; sub: string; border: string; bg: string; text: string; icon: typeof ShieldCheckIcon }
> = {
  none: {
    label: "No Result",
    sub: "Agent has not submitted yet",
    border: "border-gray-700",
    bg: "bg-surface",
    text: "text-gray-500",
    icon: ShieldIcon,
  },
  pending: {
    label: "Proof Pending",
    sub: "ZK proof generation in progress…",
    border: "border-accent-yellow/40",
    bg: "bg-accent-yellow/5",
    text: "text-accent-yellow",
    icon: ShieldIcon,
  },
  verified: {
    label: "Proof Verified",
    sub: "ZK proof accepted on-chain",
    border: "border-accent-green/40",
    bg: "bg-accent-green/5",
    text: "text-accent-green",
    icon: ShieldCheckIcon,
  },
  invalid: {
    label: "Proof Invalid",
    sub: "ZK verification failed",
    border: "border-accent-red/40",
    bg: "bg-accent-red/5",
    text: "text-accent-red",
    icon: ShieldXIcon,
  },
};

export function ProofBadge({ state, resultHash }: Props) {
  const cfg = STATE_CONFIG[state];
  const Icon = cfg.icon;

  return (
    <div
      className={cn(
        "rounded-xl border p-5 space-y-3 transition-all",
        cfg.border,
        cfg.bg
      )}
    >
      <div className="flex items-center gap-3">
        <div
          className={cn(
            "flex h-10 w-10 items-center justify-center rounded-xl border",
            cfg.border
          )}
        >
          <Icon
            className={cn(
              "h-5 w-5",
              cfg.text,
              state === "pending" && "animate-pulse_slow"
            )}
          />
        </div>
        <div>
          <p className={cn("font-semibold", cfg.text)}>{cfg.label}</p>
          <p className="text-xs text-gray-500">{cfg.sub}</p>
        </div>

        {state === "verified" && (
          <span className="ml-auto rounded-full bg-accent-green/10 px-2 py-0.5 text-xs font-mono font-semibold text-accent-green">
            RISC ZERO
          </span>
        )}
      </div>

      {resultHash && resultHash.some((b) => b !== 0) && (
        <div className="rounded-lg bg-surface px-3 py-2">
          <p className="mb-1 text-[10px] uppercase tracking-widest text-gray-600">
            Result Hash (SHA-256)
          </p>
          <p className="break-all font-mono text-xs text-gray-300">
            0x{bytesToHex(resultHash)}
          </p>
        </div>
      )}
    </div>
  );
}
