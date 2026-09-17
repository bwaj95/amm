use litesvm::types::{FailedTransactionMetadata, TransactionMetadata};
use solana_message::Instruction;
use solana_pubkey::Pubkey;

use crate::{
    amm::instructions::{
        accept_admin_ix, cancel_admin_transfer_ix, propose_admin_ix, set_paused_ix,
        update_fees_ix,
    },
    common::{context::TestContext, executor::execute_transaction},
};

fn execute_as_authority(
    ctx: &mut TestContext,
    authority: &Pubkey,
    ix: Instruction,
) -> Result<TransactionMetadata, FailedTransactionMetadata> {
    if *authority == ctx.admin.pubkey() {
        execute_transaction(&mut ctx.svm, authority, &[ctx.admin.signer()], &[ix])
    } else if *authority == ctx.alice.pubkey() {
        execute_transaction(&mut ctx.svm, authority, &[ctx.alice.signer()], &[ix])
    } else if *authority == ctx.bob.pubkey() {
        execute_transaction(&mut ctx.svm, authority, &[ctx.bob.signer()], &[ix])
    } else if *authority == ctx.charlie.pubkey() {
        execute_transaction(&mut ctx.svm, authority, &[ctx.charlie.signer()], &[ix])
    } else {
        panic!("Unknown authority: {authority}");
    }
}

pub fn set_paused(
    ctx: &mut TestContext,
    authority: &Pubkey,
    paused: bool,
) -> Result<TransactionMetadata, FailedTransactionMetadata> {
    let ix = set_paused_ix(ctx, authority, paused);
    execute_as_authority(ctx, authority, ix)
}

pub fn update_fees(
    ctx: &mut TestContext,
    authority: &Pubkey,
    swap_fee_bps: u16,
    treasury_fee_bps: u16,
) -> Result<TransactionMetadata, FailedTransactionMetadata> {
    let ix = update_fees_ix(ctx, authority, swap_fee_bps, treasury_fee_bps);
    execute_as_authority(ctx, authority, ix)
}

pub fn propose_admin(
    ctx: &mut TestContext,
    authority: &Pubkey,
    new_admin: Pubkey,
) -> Result<TransactionMetadata, FailedTransactionMetadata> {
    let ix = propose_admin_ix(ctx, authority, new_admin);
    execute_as_authority(ctx, authority, ix)
}

pub fn cancel_admin_transfer(
    ctx: &mut TestContext,
    authority: &Pubkey,
) -> Result<TransactionMetadata, FailedTransactionMetadata> {
    let ix = cancel_admin_transfer_ix(ctx, authority);
    execute_as_authority(ctx, authority, ix)
}

pub fn accept_admin(
    ctx: &mut TestContext,
    pending_admin: &Pubkey,
) -> Result<TransactionMetadata, FailedTransactionMetadata> {
    let ix = accept_admin_ix(ctx, pending_admin);
    execute_as_authority(ctx, pending_admin, ix)
}
