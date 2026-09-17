use anchor_lang::prelude::*;

use crate::{
    constants::PROTOCOL_CONFIG_SEED,
    error::AmmError,
    events::{
        AdminTransferCancelled, AdminTransferProposed, AdminTransferred, ProtocolFeesUpdated,
        ProtocolPauseChanged,
    },
    state::ProtocolConfig,
    utils::validate_fee_config,
};

#[derive(Accounts)]
pub struct AdminOnly<'info> {
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [PROTOCOL_CONFIG_SEED],
        bump = protocol_config.bump,
        constraint = protocol_config.admin == admin.key() @ AmmError::UnauthorizedAdmin,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,
}

#[derive(Accounts)]
pub struct AcceptAdmin<'info> {
    pub pending_admin: Signer<'info>,

    #[account(
        mut,
        seeds = [PROTOCOL_CONFIG_SEED],
        bump = protocol_config.bump,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,
}

pub fn set_paused_handler(ctx: Context<AdminOnly>, paused: bool) -> Result<()> {
    ctx.accounts.protocol_config.paused = paused;

    emit!(ProtocolPauseChanged {
        admin: ctx.accounts.admin.key(),
        paused,
    });

    Ok(())
}

pub fn update_fees_handler(
    ctx: Context<AdminOnly>,
    swap_fee_bps: u16,
    treasury_fee_bps: u16,
) -> Result<()> {
    require!(
        ctx.accounts.protocol_config.paused,
        AmmError::ProtocolMustBePaused
    );

    validate_fee_config(swap_fee_bps, treasury_fee_bps)?;

    let protocol_config = &mut ctx.accounts.protocol_config;
    let previous_swap_fee_bps = protocol_config.swap_fee_bps;
    let previous_treasury_fee_bps = protocol_config.treasury_fee_bps;

    protocol_config.swap_fee_bps = swap_fee_bps;
    protocol_config.treasury_fee_bps = treasury_fee_bps;

    emit!(ProtocolFeesUpdated {
        admin: ctx.accounts.admin.key(),
        previous_swap_fee_bps,
        previous_treasury_fee_bps,
        new_swap_fee_bps: swap_fee_bps,
        new_treasury_fee_bps: treasury_fee_bps,
    });

    Ok(())
}

pub fn propose_admin_handler(ctx: Context<AdminOnly>, new_admin: Pubkey) -> Result<()> {
    require!(
        new_admin != Pubkey::default() && new_admin != ctx.accounts.protocol_config.admin,
        AmmError::InvalidAdmin
    );

    ctx.accounts.protocol_config.pending_admin = Some(new_admin);

    emit!(AdminTransferProposed {
        current_admin: ctx.accounts.admin.key(),
        pending_admin: new_admin,
    });

    Ok(())
}

pub fn cancel_admin_transfer_handler(ctx: Context<AdminOnly>) -> Result<()> {
    let cancelled_pending_admin = ctx
        .accounts
        .protocol_config
        .pending_admin
        .ok_or(AmmError::NoPendingAdmin)?;

    ctx.accounts.protocol_config.pending_admin = None;

    emit!(AdminTransferCancelled {
        admin: ctx.accounts.admin.key(),
        cancelled_pending_admin,
    });

    Ok(())
}

pub fn accept_admin_handler(ctx: Context<AcceptAdmin>) -> Result<()> {
    let protocol_config = &mut ctx.accounts.protocol_config;
    let new_admin = protocol_config
        .pending_admin
        .ok_or(AmmError::NoPendingAdmin)?;

    require_keys_eq!(
        new_admin,
        ctx.accounts.pending_admin.key(),
        AmmError::UnauthorizedPendingAdmin
    );

    let previous_admin = protocol_config.admin;
    protocol_config.admin = new_admin;
    protocol_config.pending_admin = None;

    emit!(AdminTransferred {
        previous_admin,
        new_admin,
    });

    Ok(())
}
