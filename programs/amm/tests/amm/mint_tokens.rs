use litesvm::types::{FailedTransactionMetadata, TransactionMetadata};
use solana_pubkey::Pubkey;

use crate::{
    amm::instructions::mint_tokens_ix,
    common::{context::TestContext, executor::execute_transaction},
};

pub fn mint_tokens(
    ctx: &mut TestContext,
    user: &Pubkey,
    user_ata: &Pubkey,
    mint_id: u64,
    amount: u64,
) -> Result<TransactionMetadata, FailedTransactionMetadata> {
    let admin = ctx.admin.pubkey();
    let signer = ctx.admin.signer();

    let ix = mint_tokens_ix(ctx, &admin, user, user_ata, mint_id, amount);

    execute_transaction(&mut ctx.svm, &admin, &[signer], &[ix])
}
