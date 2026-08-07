use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};

use crate::{state::ProtocolConfig, MINT_SEED, PROTOCOL_CONFIG_SEED};

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
