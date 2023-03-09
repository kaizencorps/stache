use anchor_lang::prelude::*;
use crate::account::*;
use crate::constant::*;
use crate::error::*;

use keychain::program::Keychain;
use keychain::account::CurrentKeyChain;

use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use clockwork_sdk::{
    self,
    state::{Thread, Trigger, ThreadAccount, ThreadResponse},
    ThreadProgram,
};

#[derive(Accounts)]
pub struct CreateStache<'info> {

    #[account(
    init,
    payer = authority,
    seeds = [keychain.name.as_bytes().as_ref(), BEARD_SPACE.as_bytes().as_ref(), keychain.domain.as_ref(), STACHE.as_bytes().as_ref()],
    bump,
    space = 8 + CurrentStache::MAX_SIZE,
    )]
    pub stache: Account<'info, CurrentStache>,

    // #[account(mut, owner = keychain_program.key)]
    #[account(constraint = keychain.has_key(&authority.key()))]
    pub keychain: Account<'info, CurrentKeyChain>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program <'info, System>,
    pub keychain_program: Program<'info, Keychain>,

}

#[derive(Accounts)]
pub struct DestroyStache<'info> {

    #[account(
    mut,
    // send lamports to the authority (must be on the keychain)
    close = authority,
    has_one = keychain
    )]
    pub stache: Account<'info, CurrentStache>,

    // #[account(mut, owner = keychain_program.key)]
    #[account(constraint = keychain.has_key(&authority.key()))]
    pub keychain: Account<'info, CurrentKeyChain>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program <'info, System>,
    pub keychain_program: Program<'info, Keychain>,
}



#[derive(Accounts)]
pub struct Stash<'info> {

    #[account(
    mut,
    has_one = keychain
    )]
    pub stache: Account<'info, CurrentStache>,

    #[account(constraint = keychain.has_key(&owner.key()))]
    pub keychain: Account<'info, CurrentKeyChain>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = stache
    )]
    pub stache_ata: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut, has_one = owner)]
    pub from_token: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}


#[derive(Accounts)]
pub struct UnstashSol<'info> {
    #[account(
    mut,
    has_one = keychain
    )]
    pub stache: Account<'info, CurrentStache>,

    #[account(constraint = keychain.has_key(&owner.key()))]
    pub keychain: Account<'info, CurrentKeyChain>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub rent: Sysvar<'info, Rent>,

    // pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Unstash<'info> {

    #[account(
    mut,
    has_one = keychain
    )]
    pub stache: Account<'info, CurrentStache>,

    #[account(constraint = keychain.has_key(&owner.key()))]
    pub keychain: Account<'info, CurrentKeyChain>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = stache
    )]
    pub stache_ata: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut, has_one = owner)]
    pub to_token: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}


/////////// VAULTS ///////////

#[derive(Accounts)]
#[instruction(name: String)]
pub struct CreateVault<'info> {

    #[account(
    mut,
    has_one = keychain
    )]
    pub stache: Account<'info, CurrentStache>,

    #[account(constraint = keychain.has_key(&authority.key()))]
    pub keychain: Account<'info, CurrentKeyChain>,

    #[account(
    init,
    payer = authority,
    seeds = [&stache.next_vault_index.to_le_bytes(),
             VAULT_SPACE.as_bytes().as_ref(),
             stache.stacheid.as_bytes().as_ref(),
             BEARD_SPACE.as_bytes().as_ref(),
             stache.domain.as_ref(),
             STACHE.as_bytes().as_ref()],
    bump,
    space = 8 + Vault::MAX_SIZE,
    )]
    pub vault: Account<'info, Vault>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct LockVault<'info> {

    #[account(
    mut,
    constraint = stache.is_vault(vault.index).is_some() @StacheError::InvalidVault,
    has_one = keychain,
    )]
    pub stache: Account<'info, CurrentStache>,

    #[account(constraint = keychain.has_key(&authority.key()))]
    pub keychain: Account<'info, CurrentKeyChain>,

    #[account(
    mut,
    has_one = stache,
    )]
    pub vault: Account<'info, Vault>,

    #[account(mut)]
    pub authority: Signer<'info>,
}


#[derive(Accounts)]
pub struct DestroyVault<'info> {

    #[account(
    mut,
    constraint = stache.is_vault(vault.index).is_some() @StacheError::InvalidVault,
    has_one = keychain,
    )]
    pub stache: Account<'info, CurrentStache>,

    #[account(constraint = keychain.has_key(&authority.key()))]
    pub keychain: Account<'info, CurrentKeyChain>,

    #[account(
    mut,
    has_one = stache,
    close = authority,
    )]
    pub vault: Account<'info, Vault>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct WithdrawFromVault<'info> {

    #[account(
    mut,
    constraint = stache.is_vault(vault.index).is_some() @StacheError::InvalidVault,
    has_one = keychain,
    )]
    pub stache: Account<'info, CurrentStache>,

    #[account(constraint = keychain.has_key(&authority.key()))]
    pub keychain: Account<'info, CurrentKeyChain>,

    #[account(
    mut,
    has_one = stache,
    )]
    pub vault: Account<'info, Vault>,

    #[account(
    mut,
    associated_token::mint = mint,
    associated_token::authority = vault,
    )]
    pub vault_ata: Account<'info, TokenAccount>,

    // where to send the tokens when emptying
    #[account(
    mut,
    token::mint = mint,
    )]
    pub to_token: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
#[instruction(action_index: u8)]
pub struct ApproveVaultAction<'info> {

    #[account(
    mut,
    constraint = stache.is_vault(vault.index).is_some() @StacheError::InvalidVault,
    has_one = keychain,
    )]
    pub stache: Account<'info, CurrentStache>,

    #[account(constraint = keychain.has_key(&authority.key()))]
    pub keychain: Account<'info, CurrentKeyChain>,

    #[account(
    mut,
    has_one = stache,
    constraint = vault.is_action(action_index).is_some() @StacheError::InvalidAction,
    )]
    pub vault: Account<'info, Vault>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,


    /* if the action is a withdraw, then the remaining accounts will be:
    from vault ata, to token account
     */

}

#[derive(Accounts)]
pub struct DenyVaultAction<'info> {

    #[account(
    mut,
    constraint = stache.is_vault(vault.index).is_some() @StacheError::InvalidVault,
    has_one = keychain,
    )]
    pub stache: Account<'info, CurrentStache>,

    #[account(constraint = keychain.has_key(&authority.key()))]
    pub keychain: Account<'info, CurrentKeyChain>,

    #[account(
    mut,
    has_one = stache,
    )]
    pub vault: Account<'info, Vault>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

//////// AUTOMATIONS ////////

#[derive(Accounts)]
pub struct CreateAutomation<'info> {

    #[account(
        mut,
        has_one = keychain,
    )]
    pub stache: Account<'info, CurrentStache>,

    #[account(constraint = keychain.has_key(&authority.key()))]
    pub keychain: Account<'info, CurrentKeyChain>,

    #[account(
        init,
        payer = authority,
        seeds = [&stache.next_auto_index.to_le_bytes(),
        AUTO_SPACE.as_bytes().as_ref(),
        stache.stacheid.as_bytes().as_ref(),
        BEARD_SPACE.as_bytes().as_ref(),
        stache.domain.as_ref(),
        STACHE.as_bytes().as_ref()],
        bump,
        space = 8 + Auto::MAX_SIZE,
    )]
    pub auto: Account<'info, Auto>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DestroyAutomation<'info> {

    #[account(
    mut,
    has_one = keychain,
    )]
    pub stache: Account<'info, CurrentStache>,

    #[account(constraint = keychain.has_key(&authority.key()))]
    pub keychain: Account<'info, CurrentKeyChain>,

    #[account(
    mut,
    has_one = stache,
    close = authority,
    )]
    pub auto: Account<'info, Auto>,

    #[account(mut)]
    pub authority: Signer<'info>,

    // if a thread was attached, we'll need these 2 to destroy the thread
    #[account(mut,
    address = Thread::pubkey(auto.key(), auto.name.clone().into()))
    ]
    pub thread: Option<Account<'info, Thread>>,
    pub clockwork_program: Option<Program<'info, ThreadProgram>>,
}

#[derive(Accounts)]
pub struct SetAutomationTrigger<'info> {

    #[account(
    mut,
    has_one = keychain,
    )]
    pub stache: Account<'info, CurrentStache>,

    #[account(constraint = keychain.has_key(&authority.key()))]
    pub keychain: Account<'info, CurrentKeyChain>,

    #[account(
    mut,
    has_one = stache,
    )]
    pub auto: Account<'info, Auto>,

    #[account(mut)]
    pub authority: Signer<'info>,

    // depending on the trigger being set, this token account might not be needed
    #[account(
    mut,
    )]
    pub token: Option<Account<'info, TokenAccount>>,
}

#[derive(Accounts)]
pub struct SetAutomationAction<'info> {

    #[account(
    mut,
    has_one = keychain,
    )]
    pub stache: Account<'info, CurrentStache>,

    #[account(constraint = keychain.has_key(&authority.key()))]
    pub keychain: Account<'info, CurrentKeyChain>,

    #[account(
    mut,
    has_one = stache,
    )]
    pub auto: Account<'info, Auto>,

    #[account(mut)]
    pub authority: Signer<'info>,

    // depending on the trigger being set, these accounts might not be needed but for now we'll just make them req'd to make my life easier
    // todo: we can make these remaining accounts later since they're tied to a specific action

    // for now, needs to be a stache ata
    #[account(
    associated_token::mint = mint,
    associated_token::authority = stache
    )]
    pub from_token: Option<Account<'info, TokenAccount>>,

    #[account(
    token::mint = mint,
    )]
    pub to_token: Option<Account<'info, TokenAccount>>,

    pub mint: Account<'info, Mint>,
    pub associated_token_program: Option<Program<'info, AssociatedToken>>,
}

#[derive(Accounts)]
pub struct ActivateAutomation<'info> {

    #[account(
    mut,
    has_one = keychain,
    )]
    pub stache: Account<'info, CurrentStache>,

    #[account(constraint = keychain.has_key(&authority.key()))]
    pub keychain: Account<'info, CurrentKeyChain>,

    #[account(
    mut,
    has_one = stache,
    )]
    pub auto: Account<'info, Auto>,

    #[account(mut)]
    pub authority: Signer<'info>,

    // the clockwork thread account
    #[account(mut,
        address = Thread::pubkey(auto.key(), auto.name.clone().into()))
    ]
    pub thread: SystemAccount<'info>,

    pub clockwork_program: Program<'info, ThreadProgram>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct FireAutomation<'info> {

    #[account(
    mut,
    )]
    pub stache: Account<'info, CurrentStache>,

    #[account(
    mut,
    has_one = stache,
    )]
    pub auto: Account<'info, Auto>,

    // the clockwork thread account - optional in case the user wants to test the automation outside of clockwork
    #[account(
    mut,
    signer,
    constraint = thread.authority.eq(&auto.key()) @StacheError::InvalidThread,
    address = Thread::pubkey(auto.key(), auto.name.clone().into()))
    ]
    pub thread: Account<'info, Thread>,

    // for doing transfers we'll need the appropriate token accounts - but normally would be optional cause depends what the automation needs but for now req'd since
    // our only action = transfer and i don't have time to solve "instruction tries to borrow reference for an account which is already borrowed" error

    #[account(mut)]
    pub from_token: Account<'info, TokenAccount>,

    #[account(mut)]
    pub to_token: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,

}
