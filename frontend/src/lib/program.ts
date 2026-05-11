import { AnchorProvider, Program, Idl } from "@coral-xyz/anchor";
import { Connection } from "@solana/web3.js";
import { AnchorWallet } from "@solana/wallet-adapter-react";

// Inline IDL derived from contracts/ochain-marketplace/src/lib.rs.
// Replace with the output of `anchor build` (target/idl/ochain_marketplace.json)
// once the contract is compiled.
//
// Discriminators are sha256("global:<snake_case_name>")[0..8].
// Values below are placeholders — replace with real ones after `anchor build`.
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const IDL: any = {
  address: "MktpLaCE111111111111111111111111111111111111",
  metadata: { name: "ochain_marketplace", version: "0.1.0", spec: "0.1.0" },
  instructions: [
    {
      name: "postTask",
      discriminator: [0, 0, 0, 0, 0, 0, 0, 0],
      accounts: [
        { name: "client", writable: true, signer: true },
        { name: "task", writable: true, signer: false },
        { name: "vault", writable: true, signer: false },
        { name: "systemProgram", writable: false, signer: false },
      ],
      args: [
        { name: "taskId", type: "u64" },
        { name: "reward", type: "u64" },
        { name: "requiredStake", type: "u64" },
        { name: "descriptionHash", type: { array: ["u8", 32] } },
        { name: "advanceBps", type: "u16" },
      ],
    },
    {
      name: "claimTask",
      discriminator: [0, 0, 0, 0, 0, 0, 0, 0],
      accounts: [
        { name: "agent", writable: true, signer: true },
        { name: "task", writable: true, signer: false },
        { name: "vault", writable: true, signer: false },
        { name: "stakeVault", writable: true, signer: false },
        { name: "systemProgram", writable: false, signer: false },
      ],
      args: [],
    },
    {
      name: "submitResult",
      discriminator: [0, 0, 0, 0, 0, 0, 0, 0],
      accounts: [
        { name: "agent", writable: false, signer: true },
        { name: "task", writable: true, signer: false },
      ],
      args: [{ name: "resultHash", type: { array: ["u8", 32] } }],
    },
    {
      name: "approveResult",
      discriminator: [0, 0, 0, 0, 0, 0, 0, 0],
      accounts: [
        { name: "client", writable: false, signer: true },
        { name: "agent", writable: true, signer: false },
        { name: "task", writable: true, signer: false },
        { name: "vault", writable: true, signer: false },
        { name: "stakeVault", writable: true, signer: false },
        { name: "systemProgram", writable: false, signer: false },
      ],
      args: [],
    },
    {
      name: "disputeResult",
      discriminator: [0, 0, 0, 0, 0, 0, 0, 0],
      accounts: [
        { name: "client", writable: false, signer: true },
        { name: "task", writable: true, signer: false },
      ],
      args: [],
    },
    {
      name: "slashTimeout",
      discriminator: [0, 0, 0, 0, 0, 0, 0, 0],
      accounts: [
        { name: "client", writable: true, signer: false },
        { name: "agent", writable: false, signer: false },
        { name: "task", writable: true, signer: false },
        { name: "vault", writable: true, signer: false },
        { name: "stakeVault", writable: true, signer: false },
        { name: "systemProgram", writable: false, signer: false },
      ],
      args: [],
    },
  ],
  accounts: [{ name: "TaskAccount", discriminator: [0, 0, 0, 0, 0, 0, 0, 0] }],
  types: [
    {
      name: "TaskStatus",
      type: {
        kind: "enum",
        variants: [
          { name: "Open" },
          { name: "Claimed" },
          { name: "Submitted" },
          { name: "Approved" },
          { name: "Disputed" },
          { name: "Slashed" },
        ],
      },
    },
    {
      name: "TaskAccount",
      type: {
        kind: "struct",
        fields: [
          { name: "client", type: "pubkey" },
          { name: "agent", type: { option: "pubkey" } },
          { name: "taskId", type: "u64" },
          { name: "reward", type: "u64" },
          { name: "requiredStake", type: "u64" },
          { name: "descriptionHash", type: { array: ["u8", 32] } },
          { name: "resultHash", type: { option: { array: ["u8", 32] } } },
          { name: "status", type: { defined: { name: "TaskStatus" } } },
          { name: "submitDeadline", type: "i64" },
          { name: "disputeDeadline", type: "i64" },
          { name: "advanceBps", type: "u16" },
          { name: "advancePaid", type: "u64" },
          { name: "taskBump", type: "u8" },
          { name: "vaultBump", type: "u8" },
          { name: "stakeBump", type: "u8" },
        ],
      },
    },
  ],
};

export type OchainMarketplace = any;

export function getProgram(
  connection: Connection,
  wallet: AnchorWallet
// eslint-disable-next-line @typescript-eslint/no-explicit-any
): Program<any> {
  const provider = new AnchorProvider(connection, wallet, {
    commitment: "confirmed",
  });
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  return new Program(IDL as Idl, provider) as unknown as Program<any>;
}

export const RPC_ENDPOINT =
  process.env.NEXT_PUBLIC_RPC_URL ?? "https://api.devnet.solana.com";
