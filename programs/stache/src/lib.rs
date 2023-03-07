use anchor_lang::prelude::*;
use crate::program::Stache;

use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, CloseAccount, Mint, Token, TokenAccount, Transfer};

declare_id!("71KtSAv6Qtpa2AZAwDhipKeoiCoyUXKbLpNheTJUGW8B");

pub mod error;
pub mod account;
pub mod constant;
pub mod context;
mod util;

use error::*;
use account::*;
use constant::*;
use context::*;
use util::*;


#[program]
pub mod stache {
    use super::*;

    // creates the stache (beard) account
    pub fn create_stache(ctx: Context<CreateStache>) -> Result<()> {
        // todo: add a cpi call to keychain to check permissions (once it's implemented)
        //       - validate the bump .. ? since the constraint isn't working i think
        // for now we'll just check in the keychain directly

        let keychain = &mut ctx.accounts.keychain;
        // check that the creator is on the keychain
        require!(keychain.has_verified_key(&ctx.accounts.authority.key()), StacheError::NotAuthorized);

        let stache = &mut ctx.accounts.stache;

        // use the same name as the keychain
        stache.stacheid = keychain.name.clone();
        stache.domain = keychain.domain.clone();
        stache.keychain = ctx.accounts.keychain.key();
        stache.bump = *ctx.bumps.get("stache").unwrap();
        stache.next_vault_index = 1;    // we'll start at 1 and reserve 0 in case we wanna use it later
        stache.vaults = Vec::with_capacity(MAX_VAULTS);

        Ok(())
    }

    pub fn destroy_stache(ctx: Context<DestroyStache>) -> Result<()> {
        let keychain = &mut ctx.accounts.keychain;
        // check that the creator is on the keychain
        require!(keychain.has_verified_key(&ctx.accounts.authority.key()), StacheError::NotAuthorized);

        // todo: needs to be a 2-sig thing for security
        // todo: check for any stashes (token accounts)
        // todo: check for vaults

        Ok(())
    }

    // right now this is just a wrapper around the token transfer instruction, but possibly will
    // do some additional checks/stuff here depending on settings
    pub fn stash(ctx: Context<Stash>, amount: u64) -> Result<()> {
        // todo: proper checks

        let cpi_accounts = Transfer {
            from: ctx.accounts.from_token.to_account_info(),
            to: ctx.accounts.stache_ata.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        Ok(())
    }

    pub fn unstash_sol(ctx: Context<UnstashSol>, lamports: u64) -> Result<()> {

        let rent = &ctx.accounts.rent;
        let min_rent = rent.minimum_balance(8 + CurrentStache::MAX_SIZE);

        let from_account = ctx.accounts.stache.to_account_info();
        let to_account = ctx.accounts.owner.to_account_info();

        // don't allow an unstash to close our stache account
        if **from_account.try_borrow_lamports()? - min_rent - lamports < 0 {
            return Err(StacheError::InsufficientFunds.into());
        }

        // Debit from_account and credit to_account
        **from_account.try_borrow_mut_lamports()? -= lamports;
        **to_account.try_borrow_mut_lamports()? += lamports;
        Ok(())
    }

    pub fn unstash(ctx: Context<Unstash>, amount: u64) -> Result<()> {

        // todo: proper checks

        let stache = &ctx.accounts.stache;
        //        seeds = [keychain.name.as_bytes().as_ref(), BEARD_SPACE.as_bytes().as_ref(), keychain.domain.as_ref(), STACHE.as_bytes().as_ref()]

        let seeds = &[
            stache.stacheid.as_bytes().as_ref(),
            BEARD_SPACE.as_bytes().as_ref(),
            stache.domain.as_ref(),
            STACHE.as_bytes().as_ref(),
            &[stache.bump],
        ];

        let signer = &[&seeds[..]];

        let cpi_accouts = Transfer {
            from: ctx.accounts.stache_ata.to_account_info(),
            to: ctx.accounts.to_token.to_account_info(),
            authority: ctx.accounts.stache.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accouts,
            signer);
        token::transfer(cpi_ctx, amount)?;

        Ok(())
    }

    pub fn create_vault(ctx: Context<CreateVault>, name: String, vault_type: VaultType) -> Result<()> {

        let is_valid_name = is_valid_name(&name);
        require!(is_valid_name, StacheError::InvalidName);

        let stache = &mut ctx.accounts.stache;


        // add the vault to the stache
        let vault_index = stache.add_vault()?;

        // todo: if squads vault, verify squads multisig seed
        // seeds = [b"squad", create_key.as_ref(), b"multisig"], bump

        // setup the vault
        let vault = &mut ctx.accounts.vault;

        vault.stache = ctx.accounts.stache.key();
        vault.index = vault_index;
        vault.name = name;
        vault.vault_type = vault_type;
        vault.bump = *ctx.bumps.get("vault").unwrap();
        vault.next_action_index = 1;
        vault.locked = false;

        Ok(())
    }

    pub fn lock_vault(ctx: Context<LockVault>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.locked = true;
        Ok(())
    }


    pub fn withdraw_from_vault(ctx: Context<WithdrawFromVault>, amount: u64) -> Result<()> {
        let stache = &mut ctx.accounts.stache;
        let tokens_left = ctx.accounts.vault_ata.amount;

        let vault_authority = ctx.accounts.vault.clone().to_account_info();
        let mut vault = &mut ctx.accounts.vault;

        require!(amount <= tokens_left, StacheError::InsufficientFunds);

        if vault.withdraw(&ctx.accounts.authority.key(), &ctx.accounts.vault_ata.key(), &ctx.accounts.to_token.key(), amount).unwrap() {
            // withdraw
            transfer_from_vault(&stache,
                                &mut vault,
                                vault_authority,
                                ctx.accounts.vault_ata.clone().to_account_info(),
                                ctx.accounts.to_token.clone().to_account_info(),
                                amount,
                                ctx.accounts.token_program.clone().to_account_info())?;


        }

        Ok(())
    }

    pub fn approve_action<'a, 'b, 'c, 'info>(ctx: Context<'a, 'b, 'c, 'info, ApproveVaultAction<'info>>, action_index: u8) -> Result<()> {

        msg!("approving vault action");
        let stache = &mut ctx.accounts.stache;

        let vault_authority = ctx.accounts.vault.clone().to_account_info();
        let vault = &mut ctx.accounts.vault;
        let vault_type = vault.vault_type.clone();
        let vault_action = vault.get_action(action_index).unwrap();

        match vault_action.action {
            VaultActionType::Withdraw => {

                vault_action.approve(&ctx.accounts.authority.key())?;

                if vault_action.count_approvers() == 2 && vault_type == VaultType::TwoSig {

                    let withdraw_vault_action_data = vault_action.withdraw_action()?;

                    // check that the remaining accounts passed in match
                    let accs = &mut ctx.remaining_accounts.iter();
                    let from = next_account_info(accs)?.to_account_info();
                    let to = next_account_info(accs)?.to_account_info();

                    require!(from.key() == withdraw_vault_action_data.from, StacheError::InvalidAction);
                    require!(to.key() == withdraw_vault_action_data.to, StacheError::InvalidAction);

                    let from_token = Account::<'_, TokenAccount>::try_from(&from).unwrap();

                    require!(withdraw_vault_action_data.amount <= from_token.amount, StacheError::InsufficientFunds);

                    // withdraw
                    transfer_from_vault(&stache,
                                         vault,
                                        vault_authority,
                                        from,
                                        to,
                                        withdraw_vault_action_data.amount,
                                        ctx.accounts.token_program.clone().to_account_info())?;

                    // action has been executed
                    vault.remove_action(action_index);
                }
            },
        }
        Ok(())
    }

    pub fn deny_action(ctx: Context<DenyVaultAction>, action_index: u8) -> Result<()> {
        let mut vault = &mut ctx.accounts.vault;
        vault.remove_action(action_index);
        Ok(())
    }

    pub fn destroy_vault(ctx: Context<DestroyVault>) -> Result<()> {
        let stache = &mut ctx.accounts.stache;

        // get rid of the vault from stache
        stache.remove_vault(ctx.accounts.vault.index);

        Ok(())
    }

}

// transfer some tokens out of a vault ata
pub fn transfer_from_vault<'a, 'b>(current_stache: &CurrentStache,
                                   vault: &Vault,
                                   vault_authority: AccountInfo<'a>,
                                   from_vault_ata: AccountInfo<'b>,
                                   to_token: AccountInfo<'b>,
                                   amount: u64,
                                   token_program: AccountInfo<'a>) -> Result<()>
        where 'a: 'b, 'b: 'a {

    let binding = vault.index.to_le_bytes();
    let seeds = &[
        binding.as_ref(),
        VAULT_SPACE.as_bytes().as_ref(),
        current_stache.stacheid.as_bytes().as_ref(),
        BEARD_SPACE.as_bytes().as_ref(),
        current_stache.domain.as_ref(),
        STACHE.as_bytes().as_ref(),
        &[vault.bump],
    ];

    let signer = &[&seeds[..]];

    let cpi_transfer_accounts = Transfer {
        from: from_vault_ata.clone(),
        to: to_token.clone(),
        authority: vault_authority.clone(),
    };
    let cpi_ctx = CpiContext::new_with_signer(
        token_program.clone(),
        cpi_transfer_accounts, signer);

    let from_token_account = Account::<'_, TokenAccount>::try_from(&from_vault_ata).unwrap();

    let tokens_available = from_token_account.amount;
    if tokens_available < amount {
        return err!(StacheError::InsufficientFunds);
    }
    token::transfer(cpi_ctx, amount)?;

    msg!("transfered {} tokens from vault ata: {}, to account: {}", amount, from_vault_ata.key(), to_token.key());

    // now see if the vault is empty and close if it is
    if tokens_available == amount {
        msg!("closing vault ata: {}", from_vault_ata.key());

        let cpi_close_accounts = CloseAccount {
            account: from_vault_ata.clone(),
            destination: to_token.clone(),
            authority: vault_authority.clone(),
        };
        let cpi_ctx = CpiContext::new_with_signer(token_program.clone(),
                                                  cpi_close_accounts, signer);
        token::close_account(cpi_ctx)?;
    }

    Ok(())
}




