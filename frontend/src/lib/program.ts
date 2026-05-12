import { AnchorProvider, Program, Idl } from "@coral-xyz/anchor";
import { Connection } from "@solana/web3.js";
import { AnchorWallet } from "@solana/wallet-adapter-react";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
import IDL from "./idl/ochain_marketplace.json";

export type OchainMarketplace = any;

export function getProgram(
  connection: Connection,
  wallet: AnchorWallet
): Program<any> {
  const provider = new AnchorProvider(connection, wallet, {
    commitment: "confirmed",
  });
  // Anchor 0.30: program ID is taken from IDL address field
  return new Program(IDL as unknown as Idl, provider) as unknown as Program<any>;
}

export const RPC_ENDPOINT =
  process.env.NEXT_PUBLIC_RPC_URL ?? "https://api.devnet.solana.com";
