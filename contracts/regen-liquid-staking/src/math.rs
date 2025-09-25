use cosmwasm_std::{Decimal, StdError, StdResult, Uint128};

/// Calculate the exchange rate between REGEN and dREGEN
/// Formula: exchange_rate = (total_regen_staked + accumulated_rewards) / total_dregen_supply
pub fn calculate_exchange_rate(
    total_regen_staked: Uint128,
    total_dregen_supply: Uint128,
    accumulated_rewards: Uint128,
) -> StdResult<Decimal> {
    if total_dregen_supply.is_zero() {
        return Ok(Decimal::one());
    }

    let total_value = total_regen_staked.checked_add(accumulated_rewards)?;
    let rate = Decimal::from_ratio(total_value, total_dregen_supply);

    Ok(rate)
}

/// Calculate dREGEN tokens to mint for a given REGEN amount
pub fn calculate_dregen_mint_amount(regen_amount: Uint128, exchange_rate: Decimal) -> StdResult<Uint128> {
    if exchange_rate.is_zero() {
        return Err(StdError::generic_err("Exchange rate cannot be zero"));
    }

    let dregen_amount = Decimal::from_ratio(regen_amount, Uint128::one())
        .checked_div(exchange_rate)
        .map_err(|e| StdError::generic_err(e.to_string()))?
        .to_uint_floor();

    Ok(dregen_amount)
}

/// Calculate REGEN tokens to return for a given dREGEN amount
pub fn calculate_regen_return_amount(dregen_amount: Uint128, exchange_rate: Decimal) -> StdResult<Uint128> {
    let regen_amount = exchange_rate
        .checked_mul(Decimal::from_ratio(dregen_amount, Uint128::one()))?
        .to_uint_floor();

    Ok(regen_amount)
}

/// Calculate fee amount based on the given amount and fee rate
pub fn calculate_fee(amount: Uint128, fee_rate: Decimal) -> StdResult<Uint128> {
    let fee = fee_rate
        .checked_mul(Decimal::from_ratio(amount, Uint128::one()))?
        .to_uint_floor();

    Ok(fee)
}

/// Calculate optimal delegation distribution across validators
pub fn calculate_validator_distribution(
    total_amount: Uint128,
    validators: &[(String, Decimal)], // (address, weight)
) -> StdResult<Vec<(String, Uint128)>> {
    let mut distribution = Vec::new();
    let total_weight: Decimal = validators.iter().map(|(_, weight)| *weight).sum();

    if total_weight.is_zero() {
        return Err(StdError::generic_err("Total validator weight cannot be zero"));
    }

    let mut remaining_amount = total_amount;

    for (i, (validator_addr, weight)) in validators.iter().enumerate() {
        let allocation = if i == validators.len() - 1 {
            // Last validator gets remaining amount to handle rounding
            remaining_amount
        } else {
            let allocation = weight
                .checked_div(total_weight)
                .map_err(|e| StdError::generic_err(e.to_string()))?
                .checked_mul(Decimal::from_ratio(total_amount, Uint128::one()))?
                .to_uint_floor();
            remaining_amount = remaining_amount.checked_sub(allocation)?;
            allocation
        };

        if !allocation.is_zero() {
            distribution.push((validator_addr.clone(), allocation));
        }
    }

    Ok(distribution)
}

/// Calculate Annual Percentage Rate (APR) based on rewards
pub fn calculate_apr(
    total_staked: Uint128,
    rewards_over_period: Uint128,
    period_days: u64,
) -> StdResult<Decimal> {
    if total_staked.is_zero() || period_days == 0 {
        return Ok(Decimal::zero());
    }

    let annualized_rewards = Decimal::from_ratio(rewards_over_period, Uint128::one())
        .checked_mul(Decimal::from_ratio(365u128, period_days))?;

    let apr = annualized_rewards
        .checked_div(Decimal::from_ratio(total_staked, Uint128::one()))
        .map_err(|e| StdError::generic_err(e.to_string()))?;

    Ok(apr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_exchange_rate_calculation() {
        let rate = calculate_exchange_rate(
            Uint128::new(1_000_000),
            Uint128::new(1_000_000),
            Uint128::new(100_000),
        )
        .unwrap();

        assert_eq!(rate, Decimal::from_str("1.1").unwrap());
    }

    #[test]
    fn test_dregen_mint_calculation() {
        let dregen_amount =
            calculate_dregen_mint_amount(Uint128::new(1_100), Decimal::from_str("1.1").unwrap())
                .unwrap();

        assert_eq!(dregen_amount, Uint128::new(1_000));
    }

    #[test]
    fn test_validator_distribution() {
        let validators = vec![
            ("validator1".to_string(), Decimal::percent(50)),
            ("validator2".to_string(), Decimal::percent(30)),
            ("validator3".to_string(), Decimal::percent(20)),
        ];

        let distribution = calculate_validator_distribution(Uint128::new(1_000_000), &validators).unwrap();

        assert_eq!(distribution.len(), 3);
        assert_eq!(distribution[0].1, Uint128::new(500_000));
        assert_eq!(distribution[1].1, Uint128::new(300_000));
        assert_eq!(distribution[2].1, Uint128::new(200_000));
    }
}