pub mod constants;
pub mod error;
pub mod events;
pub mod instructions;
pub mod state;

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

    pub fn initialize_mint(ctx: Context<InitializeMint>, mint_id: u64, decimals: u8) -> Result<()> {
        instructions::initialize_mint_handler(ctx, mint_id, decimals)
    }

    pub fn create_pool(ctx: Context<CreatePool>) -> Result<()> {
        instructions::create_pool_handler(ctx)
    }
}
