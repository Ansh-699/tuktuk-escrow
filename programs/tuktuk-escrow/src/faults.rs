use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowFault {
    #[msg("Minimum time has not passed after make")]
    EscrowStillLocked,
}