use ::amm::{
    self as amm_protocol,
    state::protocol_config,
    utils::{calculate_add_liquidity, calculate_lp_initial},
    MINIMUM_LIQUIDITY,
};
use anchor_spl::associated_token::{
    get_associated_token_address,
    spl_associated_token_account::address::get_associated_token_address_and_bump_seed,
};

use crate::{
    amm::{
        accounts::{ata, mint, pool, token_account},
        add_initial_liquidity::add_initial_liquidity,
        create_pool::create_pool,
        fixtures::{
            add_liquidity_as_user, derive_user_pool_accounts, fund_tokens_to_user,
            pool_reserves_snapshot, setup_initialized_pool,
        },
        initialize_mint::{self, initialize_mint},
        initialize_protocol::initialize_protocol,
        mint_tokens::mint_tokens,
        pdas::{
            find_locked_lp_token_pda, find_lp_mint_pda, find_mint_pda, find_pool_pda,
            find_protocol_config_pda,
        },
        structs::InitializedPoolStruct,
    },
    common::context::TestContext,
};

mod amm;
mod common;

#[test]
pub fn test_add_balanced_liquidity_success() {
    let program_id = amm_protocol::ID;
    let mut ctx = TestContext::new(program_id);

    let pool_struct: InitializedPoolStruct = setup_initialized_pool(
        &mut ctx,
        500_000_000_000_u64,
        10_000_000_000_u64,
        40_000_000_000_u64,
    )
    .unwrap();

    // init liq added by alice
    //  fund bob
    let bob = ctx.bob.pubkey();
    let bob_token_a =
        fund_tokens_to_user(&mut ctx, &bob, pool_struct.mint_a_id, 50_000_000_000_u64);
    let bob_token_b =
        fund_tokens_to_user(&mut ctx, &bob, pool_struct.mint_b_id, 50_000_000_000_u64);
    let bob_lp = get_associated_token_address(&bob, &pool_struct.lp_mint);
    // call add liq using bob
    let bob_max_amount_a: u64 = 10_000_000_000_u64;
    let bob_max_amount_b: u64 = 40_000_000_000_u64;

    let bob_token_a_before = token_account(&ctx, &bob_token_a).amount;
    let bob_token_b_before = token_account(&ctx, &bob_token_b).amount;
    let lp_mint_before = mint(&ctx, &pool_struct.lp_mint).supply;
    let pool_before = pool_reserves_snapshot(&ctx, &pool_struct.pool);

    let pool_after = add_liquidity_as_user(
        &mut ctx,
        &pool_struct,
        &bob,
        bob_max_amount_a,
        bob_max_amount_b,
    )
    .unwrap();

    assert_eq!(
        pool_after.reserve_a,
        pool_before.reserve_a + bob_max_amount_a
    );
    assert_eq!(
        pool_after.reserve_b,
        pool_before.reserve_b + bob_max_amount_b
    );

    let bob_token_a_after = token_account(&ctx, &bob_token_a).amount;
    let bob_token_b_after = token_account(&ctx, &bob_token_b).amount;
    let bob_lp_after = token_account(&ctx, &bob_lp).amount;
    let lp_mint_after = mint(&ctx, &pool_struct.lp_mint).supply;
    let (_, _, expected_lp) = calculate_add_liquidity(
        pool_before.reserve_a,
        pool_before.reserve_b,
        bob_max_amount_a,
        bob_max_amount_b,
        lp_mint_before,
    )
    .unwrap();

    assert_eq!(bob_token_a_after, bob_token_a_before - bob_max_amount_a);
    assert_eq!(bob_token_b_after, bob_token_b_before - bob_max_amount_b);
    assert_eq!(bob_lp_after, expected_lp);
    assert_eq!(lp_mint_after, lp_mint_before + expected_lp);

    let alice_lp = get_associated_token_address(&ctx.alice.pubkey(), &pool_struct.lp_mint);
    let alice_lp_account = token_account(&ctx, &alice_lp);
    assert_eq!(
        lp_mint_after,
        MINIMUM_LIQUIDITY + alice_lp_account.amount + bob_lp_after
    );
}

// #[test]
// pub fn test_add_unbalanced_liquidity_success() {}
