"use client";

import { useConnection } from "@solana/wallet-adapter-react";
import { PublicKey } from "@solana/web3.js";
import { BorshAccountsCoder } from "@coral-xyz/anchor";
import { useEffect, useState, useCallback } from "react";
import { PROGRAM_ID } from "@/lib/pdas";
// eslint-disable-next-line @typescript-eslint/no-explicit-any
import IDL from "@/target/idl/ochain_marketplace.json";

const LAMPORTS_PER_SOL = 1_000_000_000;
const TASK_ACCOUNT_SIZE = 192;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const CODER = new BorshAccountsCoder(IDL as any);

export interface LiveTask {
  pubkey: PublicKey;
  reward: number;
  requiredStake: number;
  advanceBps: number;
  client: PublicKey;
  taskId: string;
}

export function useAllOpenTasks() {
  const { connection } = useConnection();
  const [tasks, setTasks] = useState<LiveTask[]>([]);
  const [loading, setLoading] = useState(true);

  const fetchTasks = useCallback(async () => {
    try {
      const accounts = await connection.getProgramAccounts(PROGRAM_ID, {
        filters: [{ dataSize: TASK_ACCOUNT_SIZE }],
      });

      const open: LiveTask[] = [];
      for (const { pubkey, account } of accounts) {
        try {
          const data = CODER.decode("taskAccount", account.data);
          if (!("open" in data.status)) continue;
          open.push({
            pubkey,
            reward: data.reward.toNumber() / LAMPORTS_PER_SOL,
            requiredStake: data.requiredStake.toNumber() / LAMPORTS_PER_SOL,
            advanceBps: data.advanceBps as number,
            client: data.client as PublicKey,
            taskId: data.taskId.toString(),
          });
        } catch {
          // skip malformed accounts
        }
      }

      setTasks(open);
    } catch (e) {
      console.error("Failed to fetch tasks:", e);
    } finally {
      setLoading(false);
    }
  }, [connection]);

  useEffect(() => {
    fetchTasks();
    const id = setInterval(fetchTasks, 15_000);
    return () => clearInterval(id);
  }, [fetchTasks]);

  return { tasks, loading, refresh: fetchTasks };
}
