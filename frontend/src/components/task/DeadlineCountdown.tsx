"use client";

import { useCountdown } from "@/hooks/useCountdown";
import { cn } from "@/lib/cn";
import { ClockIcon } from "lucide-react";

interface Props {
  label: string;
  deadlineUnix: number;
}

function Segment({
  value,
  unit,
  urgent,
}: {
  value: number;
  unit: string;
  urgent: boolean;
}) {
  return (
    <div className="flex flex-col items-center">
      <span
        className={cn(
          "font-mono text-3xl font-bold tabular-nums leading-none",
          urgent ? "text-accent-red" : "text-white"
        )}
      >
        {String(value).padStart(2, "0")}
      </span>
      <span className="mt-1 text-[10px] uppercase tracking-widest text-gray-500">
        {unit}
      </span>
    </div>
  );
}

function Colon({ urgent }: { urgent: boolean }) {
  return (
    <span
      className={cn(
        "mb-3 font-mono text-2xl font-bold leading-none",
        urgent ? "text-accent-red/70" : "text-gray-600"
      )}
    >
      :
    </span>
  );
}

export function DeadlineCountdown({ label, deadlineUnix }: Props) {
  const { hours, minutes, seconds, expired, urgent } =
    useCountdown(deadlineUnix);

  return (
    <div
      className={cn(
        "rounded-xl border bg-surface-card p-5 space-y-4 transition-colors",
        urgent
          ? "border-accent-red/40"
          : expired
          ? "border-gray-700"
          : "border-surface-elevated"
      )}
    >
      <div className="flex items-center gap-2">
        <div
          className={cn(
            "flex h-8 w-8 items-center justify-center rounded-lg",
            urgent ? "bg-accent-red/10" : "bg-surface-elevated"
          )}
        >
          <ClockIcon
            className={cn(
              "h-4 w-4",
              urgent ? "text-accent-red" : "text-gray-400"
            )}
          />
        </div>
        <div>
          <p className="text-sm font-semibold text-white">{label}</p>
          <p className="text-xs text-gray-500">
            {expired ? "Window closed" : urgent ? "Expiring soon!" : "Remaining"}
          </p>
        </div>
      </div>

      {expired ? (
        <div className="flex items-center justify-center rounded-lg bg-accent-red/10 py-4">
          <span className="font-mono text-lg font-bold text-accent-red">
            EXPIRED
          </span>
        </div>
      ) : (
        <div className="flex items-center justify-center gap-2">
          <Segment value={hours} unit="hrs" urgent={urgent} />
          <Colon urgent={urgent} />
          <Segment value={minutes} unit="min" urgent={urgent} />
          <Colon urgent={urgent} />
          <Segment value={seconds} unit="sec" urgent={urgent} />
        </div>
      )}
    </div>
  );
}
