pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
pub mod events;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;

declare_id!("5MyQPKV12rGp2c3ce3qeA8yk2BdDhVJvBJt6TePeRe5F");

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
