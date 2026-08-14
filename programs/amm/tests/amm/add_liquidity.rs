use litesvm::types::{FailedTransactionMetadata, TransactionMetadata};
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;

use crate::{
    amm::instructions::add_liquidity_ix,
    common::{context::TestContext, executor::execute_transaction},
};

pub fn add_liquidity(
    ctx: &mut TestContext,
    provider: &Pubkey,
    mint_a: &Pubkey,
    mint_b: &Pubkey,
    max_amount_a: u64,
    max_amount_b: u64,
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

    let ix = add_liquidity_ix(ctx, mint_a, mint_b, provider, max_amount_a, max_amount_b);

    execute_transaction(&mut ctx.svm, provider, &[signer], &[ix])
}
