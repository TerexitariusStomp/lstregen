use cosmwasm_std::{Deps, Env, Order, StdResult, Uint128};
use crate::math::{calculate_dregen_mint_amount, calculate_exchange_rate, calculate_fee, calculate_regen_return_amount};
use crate::msg::{
    ConfigResponse, ExchangeRateResponse, SimulateStakeResponse, SimulateUnbondResponse,
    StateResponse, UnbondingResponse, ValidatorsResponse,
};
use crate::state::{CONFIG, STATE, UNBONDING_REQUESTS, VALIDATORS};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        admin: cfg.admin.to_string(),
        dregen_token: cfg.dregen_token.to_string(),
        fee_rate: cfg.fee_rate,
        unbonding_period: cfg.unbonding_period,
        max_validators: cfg.max_validators,
        min_delegation: cfg.min_delegation,
        pause_contract: cfg.pause_contract,
    })
}

pub fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let st = STATE.load(deps.storage)?;
    Ok(StateResponse {
        total_regen_staked: st.total_regen_staked,
        total_dregen_supply: st.total_dregen_supply,
        exchange_rate: st.exchange_rate,
        last_update_time: st.last_update_time.seconds(),
        total_rewards_claimed: st.total_rewards_claimed,
        pending_unbonding: st.pending_unbonding,
    })
}

pub fn query_exchange_rate(deps: Deps, env: Env) -> StdResult<ExchangeRateResponse> {
    let st = STATE.load(deps.storage)?;
    let total_rewards = query_total_rewards(deps, &env)?;
    let rate = calculate_exchange_rate(
        st.total_regen_staked,
        st.total_dregen_supply,
        total_rewards,
    )?;
    Ok(ExchangeRateResponse {
        rate,
        last_updated: env.block.time.seconds(),
    })
}

pub fn query_validators(deps: Deps) -> StdResult<ValidatorsResponse> {
    let vals: Vec<_> = VALIDATORS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|r| r.map(|(_, v)| v))
        .collect::<StdResult<Vec<_>>>()?;
    Ok(ValidatorsResponse { validators: vals })
}

pub fn query_unbonding(deps: Deps, user: String) -> StdResult<UnbondingResponse> {
    let addr = deps.api.addr_validate(&user)?;
    let mut requests = Vec::new();

    for item in UNBONDING_REQUESTS.range(deps.storage, None, None, Order::Ascending) {
        let (_k, req) = item?;
        if req.user == addr {
            requests.push(req);
        }
    }

    Ok(UnbondingResponse { requests })
}

pub fn query_simulate_stake(deps: Deps, env: Env, amount: Uint128) -> StdResult<SimulateStakeResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    let st = STATE.load(deps.storage)?;
    let total_rewards = query_total_rewards(deps, &env)?;

    let exchange_rate = calculate_exchange_rate(
        st.total_regen_staked,
        st.total_dregen_supply,
        total_rewards,
    )?;
    let fee_amount = calculate_fee(amount, cfg.fee_rate)?;
    let net = amount.checked_sub(fee_amount)?;
    let dregen_amount = calculate_dregen_mint_amount(net, exchange_rate)?;

    Ok(SimulateStakeResponse {
        dregen_amount,
        exchange_rate,
        fee_amount,
    })
}

pub fn query_simulate_unbond(deps: Deps, env: Env, dregen_amount: Uint128) -> StdResult<SimulateUnbondResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    let st = STATE.load(deps.storage)?;
    let total_rewards = query_total_rewards(deps, &env)?;

    let exchange_rate = calculate_exchange_rate(
        st.total_regen_staked,
        st.total_dregen_supply,
        total_rewards,
    )?;
    let gross_regen = calculate_regen_return_amount(dregen_amount, exchange_rate)?;
    let fee_amount = calculate_fee(gross_regen, cfg.fee_rate)?;
    let regen_amount = gross_regen.checked_sub(fee_amount)?;
    let completion_time = env.block.time.plus_seconds(cfg.unbonding_period).seconds();

    Ok(SimulateUnbondResponse {
        regen_amount,
        completion_time,
        fee_amount,
    })
}

// Placeholder for rewards query from distribution module.
fn query_total_rewards(_deps: Deps, _env: &Env) -> StdResult<Uint128> {
    Ok(Uint128::zero())
}