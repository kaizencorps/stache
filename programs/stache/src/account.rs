use anchor_lang::prelude::*;

use crate::constant::{MAX_SUBMITTERS, MAX_APPROVERS, MAX_VAULTS, MAX_VAULT_ACTIONS, MAX_AUTOS};
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
    pub next_auto_index: u8,

    // vault ids that are currently active
    pub vaults: Vec<u8>,

    // automation ids that are currently active
    pub autos: Vec<u8>,

}

impl CurrentStache {
    pub const MAX_SIZE: usize = 1 + 1 + 32 + 32 + 32 + 1 + 1 + (4 + (MAX_VAULTS)) + (4 + (MAX_AUTOS)) +
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


    fn is_index(&self, index: u8, list: &Vec<u8>) -> Option<usize> {
        match list.iter().position(|&x| x == index) {
            Some(index) => Some(index),
            _ => None,
        }
    }

    pub fn is_vault(&self, index: u8) -> Option<usize> {
        return self.is_index(index, &self.vaults);
    }

    pub fn is_auto(&self, index: u8) -> Option<usize> {
        return self.is_index(index, &self.autos);
    }

    fn add_index(list: &mut Vec<u8>, max: usize, next_index: &mut u8) -> Result<u8> {
        // check that we've got room
        require!(usize::from(list.len()) < max, StacheError::HitLimit);

        // todo: handle wrapping properly
        let mut index: u8 = *next_index;
        if *next_index + 1 == u8::MAX {
            *next_index = 2;
            index = 1;
        } else {
            *next_index += 1;
        }

        list.push(index);
        return Ok(index);
    }

    // adds a vault, increments next vault index, and returns the index of added vault
    pub fn add_vault(&mut self) -> Result<u8> {
        return Self::add_index(&mut self.vaults, MAX_VAULTS, &mut self.next_vault_index);
    }

    // adds a vault, increments next vault index, and returns the index of added vault
    pub fn add_auto(&mut self) -> Result<u8> {
        return Self::add_index(&mut self.autos, MAX_AUTOS, &mut self.next_auto_index);
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
    Easy,   // not requiring sigs (like the stache)
    TwoSig,   // just 2 sigs
    Squads { multisig: Pubkey, sigs: u8}, //  squads; sigs = threshold
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
        1 + 1 + 32 +         // vault type
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
            VaultType::Easy => {
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
                    action_type: ActionType::Transfer,
                    approvers: vec![initiator.clone()],
                    action: TransferAction {
                        from: from.clone(),
                        to: to.clone(),
                        amount,
                    }.try_to_vec().unwrap(),
                };
                self.actions.push(action);
                return Ok(false);
            }
            _ => {
                // don't support multisig for now (todo)
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
pub enum ActionType {
    Transfer,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct TransferAction {
    pub from: Pubkey,
    pub to: Pubkey,
    pub amount: u64,
}

impl TransferAction {

    pub const MAX_SIZE: usize =
        8 +         // amount
            32;         // to
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct VaultAction {
    pub action_index: u8,
    pub action_type: ActionType,
    pub approvers: Vec<Pubkey>,
    pub action: Vec<u8>,      // depends on the ActionType
}

impl VaultAction {

    pub const MAX_SIZE: usize =
        1 +         // action index
        1 +         // action type
        4 + (32 * MAX_VAULTS)  +        // approvers
        128;        // should be good enough for whatever action for now

    pub fn transfer_action(&mut self) -> Result<TransferAction> {
        if self.action_type != ActionType::Transfer {
            return err!(StacheError::InvalidAction);
        }
        // deserialize the data into the TransferAction
        let withdraw_data = AnchorDeserialize::deserialize(&mut self.action.as_slice()).unwrap();
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

////////// AUTOMATIONS ///////

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum TriggerType {
    Balance,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct BalanceTrigger {
    pub account: Pubkey,
    pub trigger_balance: u64,   // the balance that triggers the action (balance gets either below/above this)
    pub above: bool,            // above = true, then trigger fires if balance is above trigger_balance, else when below
}


#[account]
pub struct Auto {
    pub stache: Pubkey,
    pub index: u8,
    pub bump: u8,
    pub active: bool,
    pub paused: bool,
    pub thread: Pubkey,         // clockwork thread
    pub name: String,
    pub action_type: ActionType,
    pub action: Vec<u8>,       //  depends on the action type
    pub trigger_type: TriggerType,
    pub trigger: Vec<u8>,
}

impl Auto {
    pub const MAX_SIZE: usize =
        32 +        // stache
        1 +        // index
        1 +         // bump
        1 +         // active
        1 +         // paused
        32 +         // thread
        32 +        // name
        1 +         // action type
        128;        // should be good enough for whatever action for now


    // todo: pull into a trait / remove dupe code

    pub fn transfer_action(&mut self) -> Result<TransferAction> {
        if self.action_type != ActionType::Transfer {
            return err!(StacheError::InvalidAction);
        }
        // deserialize the data into the TransferAction
        let withdraw_data = AnchorDeserialize::deserialize(&mut self.action.as_slice()).unwrap();
        Ok(withdraw_data)
    }

    pub fn balance_trigger(&mut self) -> Result<BalanceTrigger> {
        if self.trigger_type != TriggerType::Balance {
            return err!(StacheError::InvalidTrigger);
        }
        // deserialize the data into the TransferAction
        let trigger_data = AnchorDeserialize::deserialize(&mut self.trigger.as_slice()).unwrap();
        Ok(trigger_data)
    }
}


