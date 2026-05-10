import { cn } from "@/lib/cn";
import { TaskStatusVariant, STATUS_LABELS } from "@/types/marketplace";

const VARIANT_STYLES: Record<TaskStatusVariant, string> = {
  open: "bg-accent-green/10 text-accent-green border border-accent-green/30",
  claimed: "bg-accent-purple/10 text-accent-purple border border-accent-purple/30",
  submitted: "bg-blue-500/10 text-blue-400 border border-blue-500/30",
  approved: "bg-accent-green/20 text-accent-green border border-accent-green/50",
  disputed: "bg-accent-yellow/10 text-accent-yellow border border-accent-yellow/30",
  slashed: "bg-accent-red/10 text-accent-red border border-accent-red/30",
};

const DOT_STYLES: Record<TaskStatusVariant, string> = {
  open: "bg-accent-green animate-pulse",
  claimed: "bg-accent-purple animate-pulse",
  submitted: "bg-blue-400",
  approved: "bg-accent-green",
  disputed: "bg-accent-yellow animate-pulse",
  slashed: "bg-accent-red",
};

interface Props {
  variant: TaskStatusVariant;
  size?: "sm" | "md";
}

export function StatusBadge({ variant, size = "md" }: Props) {
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1.5 rounded-full font-mono font-semibold tracking-wide",
        size === "sm" ? "px-2 py-0.5 text-xs" : "px-3 py-1 text-sm",
        VARIANT_STYLES[variant]
      )}
    >
      <span className={cn("h-1.5 w-1.5 rounded-full", DOT_STYLES[variant])} />
      {STATUS_LABELS[variant].toUpperCase()}
    </span>
  );
}
