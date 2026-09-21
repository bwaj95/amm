use ::amm as amm_protocol;

use crate::{
    amm::{
        create_pool::create_pool,
        initialize_mint::initialize_mint,
        initialize_protocol::initialize_protocol,
        pdas::{find_mint_pda, find_pool_pda},
    },
    common::context::TestContext,
};

mod amm;
mod common;

#[test]
fn create_pool_is_deprecated_and_rolls_back_created_accounts() {
    let mut ctx = TestContext::new(amm_protocol::ID);
    initialize_protocol(&mut ctx).unwrap();
    initialize_mint(&mut ctx, 1, 9).unwrap();
    initialize_mint(&mut ctx, 2, 9).unwrap();

    let (mint_1, _) = find_mint_pda(&ctx.program_id, 1);
    let (mint_2, _) = find_mint_pda(&ctx.program_id, 2);
    let (mint_a, mint_b) = if mint_1 < mint_2 {
        (mint_1, mint_2)
    } else {
        (mint_2, mint_1)
    };
    let (pool, _) = find_pool_pda(&ctx.program_id, &mint_a, &mint_b);

    let error = create_pool(&mut ctx, &mint_a, &mint_b).unwrap_err();

    assert!(
        error
            .meta
            .logs
            .iter()
            .any(|log| log.contains("Error Code: DeprecatedInstruction")),
        "expected DeprecatedInstruction, logs: {:?}",
        error.meta.logs
    );
    assert!(
        ctx.svm.get_account(&pool).is_none(),
        "failed instruction must roll back the newly created pool"
    );
}
