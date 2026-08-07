use litesvm::types::{FailedTransactionMetadata, TransactionMetadata};
use solana_pubkey::Pubkey;

use crate::{
    amm::instructions::create_pool_ix,
    common::{context::TestContext, executor::execute_transaction},
};

pub fn create_pool(
    ctx: &mut TestContext,
    mint_a: &Pubkey,
    mint_b: &Pubkey,
) -> Result<TransactionMetadata, FailedTransactionMetadata> {
    let creator = ctx.alice.pubkey();
    let signer = ctx.alice.signer();

    let ix = create_pool_ix(ctx, &creator, mint_a, mint_b);

    execute_transaction(&mut ctx.svm, &creator, &[signer], &[ix])
}
