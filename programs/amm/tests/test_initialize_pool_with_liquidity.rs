use ::amm::{
    self as amm_protocol, utils::calculate_lp_initial, MINIMUM_LIQUIDITY,
};
use anchor_spl::{
    associated_token::get_associated_token_address,
    token,
};
use litesvm::types::FailedTransactionMetadata;
use solana_pubkey::Pubkey;

use crate::{
    amm::{
        accounts::{mint, pool, token_account},
        fixtures::setup_initialized_pool,
        initialize_mint::initialize_mint,
        initialize_pool_with_liquidity::initialize_pool_with_liquidity,
        initialize_protocol::initialize_protocol,
        mint_tokens::mint_tokens,
        pdas::{
            find_locked_lp_token_pda, find_lp_mint_pda, find_mint_pda, find_pool_pda,
            find_protocol_treasury_pda, find_vault_a_pda, find_vault_b_pda,
        },
    },
    common::context::TestContext,
};

mod amm;
mod common;

const USER_MINT_AMOUNT: u64 = 500_000_000_000;
const INITIAL_AMOUNT_A: u64 = 10_000_000_000;
const INITIAL_AMOUNT_B: u64 = 40_000_000_000;

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

fn setup_initialization_inputs(ctx: &mut TestContext) -> (Pubkey, Pubkey) {
    initialize_protocol(ctx).unwrap();
    initialize_mint(ctx, 1, 9).unwrap();
    initialize_mint(ctx, 2, 9).unwrap();

    let (mint_1, _) = find_mint_pda(&ctx.program_id, 1);
    let (mint_2, _) = find_mint_pda(&ctx.program_id, 2);
    let alice = ctx.alice.pubkey();

    let alice_token_1 = get_associated_token_address(&alice, &mint_1);
    let alice_token_2 = get_associated_token_address(&alice, &mint_2);
    mint_tokens(ctx, &alice, &alice_token_1, 1, USER_MINT_AMOUNT).unwrap();
    mint_tokens(ctx, &alice, &alice_token_2, 2, USER_MINT_AMOUNT).unwrap();

    if mint_1 < mint_2 {
        (mint_1, mint_2)
    } else {
        (mint_2, mint_1)
    }
}

#[test]
fn initializes_pool_and_liquidity_atomically_with_pda_vaults() {
    let mut ctx = TestContext::new(amm_protocol::ID);
    let (initialized_pool, mints_swapped) = setup_initialized_pool(
        &mut ctx,
        USER_MINT_AMOUNT,
        INITIAL_AMOUNT_A,
        INITIAL_AMOUNT_B,
        0,
        i64::MAX,
    )
    .unwrap();

    let expected_amount_a = if mints_swapped {
        INITIAL_AMOUNT_B
    } else {
        INITIAL_AMOUNT_A
    };
    let expected_amount_b = if mints_swapped {
        INITIAL_AMOUNT_A
    } else {
        INITIAL_AMOUNT_B
    };
    let expected_lp = calculate_lp_initial(expected_amount_a, expected_amount_b).unwrap();
    let alice = ctx.alice.pubkey();
    let alice_lp = get_associated_token_address(&alice, &initialized_pool.lp_mint);

    let pool_account = pool(&ctx, &initialized_pool.pool);
    let vault_a_account = token_account(&ctx, &initialized_pool.vault_a);
    let vault_b_account = token_account(&ctx, &initialized_pool.vault_b);
    let treasury_a_account = token_account(&ctx, &initialized_pool.treasury_a);
    let treasury_b_account = token_account(&ctx, &initialized_pool.treasury_b);
    let locked_lp_account = token_account(&ctx, &initialized_pool.locked_lp_token);
    let alice_lp_account = token_account(&ctx, &alice_lp);
    let lp_mint_account = mint(&ctx, &initialized_pool.lp_mint);

    assert_eq!(pool_account.vault_a, initialized_pool.vault_a);
    assert_eq!(pool_account.vault_b, initialized_pool.vault_b);
    assert_eq!(vault_a_account.mint, initialized_pool.mint_a);
    assert_eq!(vault_b_account.mint, initialized_pool.mint_b);
    assert_eq!(vault_a_account.owner, initialized_pool.pool);
    assert_eq!(vault_b_account.owner, initialized_pool.pool);
    assert_eq!(vault_a_account.amount, expected_amount_a);
    assert_eq!(vault_b_account.amount, expected_amount_b);
    assert_eq!(treasury_a_account.mint, initialized_pool.mint_a);
    assert_eq!(treasury_b_account.mint, initialized_pool.mint_b);
    assert_eq!(locked_lp_account.amount, MINIMUM_LIQUIDITY);
    assert_eq!(alice_lp_account.amount, expected_lp - MINIMUM_LIQUIDITY);
    assert_eq!(lp_mint_account.supply, expected_lp);

    assert_ne!(
        initialized_pool.vault_a,
        get_associated_token_address(&initialized_pool.pool, &initialized_pool.mint_a)
    );
    assert_ne!(
        initialized_pool.vault_b,
        get_associated_token_address(&initialized_pool.pool, &initialized_pool.mint_b)
    );
    assert_eq!(
        ctx.svm.get_account(&initialized_pool.vault_a).unwrap().owner,
        token::ID
    );
    assert_eq!(
        ctx.svm.get_account(&initialized_pool.vault_b).unwrap().owner,
        token::ID
    );
}

#[test]
fn slippage_failure_rolls_back_every_account_and_token_transfer() {
    let mut ctx = TestContext::new(amm_protocol::ID);
    let (mint_a, mint_b) = setup_initialization_inputs(&mut ctx);
    let alice = ctx.alice.pubkey();
    let alice_token_a = get_associated_token_address(&alice, &mint_a);
    let alice_token_b = get_associated_token_address(&alice, &mint_b);
    let balance_a_before = token_account(&ctx, &alice_token_a).amount;
    let balance_b_before = token_account(&ctx, &alice_token_b).amount;
    let quoted_provider_lp = calculate_lp_initial(INITIAL_AMOUNT_A, INITIAL_AMOUNT_B)
        .unwrap()
        - MINIMUM_LIQUIDITY;

    let error = initialize_pool_with_liquidity(
        &mut ctx,
        &mint_a,
        &mint_b,
        INITIAL_AMOUNT_A,
        INITIAL_AMOUNT_B,
        quoted_provider_lp + 1,
        i64::MAX,
    )
    .unwrap_err();

    assert_anchor_error(error, "SlippageExceeded");
    assert_eq!(token_account(&ctx, &alice_token_a).amount, balance_a_before);
    assert_eq!(token_account(&ctx, &alice_token_b).amount, balance_b_before);

    let (pool, _) = find_pool_pda(&ctx.program_id, &mint_a, &mint_b);
    let (vault_a, _) = find_vault_a_pda(&ctx.program_id, &pool);
    let (vault_b, _) = find_vault_b_pda(&ctx.program_id, &pool);
    let (lp_mint, _) = find_lp_mint_pda(&ctx.program_id, &pool);
    let (locked_lp, _) = find_locked_lp_token_pda(&ctx.program_id, &pool);
    for address in [pool, vault_a, vault_b, lp_mint, locked_lp] {
        assert!(
            ctx.svm.get_account(&address).is_none(),
            "failed atomic initialization left account {address} behind"
        );
    }
}

#[test]
fn expired_initialization_rolls_back() {
    let mut ctx = TestContext::new(amm_protocol::ID);
    let (mint_a, mint_b) = setup_initialization_inputs(&mut ctx);
    let (pool, _) = find_pool_pda(&ctx.program_id, &mint_a, &mint_b);

    let error = initialize_pool_with_liquidity(
        &mut ctx,
        &mint_a,
        &mint_b,
        INITIAL_AMOUNT_A,
        INITIAL_AMOUNT_B,
        0,
        -1,
    )
    .unwrap_err();

    assert_anchor_error(error, "DeadlineExceeded");
    assert!(ctx.svm.get_account(&pool).is_none());
}

#[test]
fn noncanonical_mint_order_is_rejected() {
    let mut ctx = TestContext::new(amm_protocol::ID);
    let (mint_a, mint_b) = setup_initialization_inputs(&mut ctx);
    let (wrong_order_pool, _) = find_pool_pda(&ctx.program_id, &mint_b, &mint_a);

    let error = initialize_pool_with_liquidity(
        &mut ctx,
        &mint_b,
        &mint_a,
        INITIAL_AMOUNT_A,
        INITIAL_AMOUNT_B,
        0,
        i64::MAX,
    )
    .unwrap_err();

    assert_anchor_error(error, "InvalidMintOrder");
    assert!(ctx.svm.get_account(&wrong_order_pool).is_none());
}

#[test]
fn prefunded_vault_pda_can_still_be_initialized_as_a_token_account() {
    let mut ctx = TestContext::new(amm_protocol::ID);
    let (mint_a, mint_b) = setup_initialization_inputs(&mut ctx);
    let (pool, _) = find_pool_pda(&ctx.program_id, &mint_a, &mint_b);
    let (vault_a, _) = find_vault_a_pda(&ctx.program_id, &pool);

    ctx.svm.airdrop(&vault_a, 1).unwrap();
    assert_ne!(ctx.svm.get_account(&vault_a).unwrap().owner, token::ID);

    initialize_pool_with_liquidity(
        &mut ctx,
        &mint_a,
        &mint_b,
        INITIAL_AMOUNT_A,
        INITIAL_AMOUNT_B,
        0,
        i64::MAX,
    )
    .unwrap();

    let vault_account = token_account(&ctx, &vault_a);
    assert_eq!(vault_account.mint, mint_a);
    assert_eq!(vault_account.owner, pool);
    assert_eq!(vault_account.amount, INITIAL_AMOUNT_A);
    assert_eq!(ctx.svm.get_account(&vault_a).unwrap().owner, token::ID);

    let (protocol_treasury, _) = find_protocol_treasury_pda(&ctx.program_id);
    let treasury_b = get_associated_token_address(&protocol_treasury, &mint_b);
    assert_eq!(token_account(&ctx, &treasury_b).mint, mint_b);
}
