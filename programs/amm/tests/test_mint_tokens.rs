use ::amm as amm_protocol;
use anchor_spl::associated_token::get_associated_token_address;

use crate::{
    amm::{
        accounts::{mint, token_account},
        initialize_mint,
        initialize_protocol::{self, initialize_protocol},
        mint_tokens,
        pdas::find_mint_pda,
    },
    common::context::TestContext,
};

mod amm;
mod common;

#[test]
fn test_mint_tokens_success() {
    let program_id = amm_protocol::ID;
    let mut ctx = TestContext::new(program_id);

    initialize_protocol(&mut ctx).unwrap();

    let mint_id = 1u64;
    let decimals = 6u8;

    initialize_mint::initialize_mint(&mut ctx, mint_id, decimals).unwrap();

    let (mint_pda, _) = find_mint_pda(&program_id, mint_id);
    let user = ctx.alice.pubkey();
    let user_ata = get_associated_token_address(&user, &mint_pda);
    let amount = 100_000_000 as u64;

    let res = mint_tokens::mint_tokens(&mut ctx, &user, &user_ata, mint_id, amount);

    if let Err(err) = &res {
        println!("mint_tokens failed: {:?}", err);
    }

    assert!(res.is_ok(), "mint_tokens failed: {:?}", res);

    let mint_account = mint(&ctx, &mint_pda);

    let user_token_account = token_account(&ctx, &user_ata);

    assert_eq!(user_token_account.mint, mint_pda);

    assert_eq!(user_token_account.owner, user);

    assert_eq!(user_token_account.amount, amount);

    assert_eq!(mint_account.supply, amount);
}
