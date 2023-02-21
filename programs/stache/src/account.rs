use anchor_lang::prelude::*;

use crate::constant::{MAX_SUBMITTERS, MAX_APPROVERS, MAX_VAULTS};


// "current" cause later we'll use the versioning system that keychain

#[account]
pub struct CurrentStache {
    pub version: u8,
    pub bump: u8,
    pub keychain: Pubkey,
    pub domain: Pubkey,
    pub stache_id: String,
    // the keychain domain
    pub vaults: Vec<Pubkey>,
    // pub submitters: Vec<Submitter>,
    // pub approvers: Vec<Approver>,
}

impl CurrentStache {
    // previous account size = 304 bytes
    // pub const MAX_SIZE: usize = 1 + 1 + 32 + 32 + 32 + (4 + (MAX_SUBMITTERS * 33)) + (4 + (MAX_APPROVERS * 33));
    // this size: 258 + rest padding (128) = 386
    pub const MAX_SIZE: usize = 1 + 1 + 32 + 32 + 32 + (4 + (MAX_VAULTS * 32)) +
        128; // extra space for now;
    pub const CURRENT_VERSION: u8 = 1;

    pub fn remove_vault(&mut self, vault: &Pubkey) {
        let index = self.is_vault(vault).unwrap();
        self.vaults.swap_remove(index);
    }

    pub fn is_vault(&self, vault: &Pubkey) -> Option<usize> {
        match self.vaults.iter().position(|&x| &x == vault) {
            Some(index) => Some(index),
            _ => None,
        }
    }
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

// "sub" stache accounts always start w/the stache so we can filter using getProgramAccounts

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum VaultType {
    Standard,
    Multisig
}


#[account]
pub struct Vault {
    pub stache: Pubkey,
    pub name: String,
    pub vault_type: VaultType,
    pub bump: u8,
    // the token account
    pub token_account: Pubkey,
}

impl Vault {
    pub const MAX_SIZE: usize =
        32 +        // stache
        32 +        // name
        1 +         // vault type
        1 +         // bump
        32 +        // token account
        128;        // extra space for now during dev
}
