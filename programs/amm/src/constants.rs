use anchor_lang::prelude::*;

#[constant]
pub const PROTOCOL_CONFIG_SEED: &[u8] = b"protocol_config";

#[constant]
pub const TREASURY_SEED: &[u8] = b"treasury";

#[constant]
pub const POOL_SEED: &[u8] = b"pool";

#[constant]
pub const MAX_BPS: u16 = 10_000;

#[constant]
pub const ONE_SOL: u64 = 1_000_000_000u64;

#[constant]
pub const HUNDRED_SOL: u64 = 100_000_000_000u64;
