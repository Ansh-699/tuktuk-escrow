use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::models::EscrowOffer;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct CreateEscrowFlow<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    pub mint_a: InterfaceAccount<'info, Mint>,
    pub mint_b: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init,
        payer = maker,
        seeds = [b"escrow", maker.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump,
        space = 8 + EscrowOffer::INIT_SPACE,
    )]
    pub escrow: Account<'info, EscrowOffer>,
    #[account(
        init,
        payer = maker,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateEscrowFlow<'info> {
    pub fn record_offer(&mut self, seed: u64, receive: u64, bumps: &CreateEscrowFlowBumps) -> Result<()> {
        self.escrow.set_inner(EscrowOffer {
            seed,
            maker: self.maker.key(),
            mint_a: self.mint_a.key(),
            mint_b: self.mint_b.key(),
            receive,
            created_at: Clock::get()?.unix_timestamp,
            bump: bumps.escrow,
        });
        Ok(())
    }

    pub fn lock_maker_funds(&mut self, deposit: u64) -> Result<()> {
        let transfer_accounts = TransferChecked {
            from: self.maker_ata_a.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.maker.to_account_info(),
            mint: self.mint_a.to_account_info(),
        };

        transfer_checked(
            CpiContext::new(self.token_program.to_account_info(), transfer_accounts),
            deposit,
            self.mint_a.decimals,
        )?;

        Ok(())
    }
}