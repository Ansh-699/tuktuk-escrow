#![allow(unexpected_cfgs)]
#![allow(deprecated)]

use anchor_lang::prelude::*;
mod faults;
mod flows;
mod models;

use flows::*;
declare_id!("8PFxoRZxSAmtbq2fUw3ok9Agtcs89Zywbihtsz4wSUtL");

#[program]
pub mod tuktuk_escrow {
    use super::*;
    pub fn make(ctx: Context<CreateEscrowFlow>, seed: u64, deposit: u64, receive: u64) -> Result<()> {
        ctx.accounts.record_offer(seed, receive, &ctx.bumps)?;
        ctx.accounts.lock_maker_funds(deposit)
    }

    pub fn refund(ctx: Context<CancelEscrowFlow>) -> Result<()> {
        ctx.accounts.release_to_maker()
    }

    pub fn take(ctx: Context<SettleEscrowFlow>) -> Result<()> {
        ctx.accounts.enforce_unlock_time()?;
        ctx.accounts.pay_maker_side()?;
        ctx.accounts.release_to_taker()
    }

    pub fn auto_refund(ctx: Context<CrankCancelEscrowFlow>) -> Result<()> {
        ctx.accounts.release_to_maker()
    }

    pub fn schedule(ctx: Context<QueueCancelEscrowFlow>, task_id: u16) -> Result<()> {
        ctx.accounts.enqueue_cancel_task(task_id, &ctx.bumps)
    }
}
