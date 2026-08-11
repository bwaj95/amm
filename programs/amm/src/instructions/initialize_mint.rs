use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::{anchor_utils::mint_tokens, state::ProtocolConfig, MINT_SEED, PROTOCOL_CONFIG_SEED};

use crate::error::AmmError;

/// Utility Mint creation for Config to init and use in tests
#[derive(Accounts)]
#[instruction(mint_id: u64, decimals: u8)]
pub struct InitializeMint<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        seeds = [PROTOCOL_CONFIG_SEED],
        bump
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,

    #[account(
        init,
        payer = admin,
        mint::authority = protocol_config,
        mint::decimals = decimals,
        seeds = [MINT_SEED, mint_id.to_le_bytes().as_ref()],
        bump
    )]
    pub mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,
}

pub fn initialize_mint_handler(
    _ctx: Context<InitializeMint>,
    mint_id: u64,
    decimals: u8,
) -> Result<()> {
    Ok(())
}

#[derive(Accounts)]
#[instruction(mint_id: u64)]
pub struct MintTokens<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        mint::authority = protocol_config,
        seeds = [MINT_SEED, mint_id.to_le_bytes().as_ref()],
        bump
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        seeds = [PROTOCOL_CONFIG_SEED],
        bump,

        has_one = admin
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,

    pub user: SystemAccount<'info>,

    #[account(
        init_if_needed,
        payer = admin,
        associated_token::mint = mint,
        associated_token::authority = user,
    )]
    pub user_ata: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn mint_tokens_handler(ctx: Context<MintTokens>, amount: u64) -> Result<()> {
    let bump = ctx.bumps.protocol_config;
    let signer_seeds: &[&[&[u8]]] = &[&[PROTOCOL_CONFIG_SEED, &[bump]]];

    mint_tokens(
        &ctx.accounts.mint.to_account_info(),
        &ctx.accounts.user_ata.to_account_info(),
        &ctx.accounts.protocol_config.to_account_info(),
        signer_seeds,
        &ctx.accounts.token_program.to_account_info(),
        amount,
    )?;

    Ok(())
}
