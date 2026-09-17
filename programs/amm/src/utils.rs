use crate::{error::AmmError, BPS_DENOMINATOR, MAX_SWAP_FEE_BPS};

pub fn validate_fee_config(swap_fee_bps: u16, treasury_fee_bps: u16) -> Result<(), AmmError> {
    if swap_fee_bps > MAX_SWAP_FEE_BPS {
        return Err(AmmError::InvalidSwapFee);
    }

    if treasury_fee_bps > swap_fee_bps {
        return Err(AmmError::InvalidTreasuryFee);
    }

    Ok(())
}

pub fn validate_deadline(deadline: i64, current_timestamp: i64) -> Result<(), AmmError> {
    if current_timestamp > deadline {
        return Err(AmmError::DeadlineExceeded);
    }

    Ok(())
}

pub fn calculate_constant_product(reserve_a: u64, reserve_b: u64) -> Result<u128, AmmError> {
    (reserve_a as u128)
        .checked_mul(reserve_b as u128)
        .ok_or(AmmError::MathOverflow)
}

pub fn validate_constant_product(
    reserve_in_before: u64,
    reserve_out_before: u64,
    reserve_in_after: u64,
    reserve_out_after: u64,
) -> Result<(), AmmError> {
    let k_before = calculate_constant_product(reserve_in_before, reserve_out_before)?;
    let k_after = calculate_constant_product(reserve_in_after, reserve_out_after)?;

    if k_after < k_before {
        return Err(AmmError::InvariantViolation);
    }

    Ok(())
}

pub fn calculate_lp_initial(amount_token_a: u64, amount_token_b: u64) -> Result<u64, AmmError> {
    let product = (amount_token_a as u128)
        .checked_mul(amount_token_b as u128)
        .ok_or(AmmError::MathOverflow)?;

    let liquidity = product.isqrt();

    u64::try_from(liquidity).map_err(|_| AmmError::MathOverflow.into())
}

/// Calculates a proportional deposit using floor division throughout.
///
/// Flooring guarantees that neither selected token amount exceeds the
/// provider's declared maximum. Any fractional remainder stays with the
/// provider, while `min_lp_out` is enforced by the instruction handler.
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

/// Calculates a constant-product swap using raw token base units.
///
/// Fees and output are deliberately rounded down. An unrepresentable
/// fractional fee is not charged; an unrepresentable output remainder stays
/// in the pool. Consequently, a sufficiently small valid swap may pay zero
/// fee in token base units.
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

    validate_fee_config(swap_fee_bps, treasury_fee_bps)
        .map_err(|_| AmmError::InvalidFeeConfig)?;

    // amount_in * (swap_fee_bps / BPS_DENOMINATOR)
    let total_swap_fees_u128: u128 = (amount_in as u128)
        .checked_mul(swap_fee_bps as u128)
        .ok_or(AmmError::MathOverflow)?
        .checked_div(BPS_DENOMINATOR as u128)
        .ok_or(AmmError::DivisionError)?;
    let total_swap_fees =
        u64::try_from(total_swap_fees_u128).map_err(|_| AmmError::MathOverflow)?;

    let treasury_fees_u128: u128 = (amount_in as u128)
        .checked_mul(treasury_fee_bps as u128)
        .ok_or(AmmError::MathOverflow)?
        .checked_div(BPS_DENOMINATOR as u128)
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

/// Calculates proportional withdrawal amounts with floor division.
///
/// Fractional base units remain in the vaults and ultimately back the locked
/// minimum LP position, giving the protocol an explicit dust policy.
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

    if lp_amount > total_lp_supply {
        return Err(AmmError::InvalidLiquidityPoolState);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_liquidity_uses_floor_sqrt_without_overflow() {
        assert_eq!(calculate_lp_initial(10, 40).unwrap(), 20);
        assert_eq!(
            calculate_lp_initial(u64::MAX, u64::MAX).unwrap(),
            u64::MAX
        );
    }

    #[test]
    fn proportional_deposit_rounds_down_and_never_exceeds_maxima() {
        let (actual_a, actual_b, lp_minted) =
            calculate_add_liquidity(3, 10, 2, 7, 30).unwrap();

        assert_eq!((actual_a, actual_b), (2, 6));
        assert!(actual_a <= 2);
        assert!(actual_b <= 7);
        assert_eq!(lp_minted, 18);
    }

    #[test]
    fn swap_rounding_preserves_the_constant_product() {
        let reserve_in = 10_000;
        let reserve_out = 40_000;
        let amount_in = 1_000;
        let calculation = calculate_swap(reserve_in, reserve_out, amount_in, 30, 5).unwrap();

        let reserve_in_after = reserve_in + amount_in - calculation.treasury_fees;
        let reserve_out_after = reserve_out - calculation.amount_out;

        assert!(
            validate_constant_product(
                reserve_in,
                reserve_out,
                reserve_in_after,
                reserve_out_after,
            )
            .is_ok()
        );
    }

    #[test]
    fn tiny_valid_swap_can_round_its_fee_to_zero() {
        let calculation = calculate_swap(1_000_000, 1_000_000, 2, 30, 5).unwrap();

        assert_eq!(calculation.total_swap_fees, 0);
        assert_eq!(calculation.treasury_fees, 0);
        assert_eq!(calculation.lp_fees, 0);
        assert_eq!(calculation.amount_out, 1);
    }

    #[test]
    fn removal_rounds_outputs_down() {
        let calculation = calculate_remove_liquidity(10, 20, 6, 1).unwrap();

        assert_eq!(calculation.amount_a, 1);
        assert_eq!(calculation.amount_b, 3);
    }

    #[test]
    fn fee_configuration_rejects_confiscatory_or_misaligned_fees() {
        assert!(validate_fee_config(30, 5).is_ok());
        assert!(matches!(
            validate_fee_config(MAX_SWAP_FEE_BPS + 1, 0),
            Err(AmmError::InvalidSwapFee)
        ));
        assert!(matches!(
            validate_fee_config(30, 31),
            Err(AmmError::InvalidTreasuryFee)
        ));
    }

    #[test]
    fn deadline_is_inclusive() {
        assert!(validate_deadline(100, 100).is_ok());
        assert!(matches!(
            validate_deadline(99, 100),
            Err(AmmError::DeadlineExceeded)
        ));
    }

    #[test]
    fn invariant_check_rejects_value_loss() {
        assert!(matches!(
            validate_constant_product(10, 10, 10, 9),
            Err(AmmError::InvariantViolation)
        ));
    }

    #[test]
    fn swap_invariant_holds_across_representative_boundaries() {
        let reserves = [(1_000, 1_000), (10_000, 40_000), (u64::MAX, u64::MAX)];
        let inputs = [1, 10, 1_000, 1_000_000, u64::MAX / 4];

        for (reserve_in, reserve_out) in reserves {
            for amount_in in inputs {
                let Ok(calculation) = calculate_swap(reserve_in, reserve_out, amount_in, 30, 5)
                else {
                    continue;
                };

                let reserve_in_after = reserve_in
                    .checked_add(amount_in)
                    .and_then(|value| value.checked_sub(calculation.treasury_fees));
                let reserve_out_after = reserve_out.checked_sub(calculation.amount_out);

                if let (Some(reserve_in_after), Some(reserve_out_after)) =
                    (reserve_in_after, reserve_out_after)
                {
                    assert!(
                        validate_constant_product(
                            reserve_in,
                            reserve_out,
                            reserve_in_after,
                            reserve_out_after,
                        )
                        .is_ok(),
                        "invariant failed for reserves=({reserve_in}, {reserve_out}), input={amount_in}"
                    );
                }
            }
        }
    }

    #[test]
    fn removal_cannot_exceed_total_lp_supply() {
        assert!(matches!(
            calculate_remove_liquidity(10, 20, 5, 6),
            Err(AmmError::InvalidLiquidityPoolState)
        ));
    }
}
