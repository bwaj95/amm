use solana_keypair::{Keypair, Signer};
use solana_pubkey::Pubkey;

pub struct User {
    name: String,
    keypair: Keypair, // says keypair never used, although it is private
}

impl User {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            keypair: Keypair::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn pubkey(&self) -> Pubkey {
        // pubkey is a 32 byte number, and implements Clone trait,
        // so just returm Pubkey instead of &Pubkey, no moving ownership out
        // keypair.pubkey() is working only if Signer trait is present in scope although we aren't using it, why?

        self.keypair.pubkey()
    }

    pub fn signer(&self) -> &Keypair {
        // Borrow the private key; ownership stays with User.F
        &self.keypair
    }
}
