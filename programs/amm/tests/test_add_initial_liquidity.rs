use ::amm as amm_protocol;

use crate::{
    amm::{
        add_initial_liquidity::add_initial_liquidity,
        fixtures::{pool_reserves_snapshot, setup_initialized_pool},
    },
    common::context::TestContext,
};

mod amm;
mod common;

#[test]
fn add_initial_liquidity_is_deprecated_and_cannot_change_an_initialized_pool() {
    let mut ctx = TestContext::new(amm_protocol::ID);
    let (initialized_pool, _) = setup_initialized_pool(
        &mut ctx,
        500_000_000_000,
        10_000_000_000,
        40_000_000_000,
        0,
        i64::MAX,
    )
    .unwrap();

    let before = pool_reserves_snapshot(&ctx, &initialized_pool.pool);

    let error = add_initial_liquidity(&mut ctx, &initialized_pool, 1, 1).unwrap_err();

    assert!(
        error
            .meta
            .logs
            .iter()
            .any(|log| log.contains("Error Code: DeprecatedInstruction")),
        "expected DeprecatedInstruction, logs: {:?}",
        error.meta.logs
    );

    let after = pool_reserves_snapshot(&ctx, &initialized_pool.pool);
    assert_eq!(after.reserve_a, before.reserve_a);
    assert_eq!(after.reserve_b, before.reserve_b);
    assert_eq!(after.lp_supply, before.lp_supply);
}
