use ::amm::{self as amm_protocol, error::AmmError, LP_MINT_DECIMALS};
use anchor_spl::associated_token::get_associated_token_address;
use solana_pubkey::Pubkey;

use crate::{
    amm::{
        accounts::{ata, mint, pool},
        create_pool::create_pool,
        initialize_mint,
        initialize_protocol::initialize_protocol,
        pdas::{find_lp_mint_pda, find_mint_pda, find_pool_pda},
    },
    common::context::TestContext,
};

mod amm;
mod common;

#[test]
fn create_pool_success() {
    let program_id = amm_protocol::id();
    let mut ctx = TestContext::new(program_id);

    let res = initialize_protocol(&mut ctx);
    assert!(res.is_ok());

    let init_mint_1 = initialize_mint::initialize_mint(&mut ctx, 1, 6);
    assert!(init_mint_1.is_ok());

    let init_mint_2 = initialize_mint::initialize_mint(&mut ctx, 2, 9);
    assert!(init_mint_2.is_ok());

    let (mut mint_a, _) = find_mint_pda(&program_id, 1u64);
    let (mut mint_b, _) = find_mint_pda(&program_id, 2u64);

    if mint_a > mint_b {
        let temp = mint_a;
        mint_a = mint_b;
        mint_b = temp;
    }

    let res = create_pool(&mut ctx, &mint_a, &mint_b);

    match &res {
        Ok(_) => {}
        Err(err) => {
            println!("create_pool failed: {:?}", err);
        }
    }

    assert!(res.is_ok(), "create_pool failed: {:?}", res);

    let (pool_pda, pool_pda_bump) = find_pool_pda(&program_id, &mint_a, &mint_b);
    let pool_account = pool(&ctx, &pool_pda);
    let (lp_mint_pda, _) = find_lp_mint_pda(&program_id, &pool_pda);
    let lp_mint_account = mint(&ctx, &lp_mint_pda);

    let vault_a_ata = get_associated_token_address(&pool_pda, &mint_a);
    let vault_a_account = ata(&ctx, &vault_a_ata);
    let vault_b_ata = get_associated_token_address(&pool_pda, &mint_b);
    let vault_b_account = ata(&ctx, &vault_b_ata);

    assert_eq!(pool_account.mint_a, mint_a);
    assert_eq!(pool_account.mint_b, mint_b);
    assert_eq!(pool_account.vault_a, vault_a_ata);
    assert_eq!(pool_account.vault_b, vault_b_ata);
    assert_eq!(pool_account.lp_mint, lp_mint_pda);
    assert_eq!(pool_account.bump, pool_pda_bump);

    assert_eq!(lp_mint_account.mint_authority, Some(pool_pda).into());
    assert_eq!(lp_mint_account.decimals, LP_MINT_DECIMALS);

    assert_eq!(vault_a_account.owner, pool_pda);
    assert_eq!(vault_b_account.owner, pool_pda);
}

#[test]
fn create_pool_fails_with_invalid_mint_order() {
    let program_id = amm_protocol::id();
    let mut ctx = TestContext::new(program_id);

    // Protocol must exist before creating a pool.
    initialize_protocol(&mut ctx).unwrap();

    initialize_mint::initialize_mint(&mut ctx, 1, 6).unwrap();
    initialize_mint::initialize_mint(&mut ctx, 2, 9).unwrap();

    let (mint_1, _) = find_mint_pda(&program_id, 1);
    let (mint_2, _) = find_mint_pda(&program_id, 2);

    let (mint_a, mint_b) = canonical_mint_order(mint_1, mint_2);

    let (wrong_a, wrong_b) = (mint_b, mint_a);

    let result = create_pool(&mut ctx, &wrong_a, &wrong_b);

    // The transaction must fail.
    assert!(result.is_err());

    let error = result.unwrap_err();

    assert!(error
        .meta
        .logs
        .iter()
        .any(|log| log.contains("Error Code: InvalidMintOrder")));
}

pub fn canonical_mint_order(a: Pubkey, b: Pubkey) -> (Pubkey, Pubkey) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}
