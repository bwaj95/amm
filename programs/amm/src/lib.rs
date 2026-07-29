pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;

declare_id!("GWEbss74ivW8qifn8pkC5tJcYcXpVsLkBAYpXdN8dVd");

#[program]
pub mod amm {
    use super::*;

    pub fn initialize_protocol(
        ctx: Context<InitializeProtocol>,
        swap_fee_bps: u16,
        treasury_fee_bps: u16,
    ) -> Result<()> {
        instructions::handler(ctx, swap_fee_bps, treasury_fee_bps)
    }
}
