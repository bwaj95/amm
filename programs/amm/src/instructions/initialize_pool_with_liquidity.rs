use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::anchor_utils::{mint_tokens, transfer_tokens_checked};
use crate::error::AmmError;
use crate::events::{InitialLiquidityAdded, PoolCreated};
use crate::state::{ProtocolConfig, ProtocolTreasury};
use crate::utils::{calculate_lp_initial, validate_deadline};
use crate::{state::Pool, POOL_SEED};
use crate::{
    LOCKED_LP_SEED, LP_MINT_DECIMALS, LP_MINT_SEED, MINIMUM_LIQUIDITY, PROTOCOL_CONFIG_SEED,
    TREASURY_SEED, VAULT_A_SEED, VAULT_B_SEED,
};

#[derive(Accounts)]
pub struct InitializePoolWithLiquidity<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    // read only mints
    pub mint_a: Box<Account<'info, Mint>>,
    pub mint_b: Box<Account<'info, Mint>>,

    // pool pda
    #[account(
        init,
        payer = creator,
        space = 8 + Pool::INIT_SPACE,
        constraint = mint_a.key() != mint_b.key() @ AmmError::SameMint,
        constraint = mint_a.key() < mint_b.key() @ AmmError::InvalidMintOrder,
        seeds = [POOL_SEED, mint_a.key().as_ref(), mint_b.key().as_ref()],
        bump
    )]
    pub pool: Box<Account<'info, Pool>>,

    // 2 vaults - pool controlled pdas under token account
    #[account(
        init,
        payer = creator,
        seeds = [VAULT_A_SEED, pool.key().as_ref()],
        bump,
        token::mint = mint_a,
        token::authority = pool
    )]
    pub vault_a: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = creator,
        seeds = [VAULT_B_SEED, pool.key().as_ref()],
        bump,
        token::mint = mint_b,
        token::authority = pool
    )]
    pub vault_b: Box<Account<'info, TokenAccount>>,

    #[account(
    seeds = [PROTOCOL_CONFIG_SEED],
    bump = protocol_config.bump,
    has_one = protocol_treasury,
    constraint = !protocol_config.paused @ AmmError::ProtocolPaused,
    )]
    pub protocol_config: Box<Account<'info, ProtocolConfig>>,

    // protocol treasury
    #[account(
        seeds = [TREASURY_SEED],
        bump
    )]
    pub protocol_treasury: Box<Account<'info, ProtocolTreasury>>,

    // 2 treasury atas
    #[account(
        init_if_needed,
        payer = creator,
        associated_token::mint = mint_a,
        associated_token::authority = protocol_treasury,
    )]
    pub treasury_a: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = creator,
        associated_token::mint = mint_a,
        associated_token::authority = protocol_treasury,
    )]
    pub treasury_b: Box<Account<'info, TokenAccount>>,

    // lp mint
    #[account(
        init,
        payer = creator,
        mint::authority = pool,
        mint::decimals = LP_MINT_DECIMALS, // 9
        seeds = [LP_MINT_SEED, pool.key().as_ref()], // lp_mint for this specific pool, derived using the pool pubkey.
        bump
    )]
    pub lp_mint: Box<Account<'info, Mint>>,

    // locked lp token - why is this already token acc and not assoc token acc?
    #[account(
    init,
    payer = creator,
    token::mint = lp_mint,
    token::authority = pool,
    seeds = [LOCKED_LP_SEED, pool.key().as_ref()],
    bump)]
    pub locked_lp_token: Box<Account<'info, TokenAccount>>,

    // user token a
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = creator,
    )]
    pub provider_token_a_ata: Box<Account<'info, TokenAccount>>,

    // user token b
    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = creator
    )]
    pub provider_token_b_ata: Box<Account<'info, TokenAccount>>,

    // user lp token
    #[account(
        init_if_needed,
        payer = creator,
        associated_token::mint = lp_mint,
        associated_token::authority = creator
    )]
    pub provider_lp_token: Box<Account<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn initialize_pool_with_liquidity_handler(
    ctx: Context<InitializePoolWithLiquidity>,
    amount_token_a: u64,
    amount_token_b: u64,
    min_lp_out: u64,
    deadline: i64,
) -> Result<()> {
    validate_deadline(deadline, Clock::get()?.unix_timestamp)?;

    let pool: &mut Box<Account<'_, Pool>> = &mut ctx.accounts.pool;

    pool.set_inner(Pool {
        mint_a: ctx.accounts.mint_a.key(),
        mint_b: ctx.accounts.mint_b.key(),
        vault_a: ctx.accounts.vault_a.key(),
        vault_b: ctx.accounts.vault_b.key(),
        treasury_a: ctx.accounts.treasury_a.key(),
        treasury_b: ctx.accounts.treasury_b.key(),
        lp_mint: ctx.accounts.lp_mint.key(),
        locked_lp_token: ctx.accounts.locked_lp_token.key(),
        bump: ctx.bumps.pool,
    });

    require!(
        ctx.accounts.lp_mint.supply == 0 && ctx.accounts.locked_lp_token.amount == 0,
        AmmError::PoolAlreadyInitialized
    );

    require!(
        ctx.accounts.vault_a.amount == 0 && ctx.accounts.vault_b.amount == 0,
        AmmError::InitialVaultNotEmpty
    );

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
    let provider_lp: u64 = initial_lp
        .checked_sub(MINIMUM_LIQUIDITY)
        .ok_or(AmmError::MathOverflow)?;
    require!(provider_lp >= min_lp_out, AmmError::SlippageExceeded);

    // cpi 2 token transfers
    transfer_tokens_checked(
        &ctx.accounts.provider_token_a_ata.to_account_info(),
        &ctx.accounts.creator.to_account_info(),
        &ctx.accounts.vault_a.to_account_info(),
        &ctx.accounts.mint_a.to_account_info(),
        &ctx.accounts.token_program.to_account_info(),
        amount_token_a,
        ctx.accounts.mint_a.decimals,
    )?;
    transfer_tokens_checked(
        &ctx.accounts.provider_token_b_ata.to_account_info(),
        &ctx.accounts.creator.to_account_info(),
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
    mint_tokens(
        &ctx.accounts.lp_mint.to_account_info(),
        &ctx.accounts.provider_lp_token.to_account_info(),
        &ctx.accounts.pool.to_account_info(),
        signer_seeds,
        &ctx.accounts.token_program.to_account_info(),
        provider_lp,
    )?;

    // emit events - maybe we can make a single event combining the two?
    emit!(PoolCreated {
        pool: ctx.accounts.pool.key(),
        mint_a: ctx.accounts.mint_a.key(),
        mint_b: ctx.accounts.mint_b.key(),
        vault_a: ctx.accounts.vault_a.key(),
        vault_b: ctx.accounts.vault_b.key(),
        treasury_a: ctx.accounts.treasury_a.key(),
        treasury_b: ctx.accounts.treasury_b.key(),
        lp_mint: ctx.accounts.lp_mint.key(),
        locked_lp_token: ctx.accounts.locked_lp_token.key(),
    });

    emit!(InitialLiquidityAdded {
        pool: ctx.accounts.pool.key(),
        mint_a: ctx.accounts.mint_a.key(),
        mint_b: ctx.accounts.mint_b.key(),
        vault_a: ctx.accounts.vault_a.key(),
        vault_b: ctx.accounts.vault_b.key(),
        lp_mint: ctx.accounts.lp_mint.key(),
        provider: ctx.accounts.creator.key(),
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
