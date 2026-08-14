use anchor_lang::prelude::*;

#[error_code]
pub enum AmmError {
    #[msg("Swap fee should be within allowed max basis points")]
    InvalidSwapFee,

    #[msg("Treasury fee cannot exceed swap fee.")]
    InvalidTreasuryFee,

    #[msg("Invalid mint order.")]
    InvalidMintOrder,

    #[msg("Same mints provided.")]
    SameMint,

    #[msg("Pool already initialized.")]
    PoolAlreadyInitialized,

    #[msg("Input amounts must be greater than zero.")]
    InvalidInputAmount,

    #[msg("Insufficient token balances in providers accounts.")]
    InsufficientFunds,

    #[msg("MathOverflow.")]
    MathOverflow,

    #[msg("DivisionError.")]
    DivisionError,

    #[msg("More tokens needed to meet minimum liquidity threshold.")]
    MinimumLiquidityThresholdNotMet,

    #[msg("Argument decimals dont match with the mint.")]
    MintDecimalsMismatch,

    #[msg("Amounts are too small to produce a valid proportional deposit.")]
    InsufficientTokensProvided,

    #[msg("Pool reserves and LP supply must be greater than zero.")]
    InvalidPoolState,
}
