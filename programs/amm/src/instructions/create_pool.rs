use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::{
    constants::{POOL_SEED, PROTOCOL_CONFIG_SEED},
    LOCKED_LP_SEED, LP_MINT_DECIMALS,
};
use crate::{error::AmmError, LP_MINT_SEED};
use crate::{
    events::PoolCreated,
    state::{Pool, ProtocolConfig},
};

#[derive(Accounts)]
pub struct CreatePool<'info> {
    // permissionless pool creation
    // creator need not be admin
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        seeds=[PROTOCOL_CONFIG_SEED],
        bump
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,

    pub mint_a: Account<'info, Mint>,
    pub mint_b: Account<'info, Mint>,

    #[account(
        init,
        payer = creator,
        space = 8 + Pool::INIT_SPACE,
        constraint = mint_a.key() != mint_b.key() @ AmmError::SameMint,
        constraint = mint_a.key() < mint_b.key() @ AmmError::InvalidMintOrder,
        seeds=[POOL_SEED, mint_a.key().as_ref(), mint_b.key().as_ref()],
        bump
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        init,
        payer = creator,
        associated_token::mint = mint_a,
        associated_token::authority = pool
    )]
    pub vault_a: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = creator,
        associated_token::mint = mint_b,
        associated_token::authority = pool
    )]
    pub vault_b: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = creator,
        mint::authority = pool,
        mint::decimals = LP_MINT_DECIMALS, // 9
        seeds = [LP_MINT_SEED, pool.key().as_ref()], // lp_mint for this specific pool, derived using the pool pubkey.
        bump
    )]
    pub lp_mint: Account<'info, Mint>,

    #[account(
    init,
    payer = creator,
    token::mint = lp_mint,
    token::authority = pool,
    seeds = [LOCKED_LP_SEED, pool.key().as_ref()],
    bump)]
    pub locked_lp_token: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn create_pool_handler(ctx: Context<CreatePool>) -> Result<()> {
    let pool: &mut Pool = &mut ctx.accounts.pool;

    pool.mint_a = ctx.accounts.mint_a.key();
    pool.mint_b = ctx.accounts.mint_b.key();
    pool.vault_a = ctx.accounts.vault_a.key();
    pool.vault_b = ctx.accounts.vault_b.key();
    pool.lp_mint = ctx.accounts.lp_mint.key();
    pool.locked_lp_token = ctx.accounts.locked_lp_token.key();
    pool.bump = ctx.bumps.pool;

    emit!(PoolCreated {
        pool: ctx.accounts.pool.key(),
        mint_a: ctx.accounts.mint_a.key(),
        mint_b: ctx.accounts.mint_b.key(),
        vault_a: ctx.accounts.vault_a.key(),
        vault_b: ctx.accounts.vault_b.key(),
        lp_mint: ctx.accounts.lp_mint.key(),
        locked_lp_token: ctx.accounts.locked_lp_token.key(),
    });

    Ok(())
}
