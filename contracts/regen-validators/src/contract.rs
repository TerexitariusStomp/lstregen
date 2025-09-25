use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdError, StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, ValidatorParams, ValidatorResponse,
    ValidatorsResponse,
};
use crate::state::{Config, ValidatorInfo, CONFIG, VALIDATORS};
use cosmwasm_std::Decimal;

const CONTRACT_NAME: &str = "crates.io:regen-validators";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, StdError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let admin: Addr = deps.api.addr_validate(&msg.admin)?;
    CONFIG.save(deps.storage, &Config { admin: admin.clone() })?;

    // Seed initial validators
    for vp in msg.validators {
        let _ = deps.api.addr_validate(&vp.address)?;
        let v = ValidatorInfo {
            address: vp.address.clone(),
            delegated_amount: Uint128::zero(),
            weight: vp.weight,
            last_reward_claim: env.block.time,
            slashing_events: 0,
            uptime_percentage: Decimal::percent(100),
            commission_rate: vp.commission_rate,
            is_active: true,
        };
        VALIDATORS.save(deps.storage, vp.address.clone(), &v)?;
    }

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", admin)
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION))
}

#[entry_point]
pub fn execute(deps: DepsMut, _env: Env, info: MessageInfo, msg: ExecuteMsg) -> Result<Response, StdError> {
    let cfg = CONFIG.load(deps.storage)?;
    if info.sender != cfg.admin {
        return Err(StdError::generic_err("unauthorized"));
    }

    match msg {
        ExecuteMsg::AddValidator { params } => exec_add_validator(deps, params),
        ExecuteMsg::RemoveValidator { address } => exec_remove_validator(deps, address),
        ExecuteMsg::UpdateWeight { address, weight } => exec_update_weight(deps, address, weight),
        ExecuteMsg::SetCommission { address, commission_rate } => {
            exec_set_commission(deps, address, commission_rate)
        }
        ExecuteMsg::Activate { address } => exec_set_active(deps, address, true),
        ExecuteMsg::Deactivate { address } => exec_set_active(deps, address, false),
        ExecuteMsg::TransferAdmin { new_admin } => exec_transfer_admin(deps, new_admin),
    }
}

fn exec_add_validator(deps: DepsMut, params: ValidatorParams) -> Result<Response, StdError> {
    let _ = deps.api.addr_validate(&params.address)?;
    if VALIDATORS.may_load(deps.storage, params.address.clone())?.is_some() {
        return Err(StdError::generic_err("validator already exists"));
    }
    VALIDATORS.save(
        deps.storage,
        params.address.clone(),
        &ValidatorInfo {
            address: params.address.clone(),
            delegated_amount: Uint128::zero(),
            weight: params.weight,
            last_reward_claim: cosmwasm_std::Timestamp::from_seconds(0),
            slashing_events: 0,
            uptime_percentage: Decimal::percent(100),
            commission_rate: params.commission_rate,
            is_active: true,
        },
    )?;
    Ok(Response::new()
        .add_attribute("action", "add_validator")
        .add_attribute("validator", params.address))
}

fn exec_remove_validator(deps: DepsMut, address: String) -> Result<Response, StdError> {
    if VALIDATORS.may_load(deps.storage, address.clone())?.is_none() {
        return Err(StdError::not_found("ValidatorInfo"));
    }
    VALIDATORS.remove(deps.storage, address.clone());
    Ok(Response::new()
        .add_attribute("action", "remove_validator")
        .add_attribute("validator", address))
}

fn exec_update_weight(deps: DepsMut, address: String, weight: Decimal) -> Result<Response, StdError> {
    VALIDATORS.update(deps.storage, address.clone(), |maybe| match maybe {
        Some(mut v) => {
            v.weight = weight;
            Ok(v)
        }
        None => Err(StdError::not_found("ValidatorInfo")),
    })?;
    Ok(Response::new()
        .add_attribute("action", "update_weight")
        .add_attribute("validator", address)
        .add_attribute("weight", weight.to_string()))
}

fn exec_set_commission(
    deps: DepsMut,
    address: String,
    commission_rate: Decimal,
) -> Result<Response, StdError> {
    VALIDATORS.update(deps.storage, address.clone(), |maybe| match maybe {
        Some(mut v) => {
            v.commission_rate = commission_rate;
            Ok(v)
        }
        None => Err(StdError::not_found("ValidatorInfo")),
    })?;
    Ok(Response::new()
        .add_attribute("action", "set_commission")
        .add_attribute("validator", address)
        .add_attribute("commission", commission_rate.to_string()))
}

fn exec_set_active(deps: DepsMut, address: String, is_active: bool) -> Result<Response, StdError> {
    VALIDATORS.update(deps.storage, address.clone(), |maybe| match maybe {
        Some(mut v) => {
            v.is_active = is_active;
            Ok(v)
        }
        None => Err(StdError::not_found("ValidatorInfo")),
    })?;
    Ok(Response::new()
        .add_attribute("action", if is_active { "activate" } else { "deactivate" })
        .add_attribute("validator", address))
}

fn exec_transfer_admin(deps: DepsMut, new_admin: String) -> Result<Response, StdError> {
    let mut cfg = CONFIG.load(deps.storage)?;
    cfg.admin = deps.api.addr_validate(&new_admin)?;
    CONFIG.save(deps.storage, &cfg)?;
    Ok(Response::new()
        .add_attribute("action", "transfer_admin")
        .add_attribute("new_admin", new_admin))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Validators {} => to_binary(&query_validators(deps)?),
        QueryMsg::Validator { address } => to_binary(&query_validator(deps, address)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    let total_active = VALIDATORS
        .range(deps.storage, None, None, Order::Ascending)
        .filter_map(|r| r.ok())
        .map(|(_, v)| v)
        .filter(|v| v.is_active)
        .count() as u32;

    Ok(ConfigResponse {
        admin: cfg.admin.to_string(),
        total_active,
    })
}

fn query_validators(deps: Deps) -> StdResult<ValidatorsResponse> {
    let vals = VALIDATORS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|r| r.map(|(_, v)| v))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ValidatorsResponse { validators: vals })
}

fn query_validator(deps: Deps, address: String) -> StdResult<ValidatorResponse> {
    let val = VALIDATORS.may_load(deps.storage, address.clone())?;
    Ok(ValidatorResponse { validator: val })
}