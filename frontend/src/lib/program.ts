import { AnchorProvider, Program, Idl } from "@coral-xyz/anchor";
import { Connection, PublicKey } from "@solana/web3.js";
import { AnchorWallet } from "@solana/wallet-adapter-react";
import { PROGRAM_ID } from "./pdas";

// Copy target/idl/ochain_marketplace.json here after `anchor build`.
// eslint-disable-next-line @typescript-eslint/no-explicit-any
import IDL from "../target/idl/ochain_marketplace.json";

export type OchainMarketplace = any;

export function getProgram(
  connection: Connection,
  wallet: AnchorWallet
): Program<any> {
  const provider = new AnchorProvider(connection, wallet, {
    commitment: "confirmed",
  });
  return new Program(IDL as unknown as Idl, PROGRAM_ID, provider) as unknown as Program<any>;
}

export const RPC_ENDPOINT =
  process.env.NEXT_PUBLIC_RPC_URL ?? "https://api.devnet.solana.com";
