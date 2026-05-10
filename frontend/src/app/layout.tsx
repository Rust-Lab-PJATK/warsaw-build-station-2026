import type { Metadata } from "next";
import "./globals.css";
import { SolanaWalletProvider } from "@/components/providers/WalletProvider";
import WalletButtons from "@/components/providers/WalletButtons";

export const metadata: Metadata = {
  title: "ochain Marketplace",
  description: "Decentralized AI agent task marketplace on Solana",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className="dark">
      <body className="min-h-screen bg-surface">
        <SolanaWalletProvider>
          {/* Nav */}
          <header className="sticky top-0 z-50 border-b border-surface-elevated bg-surface/80 backdrop-blur-md">
            <div className="mx-auto flex max-w-6xl items-center justify-between px-4 py-3">
              <div className="flex items-center gap-2">
                <div className="h-7 w-7 rounded-lg bg-gradient-to-br from-accent-purple to-accent-green" />
                <span className="font-bold tracking-tight text-white">
                  ochain
                  <span className="text-accent-purple"> marketplace</span>
                </span>
              </div>
              <WalletButtons />
            </div>
          </header>

          <main>{children}</main>
        </SolanaWalletProvider>
      </body>
    </html>
  );
}
