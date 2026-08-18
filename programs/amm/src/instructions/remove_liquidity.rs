use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::anchor_utils::{
    burn_tokens, transfer_tokens_checked, transfer_tokens_checked_with_signer,
};
use crate::constants::{POOL_SEED, PROTOCOL_CONFIG_SEED};
use crate::error::AmmError::{self};
use crate::events::{LiquidityRemoved, SwapExecuted};
use crate::state::{Pool, ProtocolConfig, ProtocolTreasury};
use crate::utils::{calculate_remove_liquidity, calculate_swap, SwapCalculation};
use crate::{LP_MINT_DECIMALS, LP_MINT_SEED, MINIMUM_LIQUIDITY, TREASURY_SEED};

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    #[account(mut)]
    pub provider: Signer<'info>,

    // client should do the canonical ordering before proiding the mints
    pub mint_a: Box<Account<'info, Mint>>,
    pub mint_b: Box<Account<'info, Mint>>,

    #[account(
        seeds = [POOL_SEED, mint_a.key().as_ref(),  mint_b.key().as_ref()],
        bump = pool.bump,

        has_one = mint_a,
        has_one = mint_b,
        has_one = vault_a,
        has_one = vault_b,
        has_one = lp_mint,
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

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = pool,
    )]
    pub vault_a: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = pool,
    )]
    pub vault_b: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = provider,
    )]
    pub provider_token_a: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = provider,
    )]
    pub provider_token_b: Box<Account<'info, TokenAccount>>,

    #[account(
     mut,
     associated_token::mint = lp_mint,
     associated_token::authority = provider
    )]
    pub provider_lp_token: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

pub fn remove_liquidity_handler(
    ctx: Context<RemoveLiquidity>,
    lp_amount: u64,
    min_amount_a: u64,
    min_amount_b: u64,
) -> Result<()> {
    require!(
        ctx.accounts.lp_mint.supply >= MINIMUM_LIQUIDITY,
        AmmError::InvalidLiquidityPoolState
    );

    let removable_lp = ctx
        .accounts
        .lp_mint
        .supply
        .checked_sub(MINIMUM_LIQUIDITY)
        .ok_or(AmmError::InvalidLiquidityPoolState)?;

    require!(
        lp_amount <= removable_lp,
        AmmError::InvalidLiquidityPoolState
    );

    require!(
        ctx.accounts.provider_lp_token.amount >= lp_amount,
        AmmError::InsufficientFunds
    );

    // calculate amoount_a, amount_b with util fn
    let remove_lp_calculation = calculate_remove_liquidity(
        ctx.accounts.vault_a.amount,
        ctx.accounts.vault_b.amount,
        ctx.accounts.lp_mint.supply,
        lp_amount,
    )?;

    require!(
        remove_lp_calculation.amount_a >= min_amount_a,
        AmmError::SlippageExceeded
    );
    require!(
        remove_lp_calculation.amount_b >= min_amount_b,
        AmmError::SlippageExceeded
    );

    // burn user lp tokens using the token burn cpi
    burn_tokens(
        &ctx.accounts.provider_lp_token.to_account_info(),
        &ctx.accounts.provider.to_account_info(),
        &ctx.accounts.lp_mint.to_account_info(),
        &ctx.accounts.token_program.to_account_info(),
        lp_amount,
    )?;

    // cpi token transfer with signer to transfer tokens from vaults to user atas
    let pool_bump = ctx.accounts.pool.bump;
    let mint_a_key = ctx.accounts.mint_a.key();
    let mint_b_key = ctx.accounts.mint_b.key();
    let signer_seeds: &[&[&[u8]]] = &[&[
        POOL_SEED,
        mint_a_key.as_ref(),
        mint_b_key.as_ref(),
        &[pool_bump],
    ]];

    transfer_tokens_checked_with_signer(
        &ctx.accounts.vault_a.to_account_info(),
        &ctx.accounts.pool.to_account_info(),
        &ctx.accounts.provider_token_a.to_account_info(),
        &ctx.accounts.mint_a.to_account_info(),
        &ctx.accounts.token_program.to_account_info(),
        remove_lp_calculation.amount_a,
        ctx.accounts.mint_a.decimals,
        signer_seeds,
    )?;

    transfer_tokens_checked_with_signer(
        &ctx.accounts.vault_b.to_account_info(),
        &ctx.accounts.pool.to_account_info(),
        &ctx.accounts.provider_token_b.to_account_info(),
        &ctx.accounts.mint_b.to_account_info(),
        &ctx.accounts.token_program.to_account_info(),
        remove_lp_calculation.amount_b,
        ctx.accounts.mint_b.decimals,
        signer_seeds,
    )?;

    let reserve_a_after = ctx
        .accounts
        .vault_a
        .amount
        .checked_sub(remove_lp_calculation.amount_a)
        .ok_or(AmmError::MathOverflow)?;

    let reserve_b_after = ctx
        .accounts
        .vault_b
        .amount
        .checked_sub(remove_lp_calculation.amount_b)
        .ok_or(AmmError::MathOverflow)?;

    let total_lp_after = ctx
        .accounts
        .lp_mint
        .supply
        .checked_sub(lp_amount)
        .ok_or(AmmError::MathOverflow)?;

    emit!(LiquidityRemoved {
        pool: ctx.accounts.pool.key(),
        provider: ctx.accounts.provider.key(),

        mint_a: ctx.accounts.mint_a.key(),
        mint_b: ctx.accounts.mint_b.key(),

        lp_burned: lp_amount,

        amount_a: remove_lp_calculation.amount_a,
        amount_b: remove_lp_calculation.amount_b,

        reserve_a_after,
        reserve_b_after,

        total_lp_after,
    });

    Ok(())
}
