use litesvm::types::{FailedTransactionMetadata, TransactionMetadata};
use solana_pubkey::Pubkey;

use crate::{
    amm::instructions::add_initial_liquidity_ix,
    common::{context::TestContext, executor::execute_transaction},
};

pub fn add_initial_liquidity(
    ctx: &mut TestContext,
    mint_a: &Pubkey,
    mint_b: &Pubkey,
    amount_token_a: u64,
    amount_token_b: u64,
) -> Result<TransactionMetadata, FailedTransactionMetadata> {
    add_initial_liquidity_with_limits(
        ctx,
        mint_a,
        mint_b,
        amount_token_a,
        amount_token_b,
        0,
        i64::MAX,
    )
}

pub fn add_initial_liquidity_with_limits(
    ctx: &mut TestContext,
    mint_a: &Pubkey,
    mint_b: &Pubkey,
    amount_token_a: u64,
    amount_token_b: u64,
    min_lp_out: u64,
    deadline: i64,
) -> Result<TransactionMetadata, FailedTransactionMetadata> {
    let provider = ctx.alice.pubkey();
    let signer = ctx.alice.signer();

    let ix = add_initial_liquidity_ix(
        ctx,
        mint_a,
        mint_b,
        &provider,
        amount_token_a,
        amount_token_b,
        min_lp_out,
        deadline,
    );

    execute_transaction(&mut ctx.svm, &provider, &[signer], &[ix])
}
