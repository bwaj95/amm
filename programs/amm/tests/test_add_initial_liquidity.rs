use ::amm::{
    self as amm_protocol, state::protocol_config, utils::calculate_lp_initial, MINIMUM_LIQUIDITY,
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
        initialize_mint::{self, initialize_mint},
        initialize_protocol::initialize_protocol,
        mint_tokens::mint_tokens,
        pdas::{
            find_locked_lp_token_pda, find_lp_mint_pda, find_mint_pda, find_pool_pda,
            find_protocol_config_pda,
        },
    },
    common::context::TestContext,
};

mod amm;
mod common;

#[test]
fn test_add_initial_liquidity_success() {
    let program_id = amm_protocol::ID;
    let mut ctx = TestContext::new(program_id);

    let res_initialize_protocol = initialize_protocol(&mut ctx);
    assert!(res_initialize_protocol.is_ok());

    let mut mint_id_1 = 1u64;
    let mut mint_id_2 = 2u64;
    let decimals = 9u64;
    let mint_amount = 500_000_000_000_u64;

    let res_init_mint_1 = initialize_mint(&mut ctx, mint_id_1, decimals as u8);
    assert!(res_init_mint_1.is_ok());

    let res_init_mint_2 = initialize_mint(&mut ctx, mint_id_2, decimals as u8);
    assert!(res_init_mint_2.is_ok());

    let (mut mint_1, _) = find_mint_pda(&program_id, mint_id_1);
    let (mut mint_2, _) = find_mint_pda(&program_id, mint_id_2);

    if mint_1 > mint_2 {
        let temp = mint_2;
        mint_2 = mint_1;
        mint_1 = temp;

        mint_id_1 = 2u64;
        mint_id_2 = 1u64;
    }

    let res_create_pool = create_pool(&mut ctx, &mint_1, &mint_2);
    assert!(
        res_create_pool.is_ok(),
        "failed to create pool: {:?}",
        res_create_pool
    );

    let user = ctx.alice.pubkey();
    let (mint_pda_1, _) = find_mint_pda(&program_id, mint_id_1);
    let user_ata_1 = get_associated_token_address(&user, &mint_pda_1);

    let res_mint_tokens_1 = mint_tokens(&mut ctx, &user, &user_ata_1, mint_id_1, mint_amount);
    assert!(
        res_mint_tokens_1.is_ok(),
        "failed res_init_mint_1: {:?}",
        res_mint_tokens_1
    );

    let (mint_pda_2, _) = find_mint_pda(&program_id, mint_id_2);
    let user_ata_2 = get_associated_token_address(&user, &mint_pda_2);

    let res_mint_tokens_2 = mint_tokens(&mut ctx, &user, &user_ata_2, mint_id_2, mint_amount);
    assert!(
        res_mint_tokens_2.is_ok(),
        "failed res_init_mint_2: {:?}",
        res_mint_tokens_2
    );

    let amount_token_a = 10_000_000_000_u64;
    let amount_token_b = 40_000_000_000_u64;

    let res_add_initial_liquidity =
        add_initial_liquidity(&mut ctx, &mint_1, &mint_2, amount_token_a, amount_token_b);
    assert!(
        res_add_initial_liquidity.is_ok(),
        "failed add init liq: {:?}",
        res_add_initial_liquidity
    );

    // ----
    // Assertions
    // ----
    let (pool, _) = find_pool_pda(&program_id, &mint_1, &mint_2);
    let vault_a = get_associated_token_address(&pool, &mint_1);
    let vault_b = get_associated_token_address(&pool, &mint_2);
    let vault_a_account = token_account(&ctx, &vault_a);
    let vault_b_account = token_account(&ctx, &vault_b);
    let user_token_a = token_account(&ctx, &user_ata_1);
    let user_token_b = token_account(&ctx, &user_ata_2);
    let (lp_mint, _) = find_lp_mint_pda(&program_id, &pool);
    let (locked_lp_token, _) = find_locked_lp_token_pda(&program_id, &pool);
    let user_lp_ata = get_associated_token_address(&user, &lp_mint);
    let user_lp_account = token_account(&ctx, &user_lp_ata);
    let locked_lp = token_account(&ctx, &locked_lp_token);
    let lp_mint = mint(&ctx, &lp_mint);

    let expected_lp = calculate_lp_initial(amount_token_a, amount_token_b).unwrap();
    let expected_provider_lp = expected_lp - MINIMUM_LIQUIDITY;

    assert_eq!(vault_a_account.amount, amount_token_a);
    assert_eq!(vault_b_account.amount, amount_token_b);
    assert_eq!(lp_mint.supply, expected_lp);
    assert_eq!(locked_lp.amount, MINIMUM_LIQUIDITY);
    assert_eq!(user_token_a.amount, mint_amount - amount_token_a);
    assert_eq!(user_token_b.amount, mint_amount - amount_token_b);
    assert_eq!(user_lp_account.amount, expected_provider_lp);
}
