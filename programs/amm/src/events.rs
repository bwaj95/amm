use anchor_lang::prelude::*;

#[event]
pub struct ProtocolInitialized {
    pub admin: Pubkey,
    pub protocol_treasury: Pubkey,
    pub swap_fee_bps: u16,
    pub treasury_fee_bps: u16,
}


