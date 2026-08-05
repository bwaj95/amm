use litesvm::types::{FailedTransactionMetadata, TransactionMetadata};

use crate::{
    amm::instructions::initialize_protocol_ix,
    common::{context::TestContext, executor::execute_transaction},
};

pub fn initialize_protocol(
    ctx: &mut TestContext,
) -> Result<TransactionMetadata, FailedTransactionMetadata> {
    let ix = initialize_protocol_ix(ctx);
    let payer = ctx.admin.pubkey();
    let signer = ctx.admin.signer();

    execute_transaction(&mut ctx.svm, &payer, &[signer], &[ix])
}
