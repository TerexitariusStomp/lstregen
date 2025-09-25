use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Decimal,
    Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, State, ValidatorInfo, CONFIG, STATE, VALIDATORS};
use crate::execute::{
    execute_claim_rewards, execute_claim_unbonding, execute_pause, execute_rebalance, execute_resume,
    execute_stake, execute_unbond, execute_update_config, execute_update_validators,
};
use crate::query::{
    query_config, query_exchange_rate, query_simulate_stake, query_simulate_unbond, query_state,
    query_unbonding, query_validators,
};

const CONTRACT_NAME: &str = "crates.io:regen-liquid-staking";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Validate admin address
    let admin = deps.api.addr_validate(&msg.admin)?;

    // Validate fee rate (must be between 0 and 20%)
    if msg.fee_rate > Decimal::percent(20) {
        return Err(ContractError::InvalidFeeRate {});
    }

    // Initialize contract configuration
    let config = Config {
        admin: admin.clone(),
        dregen_token: admin.clone(), // placeholder; updated after token creation
        fee_rate: msg.fee_rate,
        unbonding_period: msg.unbonding_period,
        max_validators: msg.max_validators,
        min_delegation: msg.min_delegation,
        pause_contract: false,
    };

    // Initialize contract state
    let state = State {
        total_regen_staked: Uint128::zero(),
        total_dregen_supply: Uint128::zero(),
        exchange_rate: Decimal::one(),
        last_update_time: env.block.time,
        total_rewards_claimed: Uint128::zero(),
        pending_unbonding: Uint128::zero(),
    };

    CONFIG.save(deps.storage, &config)?;
    STATE.save(deps.storage, &state)?;

    // Initialize validators
    for validator_param in msg.validators {
        let _validator_addr = deps.api.addr_validate(&validator_param.address)?;
        let validator_info = ValidatorInfo {
            address: validator_param.address,
            delegated_amount: Uint128::zero(),
            weight: validator_param.weight,
            last_reward_claim: env.block.time,
            slashing_events: 0,
            uptime_percentage: Decimal::percent(100),
            commission_rate: Decimal::percent(10), // Default commission
            is_active: true,
        };
        VALIDATORS.save(deps.storage, validator_info.address.clone(), &validator_info)?;
    }

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", admin)
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Stake {} => execute_stake(deps, env, info),
        ExecuteMsg::Unbond { dregen_amount } => execute_unbond(deps, env, info, dregen_amount),
        ExecuteMsg::ClaimUnbonding { unbonding_id } => {
            execute_claim_unbonding(deps, env, info, unbonding_id)
        }
        ExecuteMsg::Rebalance {} => execute_rebalance(deps, env, info),
        ExecuteMsg::ClaimRewards {} => execute_claim_rewards(deps, env, info),
        ExecuteMsg::UpdateValidators { validators } => {
            execute_update_validators(deps, env, info, validators)
        }
        ExecuteMsg::Pause {} => execute_pause(deps, env, info),
        ExecuteMsg::Resume {} => execute_resume(deps, env, info),
        ExecuteMsg::UpdateConfig {
            admin,
            fee_rate,
            max_validators,
        } => execute_update_config(deps, env, info, admin, fee_rate, max_validators),
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::State {} => to_binary(&query_state(deps)?),
        QueryMsg::ExchangeRate {} => to_binary(&query_exchange_rate(deps, env)?),
        QueryMsg::Validators {} => to_binary(&query_validators(deps)?),
        QueryMsg::Unbonding { user } => to_binary(&query_unbonding(deps, user)?),
        QueryMsg::SimulateStake { amount } => to_binary(&query_simulate_stake(deps, env, amount)?),
        QueryMsg::SimulateUnbond { dregen_amount } => {
            to_binary(&query_simulate_unbond(deps, env, dregen_amount)?)
        }
    }
}