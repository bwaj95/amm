use crate::{
    amm::{
        accounts::{ata, mint, pool, protocol_config, token_account},
        fixtures::{
            add_liquidity_as_user, derive_user_pool_accounts, fund_tokens_to_user,
            pool_reserves_snapshot, setup_initialized_pool, swap_as_user,
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
    utils::{calculate_add_liquidity, calculate_swap},
    MINIMUM_LIQUIDITY,
};
use anchor_spl::associated_token::get_associated_token_address;

mod amm;
mod common;

#[test]
pub fn test_swap_a_to_b_success() {
    let program_id = amm_protocol::ID;
    let mut ctx = TestContext::new(program_id);

    let (pool_struct, _) = setup_initialized_pool(
        &mut ctx,
        500_000_000_000_u64,
        10_000_000_000_u64,
        40_000_000_000_u64,
    )
    .unwrap();

    let bob = ctx.bob.pubkey();

    let bob_token_a =
        fund_tokens_to_user(&mut ctx, &bob, pool_struct.mint_a_id, 50_000_000_000_u64);

    // Destination ATA, might not exist yet.
    let bob_token_b = get_associated_token_address(&bob, &pool_struct.mint_b);
    let bob_token_b = get_associated_token_address(&bob, &pool_struct.mint_b);
    let pool_before = pool_reserves_snapshot(&ctx, &pool_struct.pool);
    let bob_a_before = token_account(&ctx, &bob_token_a).amount;
    let treasury_a_before = token_account(&ctx, &pool_struct.treasury_a).amount;

    let protocol_config_account = protocol_config(&ctx);
    let pool_before = pool_reserves_snapshot(&ctx, &pool_struct.pool);
    let amount_in = 1_000_000_000_u64;

    let swap_calculation = calculate_swap(
        pool_before.reserve_a,
        pool_before.reserve_b,
        amount_in,
        protocol_config_account.swap_fee_bps,
        protocol_config_account.treasury_fee_bps,
    )
    .unwrap();

    // let slippage_percentage: u32 = 1;
    // let max_amount_out_percentage: u32 = 100 - slippage_percentage;
    // println!("max_amount_out_percentage: {}", max_amount_out_percentage);

    // let min_amount_out_u128 = (amount_in as u128)
    //     .checked_mul(max_amount_out_percentage as u128)
    //     .ok_or(AmmError::MathOverflow)
    //     .unwrap()
    //     .checked_div(100 as u128)
    //     .ok_or(AmmError::DivisionError)
    //     .unwrap();
    // let min_amount_out = u64::try_from(min_amount_out_u128).unwrap();

    let min_amount_out = swap_calculation.amount_out;

    println!("am out: {:?}", swap_calculation.amount_out);
    println!("min am out: {:?}", min_amount_out);

    let pool_after = swap_as_user(
        &mut ctx,
        &pool_struct,
        &bob,
        amount_in,
        min_amount_out,
        true,
    )
    .unwrap();

    let bob_a_after = token_account(&ctx, &bob_token_a).amount;

    let bob_b_after = token_account(&ctx, &bob_token_b).amount;

    let treasury_a_after = token_account(&ctx, &pool_struct.treasury_a).amount;

    // -----------------------------------
    // User balances
    // -----------------------------------

    assert_eq!(bob_a_after, bob_a_before - amount_in);

    assert_eq!(bob_b_after, swap_calculation.amount_out);

    // -----------------------------------
    // Treasury
    // -----------------------------------

    assert_eq!(treasury_a_after, treasury_a_before + swap_calculation.treasury_fees);

    // -----------------------------------
    // Pool reserves
    // -----------------------------------

    assert_eq!(
        pool_after.reserve_a,
        pool_before.reserve_a + amount_in - swap_calculation.treasury_fees
    );

    assert_eq!(
        pool_after.reserve_b,
        pool_before.reserve_b - swap_calculation.amount_out
    );

    // -----------------------------------
    // Constant-product invariant
    // -----------------------------------

    let k_before = (pool_before.reserve_a as u128) * (pool_before.reserve_b as u128);

    let k_after = (pool_after.reserve_a as u128) * (pool_after.reserve_b as u128);

    assert!(k_after >= k_before, "constant-product invariant decreased");
}
