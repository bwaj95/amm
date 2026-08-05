use litesvm::{
    types::{FailedTransactionMetadata, TransactionMetadata},
    LiteSVM,
};
use solana_message::{Instruction, Message, VersionedMessage};
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;


 ///  Builds and submits a transaction to LiteSVM.
pub fn execute_transaction(
    svm: &mut LiteSVM,
    payer: &Pubkey,
    signers: &[&dyn Signer],
    ixs: &[Instruction],
) -> Result<TransactionMetadata, FailedTransactionMetadata> {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(ixs, Some(payer), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();

    svm.send_transaction(tx)
}
