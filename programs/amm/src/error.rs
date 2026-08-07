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
}
