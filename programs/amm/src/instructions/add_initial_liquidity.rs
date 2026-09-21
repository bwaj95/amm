use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::state::Pool;

use crate::constants::{LOCKED_LP_SEED, LP_MINT_DECIMALS, POOL_SEED};

use crate::error::AmmError::{self};

#[derive(Accounts)]
pub struct AddInitialLiquidity<'info> {
    #[account(mut)]
    pub provider: Signer<'info>,

    #[account(
        seeds = [POOL_SEED, mint_a.key().as_ref(), mint_b.key().as_ref()],
        bump,

        has_one = mint_a,
        has_one = mint_b,
        has_one = vault_a,
        has_one = vault_b,
        has_one = lp_mint,
        has_one = locked_lp_token,
    )]
    pub pool: Box<Account<'info, Pool>>,

    // readonly mints
    pub mint_a: Box<Account<'info, Mint>>,
    pub mint_b: Box<Account<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = provider,
    )]
    pub provider_token_a_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = provider
    )]
    pub provider_token_b_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = provider,
        associated_token::mint = lp_mint,
        associated_token::authority = provider
    )]
    pub provider_lp_token: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        address = pool.vault_a,
        token::mint = mint_a,
        token::authority = pool
    )]
    pub vault_a: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        address = pool.vault_b,
        token::mint = mint_b,
        token::authority = pool
    )]
    pub vault_b: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        mint::authority = pool,
        mint::decimals = LP_MINT_DECIMALS
    )]
    pub lp_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [LOCKED_LP_SEED, pool.key().as_ref()],
        bump,
        token::mint = lp_mint,
        token::authority = pool
    )]
    pub locked_lp_token: Box<Account<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn add_initial_liquidity_handler(
    _ctx: Context<AddInitialLiquidity>,
    _amount_token_a: u64,
    _amount_token_b: u64,
) -> Result<()> {
    err!(AmmError::DeprecatedInstruction)
}
