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

    #[msg("Arithmetic overflow.")]
    MathOverflow,

    #[msg("Invalid division.")]
    DivisionError,

    #[msg("More tokens needed to meet minimum liquidity threshold.")]
    MinimumLiquidityThresholdNotMet,

    #[msg("Argument decimals do not match the mint.")]
    MintDecimalsMismatch,

    #[msg("Amounts are too small to produce a valid proportional deposit.")]
    InsufficientTokensProvided,

    #[msg("Pool reserves and LP supply must be greater than zero.")]
    InvalidPoolState,

    #[msg("Invalid values for swap fee and treasury fee.")]
    InvalidFeeConfig,

    #[msg("Amount Insufficient to make a swap.")]
    InsufficientSwapAmount,

    #[msg("The calculated output does not satisfy the slippage limit.")]
    SlippageExceeded,

    #[msg("Insufficient liquidity in the pool.")]
    InsufficientLiquidity,

    #[msg("Invalid liquidity-pool state.")]
    InvalidLiquidityPoolState,

    #[msg("Liquidity amount too small to redeem.")]
    InsufficientLiquidityOutput,

    #[msg("The protocol is currently paused.")]
    ProtocolPaused,

    #[msg("The protocol must be paused for this administrative change.")]
    ProtocolMustBePaused,

    #[msg("The transaction deadline has expired.")]
    DeadlineExceeded,

    #[msg("The caller is not the protocol administrator.")]
    UnauthorizedAdmin,

    #[msg("The caller is not the pending protocol administrator.")]
    UnauthorizedPendingAdmin,

    #[msg("The proposed administrator is invalid.")]
    InvalidAdmin,

    #[msg("No administrator transfer is pending.")]
    NoPendingAdmin,

    #[msg("The constant-product invariant would decrease.")]
    InvariantViolation,

    #[msg("The pool vaults must be empty before initial liquidity is added.")]
    InitialVaultsNotEmpty,
}
