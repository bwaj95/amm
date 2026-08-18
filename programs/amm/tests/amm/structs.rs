use solana_pubkey::Pubkey;


pub struct InitializedPoolStruct {
    pub pool: Pubkey,

    pub mint_a: Pubkey,
    pub mint_b: Pubkey,

    pub mint_a_id: u64,
    pub mint_b_id: u64,

    pub vault_a: Pubkey,
    pub vault_b: Pubkey,

    pub lp_mint: Pubkey,
    pub locked_lp_token: Pubkey,

    pub treasury_a: Pubkey,
    pub treasury_b: Pubkey,
}

pub struct UserPoolAccounts {
    pub token_a: Pubkey,
    pub token_b: Pubkey,
    pub lp_token: Pubkey,
}

pub struct PoolReservesSnapshot {
    pub pool: Pubkey,
    pub reserve_a: u64,
    pub reserve_b: u64,
    pub lp_supply: u64,
    pub treasury_a_amount: u64,
    pub treasury_b_amount: u64
}