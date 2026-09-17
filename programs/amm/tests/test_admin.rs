use ::amm as amm_protocol;
use anchor_spl::associated_token::get_associated_token_address;
use litesvm::types::FailedTransactionMetadata;

use crate::{
    amm::{
        accounts::{mint, protocol_config, token_account},
        add_liquidity::add_liquidity,
        admin::{
            accept_admin, cancel_admin_transfer, propose_admin, set_paused, update_fees,
        },
        create_pool::create_pool,
        fixtures::{fund_tokens_to_user, remove_liquidity_as_user, setup_initialized_pool},
        initialize_protocol::initialize_protocol,
        initialize_mint::initialize_mint,
        pdas::find_mint_pda,
        swap::swap,
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
fn only_admin_can_pause_and_update_fees() {
    let mut ctx = TestContext::new(amm_protocol::ID);
    initialize_protocol(&mut ctx).unwrap();

    let admin = ctx.admin.pubkey();
    let bob = ctx.bob.pubkey();

    let error = update_fees(&mut ctx, &admin, 40, 10).unwrap_err();
    assert_anchor_error(error, "ProtocolMustBePaused");

    let error = set_paused(&mut ctx, &bob, true).unwrap_err();
    assert_anchor_error(error, "UnauthorizedAdmin");

    set_paused(&mut ctx, &admin, true).unwrap();
    assert!(protocol_config(&ctx).paused);

    update_fees(&mut ctx, &admin, 40, 10).unwrap();
    let config = protocol_config(&ctx);
    assert_eq!(config.swap_fee_bps, 40);
    assert_eq!(config.treasury_fee_bps, 10);

    let error = update_fees(&mut ctx, &admin, 30, 31).unwrap_err();
    assert_anchor_error(error, "InvalidTreasuryFee");

    set_paused(&mut ctx, &admin, false).unwrap();
    assert!(!protocol_config(&ctx).paused);
}

#[test]
fn admin_transfer_requires_proposal_and_acceptance() {
    let mut ctx = TestContext::new(amm_protocol::ID);
    initialize_protocol(&mut ctx).unwrap();

    let original_admin = ctx.admin.pubkey();
    let alice = ctx.alice.pubkey();
    let bob = ctx.bob.pubkey();

    propose_admin(&mut ctx, &original_admin, bob).unwrap();
    assert_eq!(protocol_config(&ctx).pending_admin, Some(bob));

    let error = accept_admin(&mut ctx, &alice).unwrap_err();
    assert_anchor_error(error, "UnauthorizedPendingAdmin");

    accept_admin(&mut ctx, &bob).unwrap();
    let config = protocol_config(&ctx);
    assert_eq!(config.admin, bob);
    assert_eq!(config.pending_admin, None);

    let error = set_paused(&mut ctx, &original_admin, true).unwrap_err();
    assert_anchor_error(error, "UnauthorizedAdmin");

    set_paused(&mut ctx, &bob, true).unwrap();
    assert!(protocol_config(&ctx).paused);
}

#[test]
fn current_admin_can_cancel_pending_transfer() {
    let mut ctx = TestContext::new(amm_protocol::ID);
    initialize_protocol(&mut ctx).unwrap();

    let admin = ctx.admin.pubkey();
    let bob = ctx.bob.pubkey();

    propose_admin(&mut ctx, &admin, bob).unwrap();
    cancel_admin_transfer(&mut ctx, &admin).unwrap();

    assert_eq!(protocol_config(&ctx).pending_admin, None);

    let error = accept_admin(&mut ctx, &bob).unwrap_err();
    assert_anchor_error(error, "NoPendingAdmin");
}

#[test]
fn pause_blocks_swaps_but_preserves_the_liquidity_exit_path() {
    let mut ctx = TestContext::new(amm_protocol::ID);
    let (pool, _) = setup_initialized_pool(
        &mut ctx,
        500_000_000_000,
        10_000_000_000,
        40_000_000_000,
    )
    .unwrap();

    let admin = ctx.admin.pubkey();
    let bob = ctx.bob.pubkey();
    let alice = ctx.alice.pubkey();

    fund_tokens_to_user(&mut ctx, &bob, pool.mint_a_id, 1_000_000_000);
    fund_tokens_to_user(&mut ctx, &bob, pool.mint_b_id, 1_000_000_000);
    set_paused(&mut ctx, &admin, true).unwrap();

    let error = add_liquidity(
        &mut ctx,
        &bob,
        &pool.mint_a,
        &pool.mint_b,
        1_000_000,
        1_000_000,
    )
    .unwrap_err();
    assert_anchor_error(error, "ProtocolPaused");

    let error = swap(
        &mut ctx,
        &bob,
        &pool.mint_a,
        &pool.mint_b,
        1_000_000,
        0,
        true,
    )
    .unwrap_err();
    assert_anchor_error(error, "ProtocolPaused");

    let alice_lp = get_associated_token_address(&alice, &pool.lp_mint);
    let lp_amount = token_account(&ctx, &alice_lp).amount / 2;
    let supply_before = mint(&ctx, &pool.lp_mint).supply;

    remove_liquidity_as_user(&mut ctx, &pool, &alice, lp_amount, 0, 0).unwrap();

    assert_eq!(mint(&ctx, &pool.lp_mint).supply, supply_before - lp_amount);
}

#[test]
fn pause_blocks_new_pool_creation() {
    let mut ctx = TestContext::new(amm_protocol::ID);
    initialize_protocol(&mut ctx).unwrap();
    initialize_mint(&mut ctx, 1, 9).unwrap();
    initialize_mint(&mut ctx, 2, 9).unwrap();

    let mut mint_a = find_mint_pda(&ctx.program_id, 1).0;
    let mut mint_b = find_mint_pda(&ctx.program_id, 2).0;
    if mint_a > mint_b {
        std::mem::swap(&mut mint_a, &mut mint_b);
    }

    let admin = ctx.admin.pubkey();
    set_paused(&mut ctx, &admin, true).unwrap();

    let error = create_pool(&mut ctx, &mint_a, &mint_b).unwrap_err();
    assert_anchor_error(error, "ProtocolPaused");
}
