use amm::state::{Pool, ProtocolConfig, ProtocolTreasury};
use anchor_lang::AccountDeserialize;
use anchor_spl::token::{spl_token::state::Mint, TokenAccount};
use solana_pubkey::Pubkey;
use solana_sdk::program_pack::Pack;

use crate::{
    amm::pdas::{find_protocol_config_pda, find_protocol_treasury_pda},
    common::context::TestContext,
};

pub fn protocol_config(ctx: &TestContext) -> ProtocolConfig {
    let account = ctx
        .svm
        .get_account(&find_protocol_config_pda(&ctx.program_id).0)
        .expect("Protocol Config not found.");

    let mut data: &[u8] = &account.data;
    let config: ProtocolConfig = ProtocolConfig::try_deserialize(&mut data).unwrap();

    config
}

pub fn protocol_treasury(ctx: &TestContext) -> ProtocolTreasury {
    let account = ctx
        .svm
        .get_account(&find_protocol_treasury_pda(&ctx.program_id).0)
        .expect("Protocol Treasury not found.");

    let mut data: &[u8] = &account.data;
    let treasury: ProtocolTreasury = ProtocolTreasury::try_deserialize(&mut data).unwrap();

    treasury
}

pub fn pool(ctx: &TestContext, pool: &Pubkey) -> Pool {
    let account = ctx.svm.get_account(pool).expect("Pool not found.");
    let mut data: &[u8] = &account.data;

    Pool::try_deserialize(&mut data).unwrap()
}

pub fn mint(ctx: &TestContext, lp_mint: &Pubkey) -> Mint {
    let account = ctx.svm.get_account(lp_mint).expect("LP Mint not found.");
    let data: &[u8] = &account.data;

    Mint::unpack(data).unwrap()
}

pub fn ata(ctx: &TestContext, ata: &Pubkey) -> TokenAccount {
    let account = ctx.svm.get_account(ata).expect("ATA not found.");
    let mut data: &[u8] = &account.data;

    TokenAccount::try_deserialize(&mut data).unwrap()
}

pub fn token_account(ctx: &TestContext, pubkey: &Pubkey) -> TokenAccount {
    let account = ctx
        .svm
        .get_account(pubkey)
        .expect("Token account not found");

    let mut data: &[u8] = &account.data;

    TokenAccount::try_deserialize(&mut data).expect("Failed to deserialize token account")
}
