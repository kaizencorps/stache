use anchor_lang::prelude::*;
use crate::program::Stache;

use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

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
    use anchor_spl::token::CloseAccount;
    use super::*;

    // creates the stache (beard) account
    pub fn create_stache(ctx: Context<CreateStache>) -> Result<()> {
        // todo: add a cpi call to keychain to check permissions (once it's implemented)
        //       - validate the bump .. ? since the constraint isn't working i think
        // for now we'll just check in the keychain directly

        let keychain = &mut ctx.accounts.keychain;
        // check that the creator is on the keychain
        require!(keychain.has_verified_key(&ctx.accounts.authority.key()), StacheError::NotAuthorized);

        // use the same name as the keychain
        ctx.accounts.stache.stache_id = keychain.name.clone();
        ctx.accounts.stache.domain = keychain.domain.clone();
        ctx.accounts.stache.keychain = ctx.accounts.keychain.key();
        ctx.accounts.stache.bump = *ctx.bumps.get("stache").unwrap();

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

    pub fn unstash(ctx: Context<Unstash>, amount: u64) -> Result<()> {

        // todo: proper checks

        let stache = &ctx.accounts.stache;
        //        seeds = [keychain.name.as_bytes().as_ref(), BEARD_SPACE.as_bytes().as_ref(), keychain.domain.as_ref(), STACHE.as_bytes().as_ref()]

        let seeds = &[
            stache.stache_id.as_bytes().as_ref(),
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

        // check that we've got room
        require!(usize::from(stache.vaults.len()) < MAX_VAULTS, StacheError::MaxVaults);

        // add the vault to the stache
        stache.vaults.push(ctx.accounts.vault.key());

        // setup the vault
        let vault = &mut ctx.accounts.vault;

        vault.stache = ctx.accounts.stache.key();
        vault.name = name;
        vault.vault_type = vault_type;
        vault.bump = *ctx.bumps.get("vault").unwrap();

        Ok(())

    }

    pub fn destroy_vault(ctx: Context<DestroyVault>) -> Result<()> {
        let stache = &mut ctx.accounts.stache;
        let tokens_left = ctx.accounts.vault_ata.amount;

        let vault = &ctx.accounts.vault;

        let seeds = &[
            vault.name.as_bytes().as_ref(),
            VAULT_SPACE.as_bytes().as_ref(),
            stache.stache_id.as_bytes().as_ref(),
            BEARD_SPACE.as_bytes().as_ref(),
            stache.domain.as_ref(),
            STACHE.as_bytes().as_ref(),
            &[vault.bump],
        ];

        let signer = &[&seeds[..]];

        let cpi_program = ctx.accounts.token_program.clone().to_account_info();

        // drain the vault first
        if (tokens_left > 0) {
            let cpi_transfer_accounts = Transfer {
                from: ctx.accounts.vault_ata.to_account_info(),
                to: ctx.accounts.drain_to.to_account_info(),
                authority: ctx.accounts.vault.to_account_info(),
            };
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.clone().to_account_info(),
                         cpi_transfer_accounts, signer);
            token::transfer(cpi_ctx, tokens_left)?;
            msg!("drained {} tokens from vault: {}, ata: {}", tokens_left, ctx.accounts.vault.key(), ctx.accounts.vault_ata.key());
        }

        // close the vault
        let cpi_close_accounts = CloseAccount {
            account: ctx.accounts.vault_ata.to_account_info(),
            destination: ctx.accounts.drain_to.to_account_info(),
            authority: ctx.accounts.vault.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(ctx.accounts.token_program.clone().to_account_info(),
                                                  cpi_close_accounts, signer);
        token::close_account(cpi_ctx)?;

        // now get rid of the vault from stache
        stache.remove_vault(&ctx.accounts.vault.key());

        Ok(())
    }



}


