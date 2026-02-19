use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace, Debug)]
pub struct EscrowOffer {
    pub seed: u64,
    pub maker: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub receive: u64,
    pub created_at: i64,
    pub bump: u8,
}

#[constant]
pub const OFFER_LOCK_WINDOW: i64 = 5 * 24 * 60 * 60;