use litesvm::types::{FailedTransactionMetadata, TransactionMetadata};
use solana_pubkey::Pubkey;

use crate::{
    amm::instructions::swap_ix,
    common::{context::TestContext, executor::execute_transaction},
};

pub fn swap(
    ctx: &mut TestContext,
    provider: &Pubkey,
    mint_a: &Pubkey,
    mint_b: &Pubkey,
    amount_in: u64,
    min_amount_out: u64,
    a_to_b: bool,
) -> Result<TransactionMetadata, FailedTransactionMetadata> {
    // Get the signer corresponding to the provider
    let signer = if *provider == ctx.alice.pubkey() {
        ctx.alice.signer()
    } else if *provider == ctx.bob.pubkey() {
        ctx.bob.signer()
    } else if *provider == ctx.charlie.pubkey() {
        ctx.charlie.signer()
    } else {
        panic!("Unknown provider: {:?}", provider);
    };

    let ix = swap_ix(
        ctx,
        provider,
        mint_a,
        mint_b,
        amount_in,
        min_amount_out,
        a_to_b,
    );

    execute_transaction(&mut ctx.svm, provider, &[signer], &[ix]) // error here
}
