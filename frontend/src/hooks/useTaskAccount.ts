"use client";

import { useConnection, useAnchorWallet } from "@solana/wallet-adapter-react";
import { PublicKey } from "@solana/web3.js";
import { useCallback, useEffect, useState } from "react";
import { getProgram } from "@/lib/program";
import { TaskAccount } from "@/types/marketplace";

interface UseTaskAccountResult {
  task: TaskAccount | null;
  loading: boolean;
  error: string | null;
  refresh: () => void;
}

export function useTaskAccount(
  taskPubkey: PublicKey | null
): UseTaskAccountResult {
  const { connection } = useConnection();
  const wallet = useAnchorWallet();
  const [task, setTask] = useState<TaskAccount | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetch = useCallback(async () => {
    if (!taskPubkey || !wallet) return;
    setLoading(true);
    setError(null);
    try {
      const program = getProgram(connection, wallet);
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const raw = await (program.account as any).taskAccount.fetch(taskPubkey);
      setTask(raw as TaskAccount);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to fetch task");
    } finally {
      setLoading(false);
    }
  }, [connection, wallet, taskPubkey?.toBase58()]);

  useEffect(() => {
    fetch();
  }, [fetch]);

  // Subscribe to account changes for live updates
  useEffect(() => {
    if (!taskPubkey || !wallet) return;
    const id = connection.onAccountChange(taskPubkey, () => fetch());
    return () => { connection.removeAccountChangeListener(id); };
  }, [connection, taskPubkey?.toBase58(), wallet]);

  return { task, loading, error, refresh: fetch };
}
