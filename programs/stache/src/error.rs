use anchor_lang::prelude::*;

#[error_code]
pub enum StacheError {
    #[msg("Not authorized")]
    NotAuthorized,
}
