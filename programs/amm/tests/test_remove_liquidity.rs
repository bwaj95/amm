use crate::{
    amm::{
        accounts::{ata, mint, pool, protocol_config, token_account},
        fixtures::{
            add_liquidity_as_user, derive_user_pool_accounts, fund_tokens_to_user,
            pool_reserves_snapshot, remove_liquidity_as_user, setup_initialized_pool, swap_as_user,
        },
        mint_tokens::mint_tokens,
        pdas::{
            find_locked_lp_token_pda, find_lp_mint_pda, find_mint_pda, find_pool_pda,
            find_protocol_config_pda,
        },
        structs::InitializedPoolStruct,
    },
    common::context::TestContext,
};
use ::amm::{
    self as amm_protocol,
    error::AmmError,
    state::protocol_config,
    utils::{calculate_add_liquidity, calculate_remove_liquidity, calculate_swap},
    MINIMUM_LIQUIDITY,
};
use anchor_spl::associated_token::get_associated_token_address;

mod amm;
mod common;

#[test]
pub fn test_remove_liquidity_success() {
    let program_id = amm_protocol::ID;
    let mut ctx = TestContext::new(program_id);

    let (pool_struct, _) = setup_initialized_pool(
        &mut ctx,
        500_000_000_000_u64,
        10_000_000_000_u64,
        40_000_000_000_u64,
    )
    .unwrap();

    let alice = ctx.alice.pubkey();

    let alice_token_a = get_associated_token_address(&alice, &pool_struct.mint_a);

    let alice_token_b = get_associated_token_address(&alice, &pool_struct.mint_b);

    let alice_lp = get_associated_token_address(&alice, &pool_struct.lp_mint);

    // ------------------------------------------------
    // BEFORE STATE
    // ------------------------------------------------

    let pool_before = pool_reserves_snapshot(&ctx, &pool_struct.pool);

    let alice_token_a_before = token_account(&ctx, &alice_token_a).amount;

    let alice_token_b_before = token_account(&ctx, &alice_token_b).amount;

    let alice_lp_before = token_account(&ctx, &alice_lp).amount;

    let lp_supply_before = mint(&ctx, &pool_struct.lp_mint).supply;

    // Burn half of Alice's LP
    let lp_amount = alice_lp_before / 2;

    let expected = calculate_remove_liquidity(
        pool_before.reserve_a,
        pool_before.reserve_b,
        lp_supply_before,
        lp_amount,
    )
    .unwrap();

    // For first happy path, exact expected amount is fine.
    let min_amount_a = expected.amount_a;

    let min_amount_b = expected.amount_b;

    // ------------------------------------------------
    // ACT
    // ------------------------------------------------

    let pool_after = remove_liquidity_as_user(
        &mut ctx,
        &pool_struct,
        &alice,
        lp_amount,
        min_amount_a,
        min_amount_b,
    )
    .unwrap();

    // ------------------------------------------------
    // AFTER STATE
    // ------------------------------------------------

    let alice_token_a_after = token_account(&ctx, &alice_token_a).amount;

    let alice_token_b_after = token_account(&ctx, &alice_token_b).amount;

    let alice_lp_after = token_account(&ctx, &alice_lp).amount;

    let lp_supply_after = mint(&ctx, &pool_struct.lp_mint).supply;

    // ------------------------------------------------
    // ASSERT - VAULTS
    // ------------------------------------------------

    assert_eq!(
        pool_after.reserve_a,
        pool_before.reserve_a - expected.amount_a
    );

    assert_eq!(
        pool_after.reserve_b,
        pool_before.reserve_b - expected.amount_b
    );

    // ------------------------------------------------
    // ASSERT - USER RECEIVES TOKENS
    // ------------------------------------------------

    assert_eq!(
        alice_token_a_after,
        alice_token_a_before + expected.amount_a
    );

    assert_eq!(
        alice_token_b_after,
        alice_token_b_before + expected.amount_b
    );

    // ------------------------------------------------
    // ASSERT - USER LP BURNED
    // ------------------------------------------------

    assert_eq!(alice_lp_after, alice_lp_before - lp_amount);

    // ------------------------------------------------
    // ASSERT - LP TOTAL SUPPLY DECREASES
    // ------------------------------------------------

    assert_eq!(lp_supply_after, lp_supply_before - lp_amount);

    // ------------------------------------------------
    // LOCKED MINIMUM LIQUIDITY MUST REMAIN
    // ------------------------------------------------

    assert!(lp_supply_after >= MINIMUM_LIQUIDITY);
}
