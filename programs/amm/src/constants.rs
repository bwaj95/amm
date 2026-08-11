use anchor_lang::prelude::*;

#[constant]
pub const PROTOCOL_CONFIG_SEED: &[u8] = b"protocol_config";

#[constant]
pub const TREASURY_SEED: &[u8] = b"treasury";

#[constant]
pub const POOL_SEED: &[u8] = b"pool";

#[constant]
pub const MINT_SEED: &[u8] = b"mint";

#[constant]
pub const LP_MINT_SEED: &[u8] = b"lp_mint";

#[constant]
pub const LOCKED_LP_SEED: &[u8] = b"locked_lp";

#[constant]
pub const MAX_BPS: u16 = 10_000;

#[constant]
pub const ONE_SOL: u64 = 1_000_000_000u64;

#[constant]
pub const HUNDRED_SOL: u64 = 100_000_000_000u64;

#[constant]
pub const LP_MINT_DECIMALS: u8 = 9;

#[constant]
pub const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

#[constant]
pub const BONK_MINT: &str = "DezXAZ8z7PnrFcZkpPR3rQHBeq5jXcX6assa5yk6PnAa";

#[constant]
pub const MINIMUM_LIQUIDITY: u64 = 1000;