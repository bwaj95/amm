use anchor_lang::require;

use crate::{error::AmmError, MAX_BPS, MINIMUM_LIQUIDITY};

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

pub fn calculate_swap(
    reserve_in: u64,
    reserve_out: u64,
    amount_in: u64,
    swap_fee_bps: u16,
    treasury_fee_bps: u16,
) -> Result<SwapCalculation, AmmError> {
    if reserve_in == 0 || reserve_out == 0 {
        return Err(AmmError::InvalidPoolState);
    }

    if amount_in == 0 {
        return Err(AmmError::InvalidInputAmount);
    }

    if swap_fee_bps > MAX_BPS || treasury_fee_bps > swap_fee_bps {
        return Err(AmmError::InvalidFeeConfig);
    }

    // amount_in * (swap_fee_bps / MAX_BPS)
    let total_swap_fees_u128: u128 = (amount_in as u128)
        .checked_mul(swap_fee_bps as u128)
        .ok_or(AmmError::MathOverflow)?
        .checked_div(MAX_BPS as u128)
        .ok_or(AmmError::DivisionError)?;
    let total_swap_fees =
        u64::try_from(total_swap_fees_u128).map_err(|_| AmmError::MathOverflow)?;

    let treasury_fees_u128: u128 = (amount_in as u128)
        .checked_mul(treasury_fee_bps as u128)
        .ok_or(AmmError::MathOverflow)?
        .checked_div(MAX_BPS as u128)
        .ok_or(AmmError::DivisionError)?;
    let treasury_fees = u64::try_from(treasury_fees_u128).map_err(|_| AmmError::MathOverflow)?;

    let lp_fees = total_swap_fees
        .checked_sub(treasury_fees)
        .ok_or(AmmError::MathOverflow)?;

    let amount_after_fees = amount_in
        .checked_sub(total_swap_fees)
        .ok_or(AmmError::MathOverflow)?;

    if amount_after_fees == 0 {
        return Err(AmmError::InsufficientSwapAmount);
    }

    // (reserve_out * amount_after_fees) / (reserve_in + amount_after_fees)
    let amount_out_u128: u128 = (reserve_out as u128)
        .checked_mul(amount_after_fees as u128)
        .ok_or(AmmError::MathOverflow)?
        .checked_div(
            (reserve_in as u128)
                .checked_add(amount_after_fees as u128)
                .ok_or(AmmError::MathOverflow)?,
        )
        .ok_or(AmmError::DivisionError)?;

    let amount_out: u64 = u64::try_from(amount_out_u128).map_err(|_| AmmError::MathOverflow)?;

    if amount_out == 0 {
        return Err(AmmError::InsufficientSwapAmount);
    }

    Ok(SwapCalculation {
        amount_out,
        total_swap_fees,
        treasury_fees,
        lp_fees,
    })
}

pub fn calculate_remove_liquidity(
    reserve_a: u64,
    reserve_b: u64,
    total_lp_supply: u64,
    lp_amount: u64,
) -> Result<LpRemoveCalculation, AmmError> {
    if reserve_a == 0 || reserve_b == 0 || total_lp_supply == 0 {
        return Err(AmmError::InvalidPoolState);
    }

    if lp_amount == 0 {
        return Err(AmmError::InvalidInputAmount);
    }

    let amount_a_u128: u128 = (lp_amount as u128)
        .checked_mul(reserve_a as u128)
        .ok_or(AmmError::MathOverflow)?
        .checked_div(total_lp_supply as u128)
        .ok_or(AmmError::DivisionError)?;
    let amount_a: u64 = u64::try_from(amount_a_u128).map_err(|_| AmmError::MathOverflow)?;

    if amount_a == 0 {
        return Err(AmmError::InsufficientLiquidityOutput);
    }

    let amount_b_u128: u128 = (lp_amount as u128)
        .checked_mul(reserve_b as u128)
        .ok_or(AmmError::MathOverflow)?
        .checked_div(total_lp_supply as u128)
        .ok_or(AmmError::DivisionError)?;
    let amount_b: u64 = u64::try_from(amount_b_u128).map_err(|_| AmmError::MathOverflow)?;

    if amount_b == 0 {
        return Err(AmmError::InsufficientLiquidityOutput);
    }

    Ok(LpRemoveCalculation { amount_a, amount_b })
}

pub struct SwapCalculation {
    pub amount_out: u64,
    pub total_swap_fees: u64,
    pub treasury_fees: u64,
    pub lp_fees: u64,
}

pub struct LpRemoveCalculation {
    pub amount_a: u64,
    pub amount_b: u64,
}
