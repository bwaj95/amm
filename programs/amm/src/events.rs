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
    pub treasury_a: Pubkey,
    pub treasury_b: Pubkey,
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

#[event]
pub struct LiquidityAdded {
    pub pool: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,

    pub provider: Pubkey,
    pub amount_a: u64,
    pub amount_b: u64,
    pub lp_minted: u64,

    pub reserve_a_after: u64,
    pub reserve_b_after: u64,
    pub total_lp_after: u64,
}

#[event]
pub struct SwapExecuted {
    pub pool: Pubkey,
    pub mint_in: Pubkey,
    pub mint_out: Pubkey,

    pub provider: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,

    pub provider_token_in: Pubkey,
    pub provider_token_out: Pubkey,

    pub total_fee: u64,
    pub treasury_fee: u64,
    pub lp_fee: u64,
}

#[event]
pub struct LiquidityRemoved {
    pub pool: Pubkey,
    pub provider: Pubkey,

    pub mint_a: Pubkey,
    pub mint_b: Pubkey,

    pub lp_burned: u64,

    pub amount_a: u64,
    pub amount_b: u64,

    pub reserve_a_after: u64,
    pub reserve_b_after: u64,
    pub total_lp_after: u64,
}
