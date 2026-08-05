use anchor_lang::prelude::*;

use crate::constants::*;

use crate::error::AmmError;
use crate::state::{ProtocolConfig, ProtocolTreasury};
use crate::events::{ProtocolInitialized};

#[derive(Accounts)]
pub struct InitializeProtocol<'info> {
    // admin
    #[account(mut)]
    pub admin: Signer<'info>,

    // protocol config pda
    #[account(
        init,
        space = 8 + ProtocolConfig::INIT_SPACE,
        payer = admin,
        seeds = [PROTOCOL_CONFIG_SEED],
        bump
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,

    // treasury pda
    #[account(
        init,
        payer = admin,
        space = 8 + ProtocolTreasury::INIT_SPACE,
        seeds = [TREASURY_SEED],
        bump
    )]
    pub protocol_treasury: Account<'info, ProtocolTreasury>,

    // system program
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializeProtocol>,
    swap_fee_bps: u16,
    treasury_fee_bps: u16,
) -> Result<()> {
    require!(swap_fee_bps <= MAX_BPS, AmmError::InvalidSwapFee);

    require!(
        treasury_fee_bps <= swap_fee_bps,
        AmmError::InvalidTreasuryFee
    );

    let protocol_config = &mut ctx.accounts.protocol_config;

    protocol_config.admin = ctx.accounts.admin.key();
    protocol_config.pending_admin = None;
    protocol_config.protocol_treasury = ctx.accounts.protocol_treasury.key();
    protocol_config.swap_fee_bps = swap_fee_bps;
    protocol_config.treasury_fee_bps = treasury_fee_bps;
    protocol_config.paused = false;
    protocol_config.bump = ctx.bumps.protocol_config;

    let protocol_treasury = &mut ctx.accounts.protocol_treasury;

    protocol_treasury.bump = ctx.bumps.protocol_treasury;

    emit!(ProtocolInitialized {
        admin: protocol_config.admin,
        protocol_treasury: protocol_config.protocol_treasury,
        swap_fee_bps: protocol_config.swap_fee_bps,
        treasury_fee_bps: protocol_config.treasury_fee_bps,
    });

    Ok(())
}
