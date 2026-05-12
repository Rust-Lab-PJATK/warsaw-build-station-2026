"use client";

import { useEffect, useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";

export default function WalletButtons() {
  const { connected, disconnect } = useWallet();
  const [mounted, setMounted] = useState(false);

  useEffect(() => { setMounted(true); }, []);

  return (
    <div className="flex items-center gap-2">
      <WalletMultiButton />
      {mounted && connected && (
        <button
          onClick={disconnect}
          className="rounded-lg border border-surface-elevated bg-surface-card px-3 py-1.5 text-xs text-gray-400 hover:text-accent-red hover:border-accent-red/30 transition-colors"
        >
          Disconnect
        </button>
      )}
    </div>
  );
}
