"use client";

import { useEffect, useState } from "react";

export interface CountdownResult {
  remaining: number; // seconds
  hours: number;
  minutes: number;
  seconds: number;
  expired: boolean;
  urgent: boolean; // < 1 h remaining
}

export function useCountdown(deadlineUnix: number): CountdownResult {
  const [remaining, setRemaining] = useState(0);

  useEffect(() => {
    const tick = () =>
      setRemaining(
        Math.max(0, deadlineUnix - Math.floor(Date.now() / 1000))
      );
    tick();
    const id = setInterval(tick, 1000);
    return () => clearInterval(id);
  }, [deadlineUnix]);

  const hours = Math.floor(remaining / 3600);
  const minutes = Math.floor((remaining % 3600) / 60);
  const seconds = remaining % 60;

  return {
    remaining,
    hours,
    minutes,
    seconds,
    expired: remaining === 0,
    urgent: remaining > 0 && remaining < 3600,
  };
}
