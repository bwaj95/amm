use litesvm::types::{FailedTransactionMetadata, TransactionMetadata};

use crate::{amm::instructions::initialize_mint_ix, common::{context::TestContext, executor::execute_transaction}};

pub fn initialize_mint(ctx: &mut TestContext, mint_id: u64, decimals: u8) -> Result<TransactionMetadata, FailedTransactionMetadata> {
    let ix = initialize_mint_ix(ctx, &ctx.admin.pubkey(), mint_id, decimals);
    let payer = ctx.admin.pubkey();
    let signer = ctx.admin.signer();

    execute_transaction(&mut ctx.svm, &payer, &[&signer], &[ix])
}