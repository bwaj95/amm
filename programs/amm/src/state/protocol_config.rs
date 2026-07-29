use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct ProtocolConfig {
    pub admin: Pubkey,
    pub pending_admin: Option<Pubkey>,
    pub protocol_treasury: Pubkey,
    pub swap_fee_bps: u16,
    pub treasury_fee_bps: u16,
    pub paused: bool,
    pub bump: u8,
    pub _reserved: [u8; 32],
}
