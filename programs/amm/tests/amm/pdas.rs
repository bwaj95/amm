use amm::{PROTOCOL_CONFIG_SEED, TREASURY_SEED};
use solana_pubkey::Pubkey;

pub fn find_protocol_config_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[PROTOCOL_CONFIG_SEED], program_id)
}

pub fn find_protocol_treasury_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[TREASURY_SEED], program_id)
}
