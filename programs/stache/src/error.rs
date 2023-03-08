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
    #[msg("Insufficient funds")]
    InsufficientFunds,
    #[msg("Vault locked")]
    VaultLocked,
    #[msg("Invalid action")]
    InvalidAction,
    #[msg("Invalid trigger")]
    InvalidTrigger,
    #[msg("Already approved")]
    AlreadyApproved,
    #[msg("Token accounts do not match")]
    NonMatchingTokenAccounts,
    #[msg("Automation limit hit")]
    MaxAutos,
    #[msg("Hit limit")]
    HitLimit,
    #[msg("Automation can't be modified")]
    AutomationLocked,
    #[msg("Missing an expected account")]
    MissingAccount,
    #[msg("Token accounts mismatch")]
    TokenAccountsMismatch,
    #[msg("Duplicate accounts passed in")]
    DupeAccount,
    #[msg("Automation trigger account must be one of the action accounts")]
    AutomationTriggerAccountMismatch,

}
