use anchor_lang::prelude::*;

use crate::constant::{MAX_SUBMITTERS, MAX_APPROVERS, MAX_VAULTS, MAX_VAULT_ACTIONS};
use crate::error::StacheError;


// "current" cause later we'll use the versioning system that keychain

#[account]
pub struct CurrentStache {
    pub version: u8,
    pub bump: u8,
    pub keychain: Pubkey,
    pub domain: Pubkey,
    pub stacheid: String,

    // next vault index; since there can only be MAX_VAULTS at a time, previous ones will expire so this index will eventually wrap
    pub next_vault_index: u8,

    // vault ids that are currently active
    pub vaults: Vec<u8>,

    // pub vaults: Vec<Pubkey>,
    // pub submitters: Vec<Submitter>,
    // pub approvers: Vec<Approver>,
}

impl CurrentStache {
    // previous account size = 304 bytes
    // pub const MAX_SIZE: usize = 1 + 1 + 32 + 32 + 32 + (4 + (MAX_SUBMITTERS * 33)) + (4 + (MAX_APPROVERS * 33));
    // this size: 258 + rest padding (128) = 386
    pub const MAX_SIZE: usize = 1 + 1 + 32 + 32 + 32 + (4 + (MAX_VAULTS)) +
        128; // extra space for now;
    pub const CURRENT_VERSION: u8 = 1;

    pub fn remove_vault(&mut self, index: u8) {
        let index = self.is_vault(index).unwrap();
        self.vaults.swap_remove(index);
    }

    // pub fn is_vault(&self, vault: &Pubkey) -> Option<usize> {
    //     match self.vaults.iter().position(|&x| &x == vault) {
    //         Some(index) => Some(index),
    //         _ => None,
    //     }
    // }

    pub fn is_vault(&self, index: u8) -> Option<usize> {
        match self.vaults.iter().position(|&x| x == index) {
            Some(index) => Some(index),
            _ => None,
        }
    }

    // adds a vault, increments next vault index, and returns the index of added vault
    pub fn add_vault(&mut self) -> Result<u8> {
        // check that we've got room
        require!(usize::from(self.vaults.len()) < MAX_VAULTS, StacheError::MaxVaults);

        let mut index = self.next_vault_index;
        if self.next_vault_index + 1 == u8::MAX {
            // todo: handle wrapping properly
            self.next_vault_index = 2;
            index = 1;
        } else {
            self.next_vault_index += 1;
        }
        self.vaults.push(index);
        return Ok(index);
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

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum VaultType {
    Simple,   // "open" not requiring sigs (like the stash)
    TwoSig,   // just 2 sigs
    Multisig { sigs: u8}, //  sigs = threshold (squads)
}


#[account]
pub struct Vault {
    pub stache: Pubkey,
    pub index: u8,
    pub bump: u8,
    pub vault_type: VaultType,
    pub locked: bool,   // for multisig squads vaults, this basically doesn't apply since the ms can be independently un/locked
    pub name: String,
    pub next_action_index: u8,      // todo: deal with wrapping
    pub actions: Vec<VaultAction>,
}

impl Vault {

    pub const MAX_SIZE: usize =
        32 +        // stache
        1 +        // index
        1 +         // bump
        2 +         // vault type
        1 +         // locked
        32 +        // name
        (4 + (MAX_VAULT_ACTIONS * VaultAction::MAX_SIZE)) + // actions
        128;        // extra space for now during dev

    pub fn get_action(&mut self, action_index: u8) -> Option<&mut VaultAction> {
        match self.actions.iter().position(|x| x.action_index == action_index) {
            Some(index) => Some(&mut self.actions[index]),
            _ => None,
        }
    }

    pub fn is_action(&self, action_index: u8) -> Option<usize> {
        match self.actions.iter().position(|x| x.action_index == action_index) {
            Some(index) => Some(index),
            _ => None,
        }
    }

    pub fn remove_action(&mut self, index: u8) {
        match self.actions.iter().position(|x| x.action_index == index) {
            Some(index) => {
                self.actions.swap_remove(index);
            }
            _ => {}
        }
    }

    // return whether to proceed with withdrawal or not
    pub fn withdraw(&mut self, initiator: &Pubkey, from: &Pubkey, to: &Pubkey, amount: u64) -> Result<bool> {
        if self.locked {
            return Err(StacheError::VaultLocked.into());
        }
        match self.vault_type {
            VaultType::Simple => {
                // do the withdraw
                return Ok(true);
            }
            VaultType::TwoSig => {
                // create the action

                // todo: deal with expiring previous actions correctly to make sure we don't have duplicate indexes
                self.next_action_index = match self.next_action_index + 1 {
                    u8::MAX => 1,
                    _ => self.next_action_index + 1,
                };

                let mut action = VaultAction {
                    action_index: self.next_action_index - 1,
                    action: VaultActionType::Withdraw,
                    approvers: vec![initiator.clone()],
                    data: WithdrawVaultActionData {
                        from: from.clone(),
                        to: to.clone(),
                        amount,
                    }.try_to_vec().unwrap(),
                };
                self.actions.push(action);
                return Ok(false);
            }
            _ => {
                // don't support multisig for now
                msg!("withdraw for vault type not supported: {:?}", self.vault_type);
                return Ok(false);
            }
        }
    }

    pub fn is_type(&self, vault_type: VaultType) -> bool {
        self.vault_type == vault_type
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum VaultActionType {
    Withdraw,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct WithdrawVaultActionData {
    pub from: Pubkey,
    pub to: Pubkey,
    pub amount: u64,
}

impl WithdrawVaultActionData {

    pub const MAX_SIZE: usize =
        8 +         // amount
            32;         // to
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct VaultAction {
    pub action_index: u8,
    pub action: VaultActionType,
    pub approvers: Vec<Pubkey>,
    pub data: Vec<u8>,
}

impl VaultAction {

    pub const MAX_SIZE: usize =
        1 +         // action type
        128;        // should be good enough for now right?

    pub fn withdraw_action(&mut self) -> Result<WithdrawVaultActionData> {
        if self.action != VaultActionType::Withdraw {
            return err!(StacheError::InvalidVaultAction);
        }
        // deserialize the data into the WithdrawVaultActionData
        let withdraw_data = AnchorDeserialize::deserialize(&mut self.data.as_slice()).unwrap();
        Ok(withdraw_data)
    }

    pub fn approve(&mut self, approver: &Pubkey) -> Result<()> {
        if self.approvers.contains(approver) {
            return err!(StacheError::AlreadyApproved);
        }
        self.approvers.push(*approver);
        Ok(())
    }

    pub fn count_approvers(&self) -> usize {
        self.approvers.len()
    }

}

