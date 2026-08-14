use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::anchor_utils::{mint_tokens, transfer_tokens_checked};
use crate::constants::{LP_MINT_SEED, POOL_SEED};
use crate::events::LiquidityAdded;
use crate::state::Pool;
use crate::utils::calculate_add_liquidity;
use crate::{LP_MINT_DECIMALS, MINIMUM_LIQUIDITY};

use crate::error::AmmError;

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub provider: Signer<'info>,

    #[account(
        seeds= [POOL_SEED, mint_a.key().as_ref(), mint_b.key().as_ref()],
        bump = pool.bump,

        has_one = mint_a,
        has_one = mint_b,
        has_one = vault_a,
        has_one = vault_b,
        has_one = lp_mint
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        mut,
        seeds = [LP_MINT_SEED, pool.key().as_ref()],
        bump,
        mint::authority = pool,
        mint::decimals = LP_MINT_DECIMALS
    )]
    pub lp_mint: Box<Account<'info, Mint>>,

    // readonly external mints
    pub mint_a: Box<Account<'info, Mint>>,
    pub mint_b: Box<Account<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = pool
    )]
    pub vault_a: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority =pool
    )]
    pub vault_b: Box<Account<'info, TokenAccount>>,

    #[account(
     mut,
        associated_token::mint = mint_a,
        associated_token::authority = provider
    )]
    pub provider_token_a: Box<Account<'info, TokenAccount>>,
    #[account(
     mut,
     associated_token::mint = mint_b,
     associated_token::authority = provider
    )]
    pub provider_token_b: Box<Account<'info, TokenAccount>>,

    #[account(
     init_if_needed,
     payer = provider,
     associated_token::mint = lp_mint,
     associated_token::authority = provider
    )]
    pub provider_lp_token: Box<Account<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn add_liquidity_handler(
    ctx: Context<AddLiquidity>,
    max_amount_a: u64,
    max_amount_b: u64,
) -> Result<()> {
    require!(
        max_amount_a > 0 && max_amount_b > 0,
        AmmError::InvalidInputAmount
    );

    let (actual_a, actual_b, lp_to_mint) = calculate_add_liquidity(
        ctx.accounts.vault_a.amount,
        ctx.accounts.vault_b.amount,
        max_amount_a,
        max_amount_b,
        ctx.accounts.lp_mint.supply,
    )?;

    require!(
        ctx.accounts.provider_token_a.amount >= actual_a
            && ctx.accounts.provider_token_b.amount >= actual_b,
        AmmError::InsufficientFunds
    );

    // Anchor's in-memory accounts may not reflect CPI updates, unless account is reloaded.
    // Calculate final values before minting, and after success use in emit.
    let total_lp_after = ctx
        .accounts
        .lp_mint
        .supply
        .checked_add(lp_to_mint)
        .ok_or(AmmError::MathOverflow)?;

    let reserve_a_after = ctx
        .accounts
        .vault_a
        .amount
        .checked_add(actual_a)
        .ok_or(AmmError::MathOverflow)?;

    let reserve_b_after = ctx
        .accounts
        .vault_b
        .amount
        .checked_add(actual_b)
        .ok_or(AmmError::MathOverflow)?;

    // transfer tokens
    transfer_tokens_checked(
        &ctx.accounts.provider_token_a.to_account_info(),
        &ctx.accounts.provider.to_account_info(),
        &ctx.accounts.vault_a.to_account_info(),
        &ctx.accounts.mint_a.to_account_info(),
        &ctx.accounts.token_program.to_account_info(),
        actual_a,
        ctx.accounts.mint_a.decimals,
    )?;

    transfer_tokens_checked(
        &ctx.accounts.provider_token_b.to_account_info(),
        &ctx.accounts.provider.to_account_info(),
        &ctx.accounts.vault_b.to_account_info(),
        &ctx.accounts.mint_b.to_account_info(),
        &ctx.accounts.token_program.to_account_info(),
        actual_b,
        ctx.accounts.mint_b.decimals,
    )?;

    let pool_bump = ctx.accounts.pool.bump;
    let mint_a_key = ctx.accounts.mint_a.key();
    let mint_b_key = ctx.accounts.mint_b.key();

    let signer_seeds: &[&[u8]] = &[
        POOL_SEED,
        mint_a_key.as_ref(),
        mint_b_key.as_ref(),
        &[pool_bump],
    ];
    let signer: &[&[&[u8]]] = &[signer_seeds];

    // mint lp to user
    mint_tokens(
        &ctx.accounts.lp_mint.to_account_info(),
        &ctx.accounts.provider_lp_token.to_account_info(),
        &ctx.accounts.pool.to_account_info(),
        signer,
        &ctx.accounts.token_program.to_account_info(),
        lp_to_mint,
    )?;

    

    emit!(LiquidityAdded {
        pool: ctx.accounts.pool.key(),
        mint_a: ctx.accounts.mint_a.key(),
        mint_b: ctx.accounts.mint_b.key(),

        provider: ctx.accounts.provider.key(),
        amount_a: actual_a,
        amount_b: actual_b,
        lp_minted: lp_to_mint,

        total_lp_after,
        reserve_a_after,
        reserve_b_after
    });

    Ok(())
}
