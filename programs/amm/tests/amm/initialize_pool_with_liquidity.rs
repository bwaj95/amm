use litesvm::types::{FailedTransactionMetadata, TransactionMetadata};
use solana_pubkey::Pubkey;

use crate::{
    amm::instructions::initialize_pool_with_liquidity_ix,
    common::{context::TestContext, executor::execute_transaction},
};

pub fn initialize_pool_with_liquidity(
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

    let ix = initialize_pool_with_liquidity_ix(
        ctx,
        &provider,
        mint_a,
        mint_b,
        amount_token_a,
        amount_token_b,
        min_lp_out,
        deadline,
    );

    execute_transaction(&mut ctx.svm, &provider, &[signer], &[ix])
}
