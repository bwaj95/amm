pub mod anchor_utils;
pub mod constants;
pub mod error;
pub mod events;
pub mod instructions;
pub mod state;
pub mod utils;

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

    pub fn mint_tokens(ctx: Context<MintTokens>, mint_id: u64, amount: u64) -> Result<()> {
        instructions::mint_tokens_handler(ctx, amount)
    }

    pub fn add_initial_liquidity(
        ctx: Context<AddInitialLiquidity>,
        amount_token_a: u64,
        amount_token_b: u64,
    ) -> Result<()> {
        instructions::add_initial_liquidity_handler(ctx, amount_token_a, amount_token_b)
    }

    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        max_amount_a: u64,
        max_amount_b: u64,
    ) -> Result<()> {
        instructions::add_liquidity_handler(ctx, max_amount_a, max_amount_b)
    }

    pub fn swap(
        ctx: Context<Swap>,
        amount_in: u64,
        min_amount_out: u64,
        a_to_b: bool,
    ) -> Result<()> {
        swap_handler(ctx, amount_in, min_amount_out, a_to_b)
    }

    pub fn remove_liquidity(
        ctx: Context<RemoveLiquidity>,
        lp_amount: u64,
        min_amount_a: u64,
        min_amount_b: u64,
    ) -> Result<()> {
        remove_liquidity_handler(ctx, lp_amount, min_amount_a, min_amount_b)
    }
}
