use ::amm as amm_protocol;

use crate::{
    amm::{
        accounts::{protocol_config, protocol_treasury},
        initialize_protocol,
        pdas::{find_protocol_config_pda, find_protocol_treasury_pda},
    },
    common::context::TestContext,
};

mod amm;
mod common;

#[test]
fn initialize_protocol_success() {
    let program_id = amm_protocol::id();
    let mut ctx = TestContext::new(program_id);

    let res = initialize_protocol::initialize_protocol(&mut ctx);

    assert!(res.is_ok());

    let config: ::amm::state::ProtocolConfig = protocol_config(&ctx);

    assert_eq!(config.admin, ctx.admin.pubkey());
    assert_eq!(config.pending_admin, None);
    assert_eq!(
        config.protocol_treasury,
        find_protocol_treasury_pda(&program_id).0
    );
    assert_eq!(config.swap_fee_bps, 30);
    assert_eq!(config.treasury_fee_bps, 5);
    assert!(!config.paused);
    assert_eq!(config.bump, find_protocol_config_pda(&program_id).1);

    let treasury: ::amm::state::ProtocolTreasury = protocol_treasury(&ctx);
    assert_eq!(treasury.bump, find_protocol_treasury_pda(&program_id).1);
}
