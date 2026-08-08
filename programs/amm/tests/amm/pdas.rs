use amm::{LOCKED_LP_SEED, LP_MINT_SEED, MINT_SEED, POOL_SEED, PROTOCOL_CONFIG_SEED, TREASURY_SEED};
use anchor_lang::Key;
use solana_pubkey::Pubkey;

pub fn find_protocol_config_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[PROTOCOL_CONFIG_SEED], program_id)
}

pub fn find_protocol_treasury_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[TREASURY_SEED], program_id)
}

pub fn find_pool_pda(program_id: &Pubkey, mint_a: &Pubkey, mint_b: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[POOL_SEED, mint_a.key().as_ref(), mint_b.key().as_ref()],
        program_id,
    )
}

pub fn find_lp_mint_pda(program_id: &Pubkey, pool: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[LP_MINT_SEED, pool.key().as_ref()], program_id)
}

pub fn find_mint_pda(program_id: &Pubkey, mint_id: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[MINT_SEED, mint_id.to_le_bytes().as_ref()], program_id)
}

pub fn find_locked_lp_token_pda(program_id: &Pubkey, pool: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[LOCKED_LP_SEED, pool.as_ref()], program_id)
}
