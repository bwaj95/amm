
use crate::error::AmmError;

pub fn calculate_lp_initial(amount_token_a: u64, amount_token_b: u64) -> Result<u64, AmmError> {
    let product = (amount_token_a as u128)
        .checked_mul(amount_token_b as u128)
        .ok_or(AmmError::MathOverflow)?;

    let liquidity = product.isqrt();

    u64::try_from(liquidity).map_err(|_| AmmError::MathOverflow.into())
}

