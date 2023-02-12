use anchor_lang::prelude::*;

use crate::constant::{MAX_SUBMITTERS, MAX_APPROVERS};

#[account]
pub struct Beard {
    pub version: u8,
    pub bump: u8,
    pub name: String,
    // the keychain domain
    pub domain: Pubkey,
    pub keychain: Pubkey,
    pub submitters: Vec<Submitter>,
    pub approvers: Vec<Approver>
}

impl Beard {
    pub const MAX_SIZE: usize = 1 + 1 + 32 + 32 + 32 + (4 + (MAX_SUBMITTERS * 33)) + (4 + (MAX_APPROVERS * 33));
    pub const CURRENT_VERSION: u8 = 1;
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct Submitter {
    pub key: Pubkey,
    pub enabled: bool,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct Approver {
    pub key: Pubkey,
    pub enabled: bool,
}
