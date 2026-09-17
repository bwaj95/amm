use ::amm::{self as amm_protocol, utils::calculate_swap};
use litesvm::types::FailedTransactionMetadata;

use crate::{
    amm::{
        accounts::{protocol_config, token_account},
        add_liquidity::add_liquidity_with_limits,
        fixtures::{
            fund_tokens_to_user, pool_reserves_snapshot, setup_initialized_pool,
        },
        remove_liquidity::remove_liquidity_with_deadline,
        swap::swap_with_deadline,
    },
    common::context::TestContext,
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
fn reverse_swap_preserves_invariant_and_collects_the_correct_treasury_token() {
    let mut ctx = TestContext::new(amm_protocol::ID);
    let (pool, _) = setup_initialized_pool(
        &mut ctx,
        500_000_000_000,
        10_000_000_000,
        40_000_000_000,
    )
    .unwrap();

    let bob = ctx.bob.pubkey();
    let amount_in = 1_000_000_000;
    fund_tokens_to_user(&mut ctx, &bob, pool.mint_b_id, amount_in);

    let before = pool_reserves_snapshot(&ctx, &pool.pool);
    let treasury_before = token_account(&ctx, &pool.treasury_b).amount;
    let config = protocol_config(&ctx);
    let quote = calculate_swap(
        before.reserve_b,
        before.reserve_a,
        amount_in,
        config.swap_fee_bps,
        config.treasury_fee_bps,
    )
    .unwrap();

    swap_with_deadline(
        &mut ctx,
        &bob,
        &pool.mint_a,
        &pool.mint_b,
        amount_in,
        quote.amount_out,
        false,
        i64::MAX,
    )
    .unwrap();

    let after = pool_reserves_snapshot(&ctx, &pool.pool);
    let treasury_after = token_account(&ctx, &pool.treasury_b).amount;

    assert_eq!(
        after.reserve_b,
        before.reserve_b + amount_in - quote.treasury_fees
    );
    assert_eq!(after.reserve_a, before.reserve_a - quote.amount_out);
    assert_eq!(treasury_after, treasury_before + quote.treasury_fees);

    let k_before = (before.reserve_a as u128) * (before.reserve_b as u128);
    let k_after = (after.reserve_a as u128) * (after.reserve_b as u128);
    assert!(k_after >= k_before);
}

#[test]
fn expired_swap_is_rejected_before_funds_move() {
    let mut ctx = TestContext::new(amm_protocol::ID);
    let (pool, _) = setup_initialized_pool(
        &mut ctx,
        500_000_000_000,
        10_000_000_000,
        40_000_000_000,
    )
    .unwrap();

    let bob = ctx.bob.pubkey();
    fund_tokens_to_user(&mut ctx, &bob, pool.mint_a_id, 1_000_000_000);
    let before = pool_reserves_snapshot(&ctx, &pool.pool);

    let error = swap_with_deadline(
        &mut ctx,
        &bob,
        &pool.mint_a,
        &pool.mint_b,
        1_000_000,
        0,
        true,
        -1,
    )
    .unwrap_err();

    assert_anchor_error(error, "DeadlineExceeded");
    let after = pool_reserves_snapshot(&ctx, &pool.pool);
    assert_eq!(after.reserve_a, before.reserve_a);
    assert_eq!(after.reserve_b, before.reserve_b);
}

#[test]
fn swap_rejects_output_below_the_users_limit() {
    let mut ctx = TestContext::new(amm_protocol::ID);
    let (pool, _) = setup_initialized_pool(
        &mut ctx,
        500_000_000_000,
        10_000_000_000,
        40_000_000_000,
    )
    .unwrap();

    let bob = ctx.bob.pubkey();
    let amount_in = 1_000_000;
    fund_tokens_to_user(&mut ctx, &bob, pool.mint_a_id, amount_in);

    let reserves = pool_reserves_snapshot(&ctx, &pool.pool);
    let config = protocol_config(&ctx);
    let quote = calculate_swap(
        reserves.reserve_a,
        reserves.reserve_b,
        amount_in,
        config.swap_fee_bps,
        config.treasury_fee_bps,
    )
    .unwrap();

    let error = swap_with_deadline(
        &mut ctx,
        &bob,
        &pool.mint_a,
        &pool.mint_b,
        amount_in,
        quote.amount_out + 1,
        true,
        i64::MAX,
    )
    .unwrap_err();

    assert_anchor_error(error, "SlippageExceeded");
}

#[test]
fn liquidity_provider_can_enforce_minimum_lp_output() {
    let mut ctx = TestContext::new(amm_protocol::ID);
    let (pool, _) = setup_initialized_pool(
        &mut ctx,
        500_000_000_000,
        10_000_000_000,
        40_000_000_000,
    )
    .unwrap();

    let bob = ctx.bob.pubkey();
    fund_tokens_to_user(&mut ctx, &bob, pool.mint_a_id, 10_000_000_000);
    fund_tokens_to_user(&mut ctx, &bob, pool.mint_b_id, 40_000_000_000);

    let error = add_liquidity_with_limits(
        &mut ctx,
        &bob,
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
fn expired_withdrawal_is_rejected() {
    let mut ctx = TestContext::new(amm_protocol::ID);
    let (pool, _) = setup_initialized_pool(
        &mut ctx,
        500_000_000_000,
        10_000_000_000,
        40_000_000_000,
    )
    .unwrap();

    let alice = ctx.alice.pubkey();
    let error = remove_liquidity_with_deadline(
        &mut ctx,
        &alice,
        &pool.mint_a,
        &pool.mint_b,
        1,
        0,
        0,
        -1,
    )
    .unwrap_err();

    assert_anchor_error(error, "DeadlineExceeded");
}
