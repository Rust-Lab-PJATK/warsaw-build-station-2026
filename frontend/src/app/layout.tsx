import type { Metadata } from "next";
import Link from "next/link";
import "./globals.css";
import { SolanaWalletProvider } from "@/components/providers/WalletProvider";
import WalletButtons from "@/components/providers/WalletButtons";
import { AmbientBackground } from "@/components/AmbientBackground";

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
        <AmbientBackground />
        <SolanaWalletProvider>
          {/* Nav */}
          <header className="sticky top-0 z-50 border-b border-surface-elevated bg-surface/80 backdrop-blur-md">
            <div className="mx-auto flex max-w-6xl items-center justify-between px-4 py-3">
              <div className="flex items-center gap-6">
                <Link href="/" className="flex items-center gap-2 shrink-0">
                  <div className="h-7 w-7 rounded-lg bg-gradient-to-br from-accent-teal to-accent-green" />
                  <span className="font-bold tracking-tight text-white">nexwork</span>
                </Link>
                <nav className="hidden sm:flex items-center gap-1 text-sm">
                  <Link href="/agent" className="rounded-lg px-3 py-1.5 text-gray-400 hover:text-white hover:bg-surface-elevated transition-colors">Browse Tasks</Link>
                  <Link href="/post" className="rounded-lg px-3 py-1.5 text-gray-400 hover:text-white hover:bg-surface-elevated transition-colors">Post Task</Link>
                </nav>
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
