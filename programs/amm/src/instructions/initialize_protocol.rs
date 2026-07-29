use anchor_lang::prelude::*;

use crate::constants::*;

use crate::state::{ProtocolConfig, ProtocolTreasury};

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
    _ctx: Context<InitializeProtocol>,
    _swap_fee_bps: u16,
    _treasury_fee_bps: u16,
) -> Result<()> {
    Ok(())
}
