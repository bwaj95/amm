use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::anchor_utils::{mint_tokens, transfer_tokens_checked};
use crate::events::InitialLiquidityAdded;
use crate::state::Pool;

use crate::constants::{LOCKED_LP_SEED, LP_MINT_DECIMALS, POOL_SEED};

use crate::error::AmmError::{self, *};
use crate::utils::calculate_lp_initial;
use crate::MINIMUM_LIQUIDITY;

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
        associated_token::mint = mint_a,
        associated_token::authority = pool
    )]
    pub vault_a: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = pool
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
    ctx: Context<AddInitialLiquidity>,
    amount_token_a: u64,
    amount_token_b: u64,
) -> Result<()> {
    //  check if pool  liquidity is truly uninitialised -> vault a b amount, lp suppkyu all 0
    require!(
        ctx.accounts.lp_mint.supply == 0 && ctx.accounts.locked_lp_token.amount == 0,
        AmmError::PoolAlreadyInitialized
    );

    //  validate non zero deposit amounts, also validate amount avail in token accounts
    require!(
        amount_token_a > 0 && amount_token_b > 0,
        AmmError::InvalidInputAmount
    );

    require!(
        ctx.accounts.provider_token_a_ata.amount >= amount_token_a
            && ctx.accounts.provider_token_b_ata.amount >= amount_token_b,
        AmmError::InsufficientFunds
    );

    //  calculate LP and check if it is above threshold LP, reject if not
    let initial_lp = calculate_lp_initial(amount_token_a, amount_token_b)?;
    require!(
        initial_lp > MINIMUM_LIQUIDITY,
        AmmError::MinimumLiquidityThresholdNotMet
    );

    // cpi 2 token transfers
    transfer_tokens_checked(
        &ctx.accounts.provider_token_a_ata.to_account_info(),
        &ctx.accounts.provider.to_account_info(),
        &ctx.accounts.vault_a.to_account_info(),
        &ctx.accounts.mint_a.to_account_info(),
        &ctx.accounts.token_program.to_account_info(),
        amount_token_a,
        ctx.accounts.mint_a.decimals,
    )?;
    transfer_tokens_checked(
        &ctx.accounts.provider_token_b_ata.to_account_info(),
        &ctx.accounts.provider.to_account_info(),
        &ctx.accounts.vault_b.to_account_info(),
        &ctx.accounts.mint_b.to_account_info(),
        &ctx.accounts.token_program.to_account_info(),
        amount_token_b,
        ctx.accounts.mint_b.decimals,
    )?;

    let mint_a_key = ctx.accounts.mint_a.key();
    let mint_b_key = ctx.accounts.mint_b.key();
    let pool_bump = &[ctx.bumps.pool];

    let pool_seeds: &[&[u8]] = &[
        POOL_SEED,
        mint_a_key.as_ref(),
        mint_b_key.as_ref(),
        pool_bump,
    ];
    let signer_seeds: &[&[&[u8]]] = &[pool_seeds];

    // mint min liq to locked lp
    mint_tokens(
        &ctx.accounts.lp_mint.to_account_info(),
        &ctx.accounts.locked_lp_token.to_account_info(),
        &ctx.accounts.pool.to_account_info(),
        signer_seeds,
        &ctx.accounts.token_program.to_account_info(),
        MINIMUM_LIQUIDITY,
    )?;

    // mint remaining lp to provider lp ata
    let provider_lp: u64 = initial_lp - MINIMUM_LIQUIDITY;
    mint_tokens(
        &ctx.accounts.lp_mint.to_account_info(),
        &ctx.accounts.provider_lp_token.to_account_info(),
        &ctx.accounts.pool.to_account_info(),
        signer_seeds,
        &ctx.accounts.token_program.to_account_info(),
        provider_lp,
    )?;

    // emit event
    emit!(InitialLiquidityAdded {
        pool: ctx.accounts.pool.key(),
        mint_a: ctx.accounts.mint_a.key(),
        mint_b: ctx.accounts.mint_b.key(),
        vault_a: ctx.accounts.vault_a.key(),
        vault_b: ctx.accounts.vault_b.key(),
        lp_mint: ctx.accounts.lp_mint.key(),
        provider: ctx.accounts.provider.key(),
        provider_token_a_ata: ctx.accounts.provider_token_a_ata.key(),
        provider_token_b_ata: ctx.accounts.provider_token_b_ata.key(),
        amount_a: amount_token_a,
        amount_b: amount_token_b,
        total_lp: initial_lp,
        provider_lp: provider_lp,
        locked_lp: MINIMUM_LIQUIDITY,
    });

    Ok(())
}
