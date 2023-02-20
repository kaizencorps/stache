use anchor_lang::prelude::*;
use crate::program::Stache;

use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("71KtSAv6Qtpa2AZAwDhipKeoiCoyUXKbLpNheTJUGW8B");

pub mod error;
pub mod account;
pub mod constant;
pub mod context;

use error::*;
use account::*;
use constant::*;
use context::*;


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
        // todo: check for any stashes (token accounts) so they don't get orphaned

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
}


