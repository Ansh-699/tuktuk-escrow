use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use tuktuk_program::{
    compile_transaction,
    tuktuk::{
        cpi::{accounts::QueueTaskV0, queue_task_v0},
        program::Tuktuk,
        types::TriggerV0,
    },
    types::QueueTaskArgsV0,
    TransactionSourceV0,
};

use crate::models::EscrowOffer;

#[derive(Accounts)]
pub struct QueueCancelEscrowFlow<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    mint_a: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
    )]
    maker_ata_a: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        close = maker,
        has_one = mint_a,
        has_one = maker,
        seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, EscrowOffer>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
    )]
    vault: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    /// CHECK: Validated by the tuktuk program during `queue_task_v0` CPI.
    pub task_queue: UncheckedAccount<'info>,
    /// CHECK: Validated by the tuktuk program during `queue_task_v0` CPI.
    pub task_queue_authority: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: Created/validated by the tuktuk program during `queue_task_v0` CPI.
    pub task: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"queue_authority"],
        bump
    )]
    /// CHECK: PDA is constrained by seeds+bump and only used as a CPI signer.
    pub queue_authority: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub tuktuk_program: Program<'info, Tuktuk>,
}

impl<'info> QueueCancelEscrowFlow<'info> {
    pub fn enqueue_cancel_task(&mut self, task_id: u16, bumps: &QueueCancelEscrowFlowBumps) -> Result<()> {
        let (compiled_tx, _) = compile_transaction(
            vec![Instruction {
                program_id: crate::ID,
                accounts: vec![
                    AccountMeta::new(self.maker.key(), false),
                    AccountMeta::new_readonly(self.mint_a.key(), false),
                    AccountMeta::new(self.maker_ata_a.key(), false),
                    AccountMeta::new(self.escrow.key(), false),
                    AccountMeta::new(self.vault.key(), false),
                    AccountMeta::new_readonly(self.token_program.key(), false),
                    AccountMeta::new_readonly(self.system_program.key(), false),
                ],
                data: anchor_lang::solana_program::hash::hash(b"global:auto_refund").to_bytes()[..8].to_vec(),
            }],
            vec![],
        )
        .unwrap();

        queue_task_v0(
            CpiContext::new_with_signer(
                self.tuktuk_program.to_account_info(),
                QueueTaskV0 {
                    payer: self.maker.to_account_info(),
                    queue_authority: self.queue_authority.to_account_info(),
                    task_queue_authority: self.task_queue_authority.to_account_info(),
                    task_queue: self.task_queue.to_account_info(),
                    task: self.task.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                },
                &[&[b"queue_authority", &[bumps.queue_authority]]],
            ),
            QueueTaskArgsV0 {
                id: task_id,
                trigger: TriggerV0::Now,
                transaction: TransactionSourceV0::CompiledV0(compiled_tx),
                crank_reward: Some(1_000_001),
                free_tasks: 0,
                description: "refund_escrow".to_string(),
            },
        )?;

        Ok(())
    }
}