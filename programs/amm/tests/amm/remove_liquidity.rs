use litesvm::types::{FailedTransactionMetadata, TransactionMetadata};
use solana_pubkey::Pubkey;

use crate::{
    amm::instructions::remove_liquidity_ix,
    common::{context::TestContext, executor::execute_transaction},
};

pub fn remove_liquidity(
    ctx: &mut TestContext,
    provider: &Pubkey,
    mint_a: &Pubkey,
    mint_b: &Pubkey,
    lp_amount: u64,
    min_amount_a: u64,
    min_amount_b: u64,
) -> std::result::Result<TransactionMetadata, FailedTransactionMetadata> {
    let signer = if *provider == ctx.alice.pubkey() {
        ctx.alice.signer()
    } else if *provider == ctx.bob.pubkey() {
        ctx.bob.signer()
    } else if *provider == ctx.charlie.pubkey() {
        ctx.charlie.signer()
    } else {
        panic!("Unknown provider: {:?}", provider);
    };

    let ix = remove_liquidity_ix(
        ctx,
        provider,
        mint_a,
        mint_b,
        lp_amount,
        min_amount_a,
        min_amount_b,
    );

    execute_transaction(&mut ctx.svm, provider, &[signer], &[ix])
}
