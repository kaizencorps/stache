use anchor_lang::prelude::*;

#[error_code]
pub enum StacheError {
    #[msg("Not authorized")]
    NotAuthorized,
    #[msg("Invalid Stache ID")]
    InvalidStacheId,
    #[msg("Invalid name. Must be lowercase, no spaces, <= 32 characters")]
    InvalidName,
    #[msg("Vault limit hit")]
    MaxVaults,
    #[msg("Invalid vault")]
    InvalidVault,
}
