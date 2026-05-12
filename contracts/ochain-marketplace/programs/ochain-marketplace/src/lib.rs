use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke_signed, system_instruction};

declare_id!("4Zf1emVAX8SKoVVpD7Jm75n9cA3WzKRZJQZmESZNHmvE");

// ─── Seeds ────────────────────────────────────────────────────────────────────

pub const TASK_SEED: &[u8] = b"task";
pub const VAULT_SEED: &[u8] = b"vault";
pub const STAKE_SEED: &[u8] = b"stake";

// ─── Timing ───────────────────────────────────────────────────────────────────

/// Agent must submit within 24 h of claiming.
pub const SUBMIT_WINDOW_SECS: i64 = 86_400;
/// Client may dispute within 12 h of a submission.
pub const DISPUTE_WINDOW_SECS: i64 = 43_200;
/// Extra 24 h after dispute_deadline before a permissionless slash is valid.
pub const SLASH_GRACE_SECS: i64 = 86_400;

// ─── Program ──────────────────────────────────────────────────────────────────

#[program]
pub mod ochain_marketplace {
    use super::*;

    /// Client posts a task and locks `reward` lamports into the vault PDA.
    ///
    /// PDAs created here:
    ///   task  = ["task",  client, task_id_le]
    ///   vault = ["vault", task]          ← system-owned lamport vault
    pub fn post_task(
        ctx: Context<PostTask>,
        task_id: u64,
        reward: u64,
        required_stake: u64,
        description_hash: [u8; 32],
        // Basis points of reward released immediately to agent on claim (0–5000).
        // e.g. 2000 = 20% advance, 80% held until approve_result.
        advance_bps: u16,
    ) -> Result<()> {
        require!(reward > 0, MarketplaceError::ZeroReward);
        require!(required_stake > 0, MarketplaceError::ZeroStake);
        require!(advance_bps <= 5_000, MarketplaceError::AdvanceBpsTooHigh);
        let advance_amount = (reward as u128)
            .checked_mul(advance_bps as u128)
            .ok_or(MarketplaceError::Overflow)?
            .checked_div(10_000)
            .ok_or(MarketplaceError::Overflow)? as u64;
        require!(advance_amount <= required_stake, MarketplaceError::AdvanceExceedsStake);

        let task = &mut ctx.accounts.task;
        task.client = ctx.accounts.client.key();
        task.agent = None;
        task.task_id = task_id;
        task.reward = reward;
        task.required_stake = required_stake;
        task.description_hash = description_hash;
        task.result_hash = None;
        task.status = TaskStatus::Open;
        task.submit_deadline = 0;
        task.dispute_deadline = 0;
        task.advance_bps = advance_bps;
        task.advance_paid = 0;
        task.task_bump = ctx.bumps.task;
        task.vault_bump = ctx.bumps.vault;
        task.stake_bump = 0; // set by claim_task

        // Deposit reward into the system-owned vault PDA.
        // vault is uninitialized (no data); system_program::transfer works because
        // it is still owned by the System Program at this point.
        anchor_lang::system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.client.to_account_info(),
                    to: ctx.accounts.vault.to_account_info(),
                },
            ),
            reward,
        )?;

        emit!(TaskPosted {
            task: ctx.accounts.task.key(),
            client: ctx.accounts.client.key(),
            task_id,
            reward,
            required_stake,
            advance_bps,
            description_hash,
        });

        Ok(())
    }

    /// Agent claims an open task and locks `required_stake` lamports into the
    /// stake vault PDA.  Starts the submit clock.
    ///
    /// PDA created here:
    ///   stake_vault = ["stake", task, agent]   ← system-owned lamport vault
    pub fn claim_task(ctx: Context<ClaimTask>) -> Result<()> {
        require!(
            ctx.accounts.task.status == TaskStatus::Open,
            MarketplaceError::TaskNotOpen
        );

        let now = Clock::get()?.unix_timestamp;
        let submit_deadline = now
            .checked_add(SUBMIT_WINDOW_SECS)
            .ok_or(MarketplaceError::Overflow)?;

        // Read before any mutable borrow.
        let required_stake = ctx.accounts.task.required_stake;
        let advance_bps = ctx.accounts.task.advance_bps;
        let reward = ctx.accounts.task.reward;
        let vault_bump = ctx.accounts.task.vault_bump;
        let task_key = ctx.accounts.task.key();
        let agent_key = ctx.accounts.agent.key();
        let stake_bump = ctx.bumps.stake_vault;

        // Lock agent stake into stake vault.
        anchor_lang::system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.agent.to_account_info(),
                    to: ctx.accounts.stake_vault.to_account_info(),
                },
            ),
            required_stake,
        )?;

        // Release advance payment from vault → agent immediately.
        let advance = reward
            .checked_mul(advance_bps as u64)
            .ok_or(MarketplaceError::Overflow)?
            .checked_div(10_000)
            .ok_or(MarketplaceError::Overflow)?;

        if advance > 0 {
            invoke_signed(
                &system_instruction::transfer(ctx.accounts.vault.key, &agent_key, advance),
                &[
                    ctx.accounts.vault.to_account_info(),
                    ctx.accounts.agent.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
                &[&[VAULT_SEED, task_key.as_ref(), &[vault_bump]]],
            )?;
        }

        let task = &mut ctx.accounts.task;
        task.agent = Some(agent_key);
        task.status = TaskStatus::Claimed;
        task.submit_deadline = submit_deadline;
        task.stake_bump = stake_bump;
        task.advance_paid = advance;

        emit!(TaskClaimed {
            task: task_key,
            agent: agent_key,
            advance_paid: advance,
            submit_deadline,
        });

        Ok(())
    }

    /// Agent submits the SHA-256 hash of their result artifact (e.g. IPFS CID).
    /// Must be called before `submit_deadline`.
    pub fn submit_result(ctx: Context<SubmitResult>, result_hash: [u8; 32]) -> Result<()> {
        require!(
            ctx.accounts.task.status == TaskStatus::Claimed,
            MarketplaceError::InvalidStatus
        );
        require!(
            ctx.accounts.task.agent == Some(ctx.accounts.agent.key()),
            MarketplaceError::Unauthorized
        );

        let now = Clock::get()?.unix_timestamp;
        require!(
            now <= ctx.accounts.task.submit_deadline,
            MarketplaceError::DeadlineExpired
        );

        let dispute_deadline = now
            .checked_add(DISPUTE_WINDOW_SECS)
            .ok_or(MarketplaceError::Overflow)?;
        let task_key = ctx.accounts.task.key();
        let agent_key = ctx.accounts.agent.key();

        let task = &mut ctx.accounts.task;
        task.result_hash = Some(result_hash);
        task.status = TaskStatus::Submitted;
        task.dispute_deadline = dispute_deadline;

        emit!(ResultSubmitted {
            task: task_key,
            agent: agent_key,
            result_hash,
            dispute_deadline,
        });

        Ok(())
    }

    /// Client accepts the result.  Releases reward and stake back to the agent.
    pub fn approve_result(ctx: Context<ApproveResult>) -> Result<()> {
        require!(
            ctx.accounts.task.status == TaskStatus::Submitted,
            MarketplaceError::InvalidStatus
        );
        require_keys_eq!(
            ctx.accounts.client.key(),
            ctx.accounts.task.client,
            MarketplaceError::Unauthorized
        );
        require!(
            ctx.accounts.task.agent == Some(ctx.accounts.agent.key()),
            MarketplaceError::AgentMismatch
        );

        let task_key = ctx.accounts.task.key();
        let reward = ctx.accounts.task.reward;
        let advance_paid = ctx.accounts.task.advance_paid;
        let required_stake = ctx.accounts.task.required_stake;
        let vault_bump = ctx.accounts.task.vault_bump;
        let stake_bump = ctx.accounts.task.stake_bump;
        let agent_key = ctx.accounts.agent.key();

        // Vault only holds the remaining reward (advance already released on claim).
        let remaining_reward = reward
            .checked_sub(advance_paid)
            .ok_or(MarketplaceError::Overflow)?;

        ctx.accounts.task.status = TaskStatus::Approved;

        // vault → agent  (remaining reward)
        invoke_signed(
            &system_instruction::transfer(ctx.accounts.vault.key, &agent_key, remaining_reward),
            &[
                ctx.accounts.vault.to_account_info(),
                ctx.accounts.agent.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[&[VAULT_SEED, task_key.as_ref(), &[vault_bump]]],
        )?;

        // stake_vault → agent  (stake refund)
        invoke_signed(
            &system_instruction::transfer(
                ctx.accounts.stake_vault.key,
                &agent_key,
                required_stake,
            ),
            &[
                ctx.accounts.stake_vault.to_account_info(),
                ctx.accounts.agent.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[&[STAKE_SEED, task_key.as_ref(), agent_key.as_ref(), &[stake_bump]]],
        )?;

        emit!(ResultApproved {
            task: task_key,
            agent: agent_key,
            advance_paid,
            remaining_reward,
        });

        Ok(())
    }

    /// Client opens a dispute within the dispute window after submission.
    pub fn dispute_result(ctx: Context<DisputeResult>) -> Result<()> {
        require!(
            ctx.accounts.task.status == TaskStatus::Submitted,
            MarketplaceError::InvalidStatus
        );
        require_keys_eq!(
            ctx.accounts.client.key(),
            ctx.accounts.task.client,
            MarketplaceError::Unauthorized
        );

        let now = Clock::get()?.unix_timestamp;
        require!(
            now <= ctx.accounts.task.dispute_deadline,
            MarketplaceError::DisputeWindowClosed
        );

        let task_key = ctx.accounts.task.key();
        ctx.accounts.task.status = TaskStatus::Disputed;

        emit!(DisputeOpened {
            task: task_key,
            client: ctx.accounts.client.key(),
            disputed_at: now,
        });

        Ok(())
    }

    /// Permissionless: slashes the agent once a deadline has been missed.
    ///
    /// Two cases trigger a slash:
    ///   1. Status == Claimed  AND  now > submit_deadline   (agent never submitted)
    ///   2. Status == Disputed AND  now > dispute_deadline + SLASH_GRACE_SECS
    ///                                                       (dispute unresolved)
    ///
    /// Both reward and stake are transferred to the original client.
    pub fn slash_timeout(ctx: Context<SlashTimeout>) -> Result<()> {
        require!(
            ctx.accounts.task.agent == Some(ctx.accounts.agent.key()),
            MarketplaceError::AgentMismatch
        );

        let now = Clock::get()?.unix_timestamp;

        let can_slash = match ctx.accounts.task.status {
            TaskStatus::Claimed => now > ctx.accounts.task.submit_deadline,
            TaskStatus::Disputed => {
                let slash_after = ctx
                    .accounts
                    .task
                    .dispute_deadline
                    .checked_add(SLASH_GRACE_SECS)
                    .ok_or(MarketplaceError::Overflow)?;
                now > slash_after
            }
            _ => false,
        };
        require!(can_slash, MarketplaceError::SlashNotReady);

        let task_key = ctx.accounts.task.key();
        let reward = ctx.accounts.task.reward;
        let advance_paid = ctx.accounts.task.advance_paid;
        let required_stake = ctx.accounts.task.required_stake;
        let vault_bump = ctx.accounts.task.vault_bump;
        let stake_bump = ctx.accounts.task.stake_bump;
        let agent_key = ctx.accounts.agent.key();
        let client_key = ctx.accounts.client.key();

        // Client only recovers what remains in the vault after the advance.
        let vault_remaining = reward
            .checked_sub(advance_paid)
            .ok_or(MarketplaceError::Overflow)?;

        ctx.accounts.task.status = TaskStatus::Slashed;

        // vault → client  (remaining reward refund)
        invoke_signed(
            &system_instruction::transfer(ctx.accounts.vault.key, &client_key, vault_remaining),
            &[
                ctx.accounts.vault.to_account_info(),
                ctx.accounts.client.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[&[VAULT_SEED, task_key.as_ref(), &[vault_bump]]],
        )?;

        // stake_vault → client  (slash)
        invoke_signed(
            &system_instruction::transfer(
                ctx.accounts.stake_vault.key,
                &client_key,
                required_stake,
            ),
            &[
                ctx.accounts.stake_vault.to_account_info(),
                ctx.accounts.client.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[&[STAKE_SEED, task_key.as_ref(), agent_key.as_ref(), &[stake_bump]]],
        )?;

        emit!(AgentSlashed {
            task: task_key,
            agent: agent_key,
            slashed_amount: required_stake,
        });

        Ok(())
    }
}

// ─── Account Contexts ─────────────────────────────────────────────────────────

#[derive(Accounts)]
#[instruction(task_id: u64)]
pub struct PostTask<'info> {
    #[account(mut)]
    pub client: Signer<'info>,

    #[account(
        init,
        payer = client,
        space = 8 + TaskAccount::INIT_SPACE,
        seeds = [TASK_SEED, client.key().as_ref(), &task_id.to_le_bytes()],
        bump,
    )]
    pub task: Account<'info, TaskAccount>,

    /// CHECK: system-owned vault PDA that holds the reward lamports.
    /// Seeds are [VAULT_SEED, task].  The account has no data; it stays
    /// System-Program-owned so subsequent invoke_signed transfers work.
    #[account(
        mut,
        seeds = [VAULT_SEED, task.key().as_ref()],
        bump,
    )]
    pub vault: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimTask<'info> {
    #[account(mut)]
    pub agent: Signer<'info>,

    #[account(mut)]
    pub task: Account<'info, TaskAccount>,

    /// CHECK: reward vault PDA; advance payment is released from here.
    /// Seeds verified + bump loaded from task to avoid re-deriving.
    #[account(
        mut,
        seeds = [VAULT_SEED, task.key().as_ref()],
        bump = task.vault_bump,
    )]
    pub vault: UncheckedAccount<'info>,

    /// CHECK: system-owned stake vault PDA.  Seeds [STAKE_SEED, task, agent].
    /// Receives the agent's stake via system_program::transfer.
    #[account(
        mut,
        seeds = [STAKE_SEED, task.key().as_ref(), agent.key().as_ref()],
        bump,
    )]
    pub stake_vault: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SubmitResult<'info> {
    pub agent: Signer<'info>,

    #[account(mut)]
    pub task: Account<'info, TaskAccount>,
}

#[derive(Accounts)]
pub struct ApproveResult<'info> {
    pub client: Signer<'info>,

    /// CHECK: agent receiving the reward.  Key is verified against task.agent
    /// in the instruction body, and it is the seed used to derive stake_vault.
    #[account(mut)]
    pub agent: UncheckedAccount<'info>,

    #[account(mut)]
    pub task: Account<'info, TaskAccount>,

    /// CHECK: reward vault PDA verified by seeds + stored bump.
    #[account(
        mut,
        seeds = [VAULT_SEED, task.key().as_ref()],
        bump = task.vault_bump,
    )]
    pub vault: UncheckedAccount<'info>,

    /// CHECK: stake vault PDA verified by seeds + stored bump.
    #[account(
        mut,
        seeds = [STAKE_SEED, task.key().as_ref(), agent.key().as_ref()],
        bump = task.stake_bump,
    )]
    pub stake_vault: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DisputeResult<'info> {
    pub client: Signer<'info>,

    #[account(mut)]
    pub task: Account<'info, TaskAccount>,
}

#[derive(Accounts)]
pub struct SlashTimeout<'info> {
    /// CHECK: must be the original client recorded in task.client.
    /// Checked via constraint so no signer required (permissionless slash).
    #[account(
        mut,
        constraint = client.key() == task.client @ MarketplaceError::Unauthorized,
    )]
    pub client: UncheckedAccount<'info>,

    /// CHECK: agent being slashed.  Key verified against task.agent in body
    /// and used as seed for stake_vault.
    pub agent: UncheckedAccount<'info>,

    #[account(mut)]
    pub task: Account<'info, TaskAccount>,

    /// CHECK: reward vault PDA verified by seeds + stored bump.
    #[account(
        mut,
        seeds = [VAULT_SEED, task.key().as_ref()],
        bump = task.vault_bump,
    )]
    pub vault: UncheckedAccount<'info>,

    /// CHECK: stake vault PDA verified by seeds + stored bump.
    #[account(
        mut,
        seeds = [STAKE_SEED, task.key().as_ref(), agent.key().as_ref()],
        bump = task.stake_bump,
    )]
    pub stake_vault: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

// ─── State ────────────────────────────────────────────────────────────────────

#[account]
#[derive(InitSpace)]
pub struct TaskAccount {
    /// Wallet that posted the task and funded the vault.
    pub client: Pubkey,
    /// Set to Some(agent) when the task is claimed.
    pub agent: Option<Pubkey>,
    /// Caller-chosen ID, unique per client (used in PDA seed).
    pub task_id: u64,
    /// Total reward lamports locked in vault at post time.
    pub reward: u64,
    /// Lamports the agent must stake to claim.
    pub required_stake: u64,
    /// Basis points of reward released to agent immediately on claim (0–5000).
    pub advance_bps: u16,
    /// Lamports already paid to agent as advance (set in claim_task).
    pub advance_paid: u64,
    /// SHA-256 or IPFS CIDv1 of the task specification (off-chain content).
    pub description_hash: [u8; 32],
    /// SHA-256 / CID of the preview or final result; None until submit_result.
    pub result_hash: Option<[u8; 32]>,
    pub status: TaskStatus,
    /// Unix timestamp by which agent must submit (set in claim_task).
    pub submit_deadline: i64,
    /// Unix timestamp by which client may dispute (set in submit_result).
    pub dispute_deadline: i64,
    pub task_bump: u8,
    pub vault_bump: u8,
    pub stake_bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum TaskStatus {
    Open,
    Claimed,
    Submitted,
    Approved,
    Disputed,
    Slashed,
}

// ─── Events ───────────────────────────────────────────────────────────────────

#[event]
pub struct TaskPosted {
    pub task: Pubkey,
    pub client: Pubkey,
    pub task_id: u64,
    pub reward: u64,
    pub required_stake: u64,
    pub advance_bps: u16,
    pub description_hash: [u8; 32],
}

#[event]
pub struct TaskClaimed {
    pub task: Pubkey,
    pub agent: Pubkey,
    pub advance_paid: u64,
    pub submit_deadline: i64,
}

#[event]
pub struct ResultSubmitted {
    pub task: Pubkey,
    pub agent: Pubkey,
    pub result_hash: [u8; 32],
    pub dispute_deadline: i64,
}

#[event]
pub struct ResultApproved {
    pub task: Pubkey,
    pub agent: Pubkey,
    pub advance_paid: u64,
    pub remaining_reward: u64,
}

#[event]
pub struct DisputeOpened {
    pub task: Pubkey,
    pub client: Pubkey,
    pub disputed_at: i64,
}

#[event]
pub struct AgentSlashed {
    pub task: Pubkey,
    pub agent: Pubkey,
    pub slashed_amount: u64,
}

// ─── Errors ───────────────────────────────────────────────────────────────────

#[error_code]
pub enum MarketplaceError {
    #[msg("Reward must be greater than zero")]
    ZeroReward,
    #[msg("Required stake must be greater than zero")]
    ZeroStake,
    #[msg("Task is not open for claiming")]
    TaskNotOpen,
    #[msg("Task is not in the expected status for this instruction")]
    InvalidStatus,
    #[msg("Caller is not authorized for this operation")]
    Unauthorized,
    #[msg("Provided agent does not match the agent recorded on the task")]
    AgentMismatch,
    #[msg("Submit deadline has passed")]
    DeadlineExpired,
    #[msg("Dispute window has closed")]
    DisputeWindowClosed,
    #[msg("Slash conditions have not been met yet")]
    SlashNotReady,
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("Advance basis points cannot exceed 50% (5000 bps)")]
    AdvanceBpsTooHigh,
    #[msg("Advance payment would exceed required stake")]
    AdvanceExceedsStake,
}
