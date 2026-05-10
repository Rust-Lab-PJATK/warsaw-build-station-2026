"use client";

import { useConnection, useAnchorWallet, useWallet } from "@solana/wallet-adapter-react";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { BN } from "@coral-xyz/anchor";
import { useCallback, useState } from "react";
import { getProgram } from "@/lib/program";
import { findTaskPda, findVaultPda, findStakePda } from "@/lib/pdas";

interface TxState {
  loading: boolean;
  error: string | null;
  signature: string | null;
}

function useTxState() {
  const [state, setState] = useState<TxState>({
    loading: false,
    error: null,
    signature: null,
  });
  const start = () => setState({ loading: true, error: null, signature: null });
  const ok = (sig: string) =>
    setState({ loading: false, error: null, signature: sig });
  const fail = (e: unknown) =>
    setState({
      loading: false,
      error: e instanceof Error ? e.message : "Transaction failed",
      signature: null,
    });
  return { state, start, ok, fail };
}

export function usePostTask() {
  const { connection } = useConnection();
  const wallet = useAnchorWallet();
  const { publicKey } = useWallet();
  const { state, start, ok, fail } = useTxState();

  const postTask = useCallback(
    async (
      taskId: BN,
      rewardLamports: BN,
      requiredStakeLamports: BN,
      descriptionHash: number[],
      advanceBps: number
    ) => {
      if (!wallet || !publicKey) return;
      start();
      try {
        const program = getProgram(connection, wallet);
        const [task] = findTaskPda(publicKey, taskId);
        const [vault] = findVaultPda(task);

        const sig = await program.methods
          .postTask(
            taskId,
            rewardLamports,
            requiredStakeLamports,
            descriptionHash,
            advanceBps
          )
          .accounts({
            client: publicKey,
            task,
            vault,
            systemProgram: SystemProgram.programId,
          })
          .rpc();
        ok(sig);
        return { sig, taskPubkey: task };
      } catch (e) {
        fail(e);
      }
    },
    [connection, wallet, publicKey]
  );

  return { postTask, ...state };
}

export function useClaimTask() {
  const { connection } = useConnection();
  const wallet = useAnchorWallet();
  const { publicKey } = useWallet();
  const { state, start, ok, fail } = useTxState();

  const claimTask = useCallback(
    async (taskPubkey: PublicKey) => {
      if (!wallet || !publicKey) return;
      start();
      try {
        const program = getProgram(connection, wallet);
        const [vault] = findVaultPda(taskPubkey);
        const [stakeVault] = findStakePda(taskPubkey, publicKey);

        const sig = await program.methods
          .claimTask()
          .accounts({
            agent: publicKey,
            task: taskPubkey,
            vault,
            stakeVault,
            systemProgram: SystemProgram.programId,
          })
          .rpc();
        ok(sig);
        return sig;
      } catch (e) {
        fail(e);
      }
    },
    [connection, wallet, publicKey]
  );

  return { claimTask, ...state };
}

export function useSubmitResult() {
  const { connection } = useConnection();
  const wallet = useAnchorWallet();
  const { publicKey } = useWallet();
  const { state, start, ok, fail } = useTxState();

  const submitResult = useCallback(
    async (taskPubkey: PublicKey, resultHash: number[]) => {
      if (!wallet || !publicKey) return;
      start();
      try {
        const program = getProgram(connection, wallet);

        const sig = await program.methods
          .submitResult(resultHash)
          .accounts({ agent: publicKey, task: taskPubkey })
          .rpc();
        ok(sig);
        return sig;
      } catch (e) {
        fail(e);
      }
    },
    [connection, wallet, publicKey]
  );

  return { submitResult, ...state };
}

export function useApproveResult() {
  const { connection } = useConnection();
  const wallet = useAnchorWallet();
  const { publicKey } = useWallet();
  const { state, start, ok, fail } = useTxState();

  const approveResult = useCallback(
    async (taskPubkey: PublicKey, agentPubkey: PublicKey) => {
      if (!wallet || !publicKey) return;
      start();
      try {
        const program = getProgram(connection, wallet);
        const [vault] = findVaultPda(taskPubkey);
        const [stakeVault] = findStakePda(taskPubkey, agentPubkey);

        const sig = await program.methods
          .approveResult()
          .accounts({
            client: publicKey,
            agent: agentPubkey,
            task: taskPubkey,
            vault,
            stakeVault,
            systemProgram: SystemProgram.programId,
          })
          .rpc();
        ok(sig);
        return sig;
      } catch (e) {
        fail(e);
      }
    },
    [connection, wallet, publicKey]
  );

  return { approveResult, ...state };
}

export function useDisputeResult() {
  const { connection } = useConnection();
  const wallet = useAnchorWallet();
  const { publicKey } = useWallet();
  const { state, start, ok, fail } = useTxState();

  const disputeResult = useCallback(
    async (taskPubkey: PublicKey) => {
      if (!wallet || !publicKey) return;
      start();
      try {
        const program = getProgram(connection, wallet);

        const sig = await program.methods
          .disputeResult()
          .accounts({ client: publicKey, task: taskPubkey })
          .rpc();
        ok(sig);
        return sig;
      } catch (e) {
        fail(e);
      }
    },
    [connection, wallet, publicKey]
  );

  return { disputeResult, ...state };
}

export function useSlashTimeout() {
  const { connection } = useConnection();
  const wallet = useAnchorWallet();
  const { publicKey } = useWallet();
  const { state, start, ok, fail } = useTxState();

  const slashTimeout = useCallback(
    async (
      taskPubkey: PublicKey,
      clientPubkey: PublicKey,
      agentPubkey: PublicKey
    ) => {
      if (!wallet || !publicKey) return;
      start();
      try {
        const program = getProgram(connection, wallet);
        const [vault] = findVaultPda(taskPubkey);
        const [stakeVault] = findStakePda(taskPubkey, agentPubkey);

        const sig = await program.methods
          .slashTimeout()
          .accounts({
            client: clientPubkey,
            agent: agentPubkey,
            task: taskPubkey,
            vault,
            stakeVault,
            systemProgram: SystemProgram.programId,
          })
          .rpc();
        ok(sig);
        return sig;
      } catch (e) {
        fail(e);
      }
    },
    [connection, wallet, publicKey]
  );

  return { slashTimeout, ...state };
}
