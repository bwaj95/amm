use crate::error::AmmError;

pub fn calculate_lp_initial(amount_token_a: u64, amount_token_b: u64) -> Result<u64, AmmError> {
    let product = (amount_token_a as u128)
        .checked_mul(amount_token_b as u128)
        .ok_or(AmmError::MathOverflow)?;

    let liquidity = product.isqrt();

    u64::try_from(liquidity).map_err(|_| AmmError::MathOverflow.into())
}

pub fn calculate_add_liquidity(
    reserve_a: u64,
    reserve_b: u64,
    max_amount_a: u64,
    max_amount_b: u64,
    total_lp_supply: u64,
) -> Result<(u64, u64, u64), AmmError> {
    if reserve_a == 0 || reserve_b == 0 || total_lp_supply == 0 {
        return Err(AmmError::InvalidPoolState);
    }

    //(max_amount_a / reserve_a) * reserve_b;
    let required_b_u128 = (max_amount_a as u128)
        .checked_mul(reserve_b as u128)
        .ok_or(AmmError::MathOverflow)?
        .checked_div(reserve_a as u128)
        .ok_or(AmmError::InvalidPoolState)?;

    let required_b = u64::try_from(required_b_u128).map_err(|_| AmmError::MathOverflow)?;

    let (actual_a, actual_b) = if required_b <= max_amount_b {
        (max_amount_a, required_b)
    } else {
        let required_a_u128 = (max_amount_b as u128)
            .checked_mul(reserve_a as u128)
            .ok_or(AmmError::MathOverflow)?
            .checked_div(reserve_b as u128)
            .ok_or(AmmError::InvalidPoolState)?;

        let required_a = u64::try_from(required_a_u128).map_err(|_| AmmError::MathOverflow)?;

        (required_a, max_amount_b)
    };

    if actual_a == 0 || actual_b == 0 {
        return Err(AmmError::InsufficientTokensProvided);
    }

    let lp_from_a_u128 = (actual_a as u128)
        .checked_mul(total_lp_supply as u128)
        .ok_or(AmmError::MathOverflow)?
        .checked_div(reserve_a as u128)
        .ok_or(AmmError::InvalidPoolState)?;

    let lp_from_a = u64::try_from(lp_from_a_u128).map_err(|_| AmmError::MathOverflow)?;

    let lp_from_b_u128 = (actual_b as u128)
        .checked_mul(total_lp_supply as u128)
        .ok_or(AmmError::MathOverflow)?
        .checked_div(reserve_b as u128)
        .ok_or(AmmError::InvalidPoolState)?;

    let lp_from_b = u64::try_from(lp_from_b_u128).map_err(|_| AmmError::MathOverflow)?;

    let lp_to_mint: u64 = std::cmp::min(lp_from_a, lp_from_b);
    if lp_to_mint == 0 {
        return Err(AmmError::InsufficientTokensProvided);
    }

    Ok((actual_a, actual_b, lp_to_mint))
}
