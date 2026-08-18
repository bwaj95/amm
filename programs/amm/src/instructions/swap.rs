use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::anchor_utils::{transfer_tokens_checked, transfer_tokens_checked_with_signer};
use crate::constants::{POOL_SEED, PROTOCOL_CONFIG_SEED};
use crate::error::AmmError::{self};
use crate::events::SwapExecuted;
use crate::state::{Pool, ProtocolConfig, ProtocolTreasury};
use crate::utils::{calculate_swap, SwapCalculation};
use crate::TREASURY_SEED;

#[derive(Accounts)]
pub struct Swap<'info> {
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
        has_one = treasury_a,
        has_one = treasury_b,
    )]
    pub pool: Box<Account<'info, Pool>>,

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
        init_if_needed,
        payer = provider,
        associated_token::mint = mint_a,
        associated_token::authority = provider,
    )]
    pub provider_token_a: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = provider,
        associated_token::mint = mint_b,
        associated_token::authority = provider,
    )]
    pub provider_token_b: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [TREASURY_SEED],
        bump = protocol_treasury.bump,
    )]
    pub protocol_treasury: Box<Account<'info, ProtocolTreasury>>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = protocol_treasury
    )]
    pub treasury_a: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = protocol_treasury
    )]
    pub treasury_b: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [PROTOCOL_CONFIG_SEED],
        bump = protocol_config.bump
    )]
    pub protocol_config: Box<Account<'info, ProtocolConfig>>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn swap_handler(
    ctx: Context<Swap>,
    amount_in: u64,
    min_amount_out: u64,
    a_to_b: bool,
) -> Result<()> {
    let (mint_in, mint_out, vault_in, vault_out, provider_in, provider_out, treasury_in) = if a_to_b
    {
        (
            &ctx.accounts.mint_a,
            &ctx.accounts.mint_b,
            &ctx.accounts.vault_a,
            &ctx.accounts.vault_b,
            &ctx.accounts.provider_token_a,
            &ctx.accounts.provider_token_b,
            &ctx.accounts.treasury_a,
        )
    } else {
        (
            &ctx.accounts.mint_b,
            &ctx.accounts.mint_a,
            &ctx.accounts.vault_b,
            &ctx.accounts.vault_a,
            &ctx.accounts.provider_token_b,
            &ctx.accounts.provider_token_a,
            &ctx.accounts.treasury_b,
        )
    };

    require!(provider_in.amount >= amount_in, AmmError::InsufficientFunds);

    let swap_calculation: SwapCalculation = calculate_swap(
        vault_in.amount,
        vault_out.amount,
        amount_in,
        ctx.accounts.protocol_config.swap_fee_bps,
        ctx.accounts.protocol_config.treasury_fee_bps,
    )?;

    require!(
        vault_out.amount >= swap_calculation.amount_out,
        AmmError::InsufficientLiquidity
    );

    require!(
        swap_calculation.amount_out >= min_amount_out,
        AmmError::SlippageExceeded
    );

    //  cpi transfer checked amount_in from provider_in to vault_in
    transfer_tokens_checked(
        &provider_in.to_account_info(),
        &ctx.accounts.provider.to_account_info(),
        &vault_in.to_account_info(),
        &mint_in.to_account_info(),
        &ctx.accounts.token_program,
        amount_in,
        mint_in.decimals,
    )?;

    //  cpi transfer checked with signer from amount_out from vault_out owned by pool pda to provider_out
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
        &vault_out.to_account_info(),
        &ctx.accounts.pool.to_account_info(),
        &provider_out.to_account_info(),
        &mint_out.to_account_info(),
        &ctx.accounts.token_program,
        swap_calculation.amount_out,
        mint_out.decimals,
        signer_seeds,
    )?;

    // cpi transfer checked with signer the treasury_fees from vault_in owned by pool pda to treasury_in
    transfer_tokens_checked_with_signer(
        &vault_in.to_account_info(),
        &ctx.accounts.pool.to_account_info(),
        &treasury_in.to_account_info(),
        &mint_in.to_account_info(),
        &ctx.accounts.token_program.to_account_info(),
        swap_calculation.treasury_fees,
        mint_in.decimals,
        signer_seeds,
    )?;

    emit!(SwapExecuted {
        pool: ctx.accounts.pool.key(),
        mint_in: mint_in.key(),
        mint_out: mint_out.key(),
        provider: ctx.accounts.provider.key(),
        amount_in,
        amount_out: swap_calculation.amount_out,
        provider_token_in: provider_in.key(),
        provider_token_out: provider_out.key(),
        treasury_fee: swap_calculation.treasury_fees,
        lp_fee: swap_calculation.lp_fees,
        total_fee: swap_calculation.total_swap_fees,
    });

    Ok(())
}
