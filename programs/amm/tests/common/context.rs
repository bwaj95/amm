use amm::HUNDRED_SOL;
use litesvm::{
    types::{FailedTransactionMetadata, TransactionMetadata},
    LiteSVM,
};
use solana_pubkey::Pubkey;
use crate::common::users::User;

pub struct TestContext {
    pub svm: LiteSVM,
    pub program_id: Pubkey,
    pub admin: User,
    pub alice: User,
    pub bob: User,
    pub charlie: User,
}

impl TestContext {
    pub fn new(program_id: Pubkey) -> Self {
        // boot the chain
        let mut svm = LiteSVM::new();

        // load our program in it
        let bytes = include_bytes!("../../../../target/deploy/amm.so");
        svm.add_program(program_id, bytes).unwrap();

        // create users, fund them
        let admin = Self::init_user("admin", HUNDRED_SOL, &mut svm);
        let alice = Self::init_user("alice", HUNDRED_SOL, &mut svm);
        let bob = Self::init_user("bob", HUNDRED_SOL, &mut svm);
        let charlie = Self::init_user("charlie", HUNDRED_SOL, &mut svm);

        // return the test context
        Self {
            svm,
            program_id,
            admin,
            alice,
            bob,
            charlie,
        }
    }

    fn init_user(name: &str, amount: u64, svm: &mut LiteSVM) -> User {
        let user = User::new(name);
        Self::fund_account(&user.pubkey(), amount, svm).unwrap();

        user
    }

    fn fund_account(
        pubkey: &Pubkey,
        amount: u64,
        svm: &mut LiteSVM,
    ) -> Result<TransactionMetadata, FailedTransactionMetadata> {
        // error fixed
        svm.airdrop(pubkey, amount)
    }
}
