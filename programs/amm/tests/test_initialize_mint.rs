use crate::{
    amm::{
        accounts::mint,
        initialize_mint,
        initialize_protocol::initialize_protocol,
        pdas::{find_mint_pda, find_protocol_config_pda},
    },
    common::context::TestContext,
};

use ::amm as amm_protocol;
mod amm;
mod common;

#[test]
pub fn test_initilize_mint_success() {
    let program_id = amm_protocol::id();
    let mut ctx = TestContext::new(program_id);

    let res = initialize_protocol(&mut ctx);
    assert!(res.is_ok());

    let res = initialize_mint::initialize_mint(&mut ctx, 1u64, 6);

    match &res {
        Ok(_) => {}
        Err(err) => {
            println!("init_mint failed failed: {:?}", err);
        }
    }

    assert!(res.is_ok());

    let (mint_pda, _) = find_mint_pda(&program_id, 1u64);
    let mint = mint(&ctx, &mint_pda);

    assert_eq!(
        mint.mint_authority,
        Some(find_protocol_config_pda(&program_id).0).into()
    );
    assert_eq!(mint.decimals, 6)
}
