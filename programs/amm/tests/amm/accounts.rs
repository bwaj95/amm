use amm::state::{ProtocolConfig, ProtocolTreasury};
use anchor_lang::AccountDeserialize;

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
