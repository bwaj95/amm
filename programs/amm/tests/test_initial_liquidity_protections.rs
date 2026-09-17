use ::amm as amm_protocol;
use anchor_spl::{
    associated_token::get_associated_token_address,
    token::{self, spl_token},
};
use litesvm::types::FailedTransactionMetadata;

use crate::{
    amm::{
        add_initial_liquidity::add_initial_liquidity_with_limits,
        fixtures::setup_empty_pool,
    },
    common::{context::TestContext, executor::execute_transaction},
};

mod amm;
mod common;

fn assert_anchor_error(error: FailedTransactionMetadata, expected_code: &str) {
    assert!(
        error
            .meta
            .logs
            .iter()
            .any(|log| log.contains(&format!("Error Code: {expected_code}"))),
        "expected Anchor error {expected_code}, logs: {:?}",
        error.meta.logs
    );
}

#[test]
fn initial_provider_can_enforce_minimum_lp_output() {
    let mut ctx = TestContext::new(amm_protocol::ID);
    let (pool, _) = setup_empty_pool(&mut ctx, 500_000_000_000).unwrap();

    let error = add_initial_liquidity_with_limits(
        &mut ctx,
        &pool.mint_a,
        &pool.mint_b,
        10_000_000_000,
        40_000_000_000,
        u64::MAX,
        i64::MAX,
    )
    .unwrap_err();

    assert_anchor_error(error, "SlippageExceeded");
}

#[test]
fn expired_initial_deposit_is_rejected() {
    let mut ctx = TestContext::new(amm_protocol::ID);
    let (pool, _) = setup_empty_pool(&mut ctx, 500_000_000_000).unwrap();

    let error = add_initial_liquidity_with_limits(
        &mut ctx,
        &pool.mint_a,
        &pool.mint_b,
        10_000_000_000,
        40_000_000_000,
        0,
        -1,
    )
    .unwrap_err();

    assert_anchor_error(error, "DeadlineExceeded");
}

#[test]
fn pre_funded_vault_cannot_distort_initial_lp_ownership() {
    let mut ctx = TestContext::new(amm_protocol::ID);
    let (pool, _) = setup_empty_pool(&mut ctx, 500_000_000_000).unwrap();

    let alice = ctx.alice.pubkey();
    let alice_token_a = get_associated_token_address(&alice, &pool.mint_a);
    let transfer = spl_token::instruction::transfer_checked(
        &token::ID,
        &alice_token_a,
        &pool.mint_a,
        &pool.vault_a,
        &alice,
        &[],
        1,
        9,
    )
    .unwrap();

    let signer = ctx.alice.signer();
    execute_transaction(&mut ctx.svm, &alice, &[signer], &[transfer]).unwrap();

    let error = add_initial_liquidity_with_limits(
        &mut ctx,
        &pool.mint_a,
        &pool.mint_b,
        10_000_000_000,
        40_000_000_000,
        0,
        i64::MAX,
    )
    .unwrap_err();

    assert_anchor_error(error, "InitialVaultsNotEmpty");
}
