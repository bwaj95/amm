use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::state::{Pool, ProtocolConfig};
use crate::{
    constants::{POOL_SEED, PROTOCOL_CONFIG_SEED},
    state::ProtocolTreasury,
    LOCKED_LP_SEED, LP_MINT_DECIMALS, TREASURY_SEED,
};
use crate::{error::AmmError, LP_MINT_SEED};

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
    pub protocol_config: Box<Account<'info, ProtocolConfig>>,

    pub mint_a: Box<Account<'info, Mint>>,
    pub mint_b: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = creator,
        space = 8 + Pool::INIT_SPACE,
        constraint = mint_a.key() != mint_b.key() @ AmmError::SameMint,
        constraint = mint_a.key() < mint_b.key() @ AmmError::InvalidMintOrder,
        seeds=[POOL_SEED, mint_a.key().as_ref(), mint_b.key().as_ref()],
        bump
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        init,
        payer = creator,
        associated_token::mint = mint_a,
        associated_token::authority = pool
    )]
    pub vault_a: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = creator,
        associated_token::mint = mint_b,
        associated_token::authority = pool
    )]
    pub vault_b: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [TREASURY_SEED],
        bump
    )]
    pub protocol_treasury: Box<Account<'info, ProtocolTreasury>>,

    #[account(
    init,
    payer = creator,
    associated_token::mint = mint_a,
    associated_token::authority = protocol_treasury,
)]
    pub treasury_a: Box<Account<'info, TokenAccount>>,

    #[account(
    init,
    payer = creator,
    associated_token::mint = mint_b,
    associated_token::authority = protocol_treasury,
)]
    pub treasury_b: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = creator,
        mint::authority = pool,
        mint::decimals = LP_MINT_DECIMALS, // 9
        seeds = [LP_MINT_SEED, pool.key().as_ref()], // lp_mint for this specific pool, derived using the pool pubkey.
        bump
    )]
    pub lp_mint: Box<Account<'info, Mint>>,

    #[account(
    init,
    payer = creator,
    token::mint = lp_mint,
    token::authority = pool,
    seeds = [LOCKED_LP_SEED, pool.key().as_ref()],
    bump)]
    pub locked_lp_token: Box<Account<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn create_pool_handler(_ctx: Context<CreatePool>) -> Result<()> {
    err!(AmmError::DeprecatedInstruction)
}
