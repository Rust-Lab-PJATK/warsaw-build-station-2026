"use client";

import { useConnection } from "@solana/wallet-adapter-react";
import { BorshAccountsCoder } from "@coral-xyz/anchor";
import { useEffect, useState } from "react";
import { PROGRAM_ID } from "@/lib/pdas";
// eslint-disable-next-line @typescript-eslint/no-explicit-any
import IDL from "@/target/idl/ochain_marketplace.json";

const LAMPORTS_PER_SOL = 1_000_000_000;
const TASK_ACCOUNT_SIZE = 192;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const CODER = new BorshAccountsCoder(IDL as any);

export interface MarketplaceStats {
  totalTasks: number;
  openTasks: number;
  tvlSol: number;
}

export function useMarketplaceStats() {
  const { connection } = useConnection();
  const [stats, setStats] = useState<MarketplaceStats | null>(null);

  useEffect(() => {
    async function load() {
      try {
        const accounts = await connection.getProgramAccounts(PROGRAM_ID, {
          filters: [{ dataSize: TASK_ACCOUNT_SIZE }],
        });

        let totalTasks = 0;
        let openTasks = 0;
        let tvlLamports = 0;

        for (const { account } of accounts) {
          try {
            const data = CODER.decode("taskAccount", account.data);
            totalTasks++;
            const isOpen = "open" in data.status;
            const isClaimed = "claimed" in data.status;
            const isSubmitted = "submitted" in data.status;

            if (isOpen) {
              openTasks++;
              tvlLamports += data.reward.toNumber();
            } else if (isClaimed || isSubmitted) {
              tvlLamports += data.reward.toNumber() - data.advancePaid.toNumber();
            }
          } catch {
            // skip malformed
          }
        }

        setStats({
          totalTasks,
          openTasks,
          tvlSol: tvlLamports / LAMPORTS_PER_SOL,
        });
      } catch (e) {
        console.error("Stats fetch failed:", e);
      }
    }

    load();
    const id = setInterval(load, 30_000);
    return () => clearInterval(id);
  }, [connection]);

  return stats;
}
