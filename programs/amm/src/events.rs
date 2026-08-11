use anchor_lang::prelude::*;

#[event]
pub struct ProtocolInitialized {
    pub admin: Pubkey,
    pub protocol_treasury: Pubkey,
    pub swap_fee_bps: u16,
    pub treasury_fee_bps: u16,
}

#[event]
pub struct PoolCreated {
    pub pool: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub vault_a: Pubkey,
    pub vault_b: Pubkey,
    pub lp_mint: Pubkey,
    pub locked_lp_token: Pubkey,
}

#[event]
pub struct InitialLiquidityAdded {
    pub pool: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub vault_a: Pubkey,
    pub vault_b: Pubkey,
    pub lp_mint: Pubkey,
    pub provider: Pubkey,
    pub provider_token_a_ata: Pubkey,
    pub provider_token_b_ata: Pubkey,
    pub amount_a: u64,
    pub amount_b: u64,
    pub total_lp: u64,
    pub provider_lp: u64,
    pub locked_lp: u64,
}
