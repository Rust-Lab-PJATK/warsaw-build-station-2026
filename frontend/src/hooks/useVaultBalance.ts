"use client";

import { useConnection } from "@solana/wallet-adapter-react";
import { PublicKey } from "@solana/web3.js";
import { useEffect, useState } from "react";
import { findVaultPda, findStakePda } from "@/lib/pdas";

export function useVaultBalance(taskPubkey: PublicKey | null): number | null {
  const { connection } = useConnection();
  const [balance, setBalance] = useState<number | null>(null);

  useEffect(() => {
    if (!taskPubkey) return;
    const [vault] = findVaultPda(taskPubkey);

    connection.getBalance(vault).then(setBalance).catch(() => setBalance(null));

    const id = connection.onAccountChange(vault, (info) =>
      setBalance(info.lamports)
    );
    return () => { connection.removeAccountChangeListener(id); };
  }, [connection, taskPubkey?.toBase58()]);

  return balance;
}

export function useStakeBalance(
  taskPubkey: PublicKey | null,
  agentPubkey: PublicKey | null
): number | null {
  const { connection } = useConnection();
  const [balance, setBalance] = useState<number | null>(null);

  useEffect(() => {
    if (!taskPubkey || !agentPubkey) return;
    const [stake] = findStakePda(taskPubkey, agentPubkey);

    connection.getBalance(stake).then(setBalance).catch(() => setBalance(null));

    const id = connection.onAccountChange(stake, (info) =>
      setBalance(info.lamports)
    );
    return () => { connection.removeAccountChangeListener(id); };
  }, [connection, taskPubkey?.toBase58(), agentPubkey?.toBase58()]);

  return balance;
}
