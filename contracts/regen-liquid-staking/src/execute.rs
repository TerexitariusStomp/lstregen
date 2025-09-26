use cosmwasm_std::{
    to_binary, Coin, CosmosMsg, Deps, DepsMut, Decimal, Env, MessageInfo, Order, Response,
    Uint128,
};
use cosmwasm_std::{BankMsg, DistributionMsg, StakingMsg, WasmMsg};
use cw20::Cw20ExecuteMsg;

use crate::error::ContractError;
use crate::math::{
    calculate_dregen_mint_amount, calculate_exchange_rate, calculate_fee,
    calculate_regen_return_amount, calculate_validator_distribution,
};
use crate::msg::ValidatorParams;
use crate::state::{
    Config, State, UnbondingRequest, ValidatorInfo, CONFIG, NEXT_UNBONDING_ID, STATE,
    UNBONDING_REQUESTS, VALIDATORS,
};

pub fn execute_stake(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;

    // Check if contract is paused
    if config.pause_contract {
        return Err(ContractError::ContractPaused {});
    }

    // Validate staking amount
    let regen_amount = info
        .funds
        .iter()
        .find(|coin| coin.denom == "uregen")
        .map(|coin| coin.amount)
        .unwrap_or_else(Uint128::zero);

    if regen_amount < config.min_delegation {
        return Err(ContractError::InsufficientStake {
            minimum: config.min_delegation,
            received: regen_amount,
        });
    }

    // Calculate current exchange rate
    let total_rewards = query_total_rewards(deps.as_ref(), &env)?;
    let current_exchange_rate = calculate_exchange_rate(
        state.total_regen_staked,
        state.total_dregen_supply,
        total_rewards,
    )?;

    // Calculate fee and net staking amount
    let fee_amount = calculate_fee(regen_amount, config.fee_rate)?;
    let net_stake_amount = regen_amount.checked_sub(fee_amount).map_err(|e| cosmwasm_std::StdError::generic_err(e.to_string()))?;

    // Calculate dREGEN tokens to mint
    let dregen_mint_amount =
        calculate_dregen_mint_amount(net_stake_amount, current_exchange_rate)?;

    // Get active validators and calculate distribution
    let active_validators = get_active_validators(deps.as_ref())?;
    let validator_distribution =
        calculate_validator_distribution(net_stake_amount, &active_validators)?;

    // Create delegation messages
    let mut messages: Vec<CosmosMsg> = Vec::new();

    for (validator_addr, delegation_amount) in validator_distribution {
        messages.push(CosmosMsg::Staking(StakingMsg::Delegate {
            validator: validator_addr.clone(),
            amount: Coin {
                denom: "uregen".to_string(),
                amount: delegation_amount,
            },
        }));

        // Update validator info
        VALIDATORS.update(
            deps.storage,
            validator_addr.clone(),
            |validator_info: Option<ValidatorInfo>| match validator_info {
                Some(mut info) => {
                    info.delegated_amount = info.delegated_amount.checked_add(delegation_amount).map_err(|e| cosmwasm_std::StdError::generic_err(e.to_string()))?;
                    Ok(info)
                }
                None => Err(ContractError::ValidatorNotFound { validator: validator_addr }),
            },
        )?;
    }

    // Mint dREGEN tokens to user
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.dregen_token.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Mint {
            recipient: info.sender.to_string(),
            amount: dregen_mint_amount,
        })?,
        funds: vec![],
    }));

    // Send fee to treasury if applicable
    if !fee_amount.is_zero() {
        messages.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: config.admin.to_string(),
            amount: vec![Coin {
                denom: "uregen".to_string(),
                amount: fee_amount,
            }],
        }));
    }

    // Update state
    state.total_regen_staked = state.total_regen_staked.checked_add(net_stake_amount).map_err(|e| cosmwasm_std::StdError::generic_err(e.to_string()))?;
    state.total_dregen_supply = state.total_dregen_supply.checked_add(dregen_mint_amount).map_err(|e| cosmwasm_std::StdError::generic_err(e.to_string()))?;
    state.exchange_rate = current_exchange_rate;
    state.last_update_time = env.block.time;

    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("method", "stake")
        .add_attribute("staker", info.sender)
        .add_attribute("regen_amount", regen_amount)
        .add_attribute("dregen_amount", dregen_mint_amount)
        .add_attribute("exchange_rate", current_exchange_rate.to_string())
        .add_attribute("fee_amount", fee_amount))
}

pub fn execute_unbond(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    dregen_amount: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;

    // Check if contract is paused
    if config.pause_contract {
        return Err(ContractError::ContractPaused {});
    }

    // Validate unbonding amount
    if dregen_amount.is_zero() {
        return Err(ContractError::InvalidUnbondAmount {});
    }

    // Calculate current exchange rate
    let total_rewards = query_total_rewards(deps.as_ref(), &env)?;
    let current_exchange_rate = calculate_exchange_rate(
        state.total_regen_staked,
        state.total_dregen_supply,
        total_rewards,
    )?;

    // Calculate REGEN amount to unbond
    let regen_amount = calculate_regen_return_amount(dregen_amount, current_exchange_rate)?;

    // Calculate fee
    let fee_amount = calculate_fee(regen_amount, config.fee_rate)?;
    let net_unbond_amount = regen_amount.checked_sub(fee_amount).map_err(|e| cosmwasm_std::StdError::generic_err(e.to_string()))?;

    // Get validators and calculate undelegation distribution
    let active_validators = get_active_validators_with_delegations(deps.as_ref())?;
    let undelegation_distribution =
        calculate_validator_distribution(net_unbond_amount, &active_validators)?;

    // Create undelegation messages
    let mut messages: Vec<CosmosMsg> = Vec::new();

    for (validator_addr, undelegation_amount) in undelegation_distribution {
        messages.push(CosmosMsg::Staking(StakingMsg::Undelegate {
            validator: validator_addr.clone(),
            amount: Coin {
                denom: "uregen".to_string(),
                amount: undelegation_amount,
            },
        }));

        // Update validator info
        VALIDATORS.update(
            deps.storage,
            validator_addr.clone(),
            |validator_info: Option<ValidatorInfo>| match validator_info {
                Some(mut info) => {
                    info.delegated_amount = info.delegated_amount.checked_sub(undelegation_amount).map_err(|e| cosmwasm_std::StdError::generic_err(e.to_string()))?;
                    Ok(info)
                }
                None => Err(ContractError::ValidatorNotFound { validator: validator_addr }),
            },
        )?;
    }

    // Burn dREGEN tokens from user
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.dregen_token.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::BurnFrom {
            owner: info.sender.to_string(),
            amount: dregen_amount,
        })?,
        funds: vec![],
    }));

    // Create unbonding request
    let unbonding_id = NEXT_UNBONDING_ID.may_load(deps.storage)?.unwrap_or(0);
    let completion_time = env.block.time.plus_seconds(config.unbonding_period);

    let unbonding_request = UnbondingRequest {
        user: info.sender.clone(),
        dregen_amount,
        regen_amount: net_unbond_amount,
        completion_time,
        nft_token_id: format!("unbond-{}", unbonding_id),
    };

    UNBONDING_REQUESTS.save(deps.storage, unbonding_id, &unbonding_request)?;
    NEXT_UNBONDING_ID.save(deps.storage, &(unbonding_id + 1))?;

    // Update state
    state.total_regen_staked = state.total_regen_staked.checked_sub(net_unbond_amount).map_err(|e| cosmwasm_std::StdError::generic_err(e.to_string()))?;
    state.total_dregen_supply = state.total_dregen_supply.checked_sub(dregen_amount).map_err(|e| cosmwasm_std::StdError::generic_err(e.to_string()))?;
    state.pending_unbonding = state.pending_unbonding.checked_add(net_unbond_amount).map_err(|e| cosmwasm_std::StdError::generic_err(e.to_string()))?;
    state.exchange_rate = current_exchange_rate;
    state.last_update_time = env.block.time;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("method", "unbond")
        .add_attribute("user", info.sender)
        .add_attribute("dregen_amount", dregen_amount)
        .add_attribute("regen_amount", net_unbond_amount)
        .add_attribute("unbonding_id", unbonding_id.to_string())
        .add_attribute("completion_time", completion_time.seconds().to_string())
        .add_attribute("exchange_rate", current_exchange_rate.to_string()))
}

pub fn execute_claim_unbonding(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    unbonding_id: u64,
) -> Result<Response, ContractError> {
    let unbonding_request = UNBONDING_REQUESTS.load(deps.storage, unbonding_id)?;

    // Verify ownership
    if unbonding_request.user != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // Check if unbonding period has completed
    if env.block.time < unbonding_request.completion_time {
        return Err(ContractError::UnbondingNotComplete {
            completion_time: unbonding_request.completion_time.seconds(),
        });
    }

    // Send REGEN tokens to user
    let message = CosmosMsg::Bank(BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![Coin {
            denom: "uregen".to_string(),
            amount: unbonding_request.regen_amount,
        }],
    });

    // Update state
    let mut state = STATE.load(deps.storage)?;
    state.pending_unbonding = state
        .pending_unbonding
        .checked_sub(unbonding_request.regen_amount).map_err(|e| cosmwasm_std::StdError::generic_err(e.to_string()))?;
    STATE.save(deps.storage, &state)?;

    // Remove unbonding request
    UNBONDING_REQUESTS.remove(deps.storage, unbonding_id);

    Ok(Response::new()
        .add_message(message)
        .add_attribute("method", "claim_unbonding")
        .add_attribute("user", info.sender)
        .add_attribute("unbonding_id", unbonding_id.to_string())
        .add_attribute("regen_amount", unbonding_request.regen_amount))
}

pub fn execute_claim_rewards(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Only admin or automated process can claim rewards
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    let active_validators = get_active_validators(deps.as_ref())?;
    let mut messages: Vec<CosmosMsg> = Vec::new();

    // Claim rewards from all validators
    for (validator_addr, _) in active_validators {
        messages.push(CosmosMsg::Distribution(
            DistributionMsg::WithdrawDelegatorReward {
                validator: validator_addr.clone(),
            },
        ));
    }

    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("method", "claim_rewards")
        .add_attribute("claimer", info.sender))
}

pub fn execute_rebalance(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    // For now, restrict to admin
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    // Placeholder: rebalancing logic can be implemented to equalize weights
    Ok(Response::new()
        .add_attribute("method", "rebalance")
        .add_attribute("status", "noop"))
}

pub fn execute_update_validators(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    validators: Vec<ValidatorParams>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    if validators.len() as u32 > config.max_validators {
        return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
            "Validator set exceeds max_validators",
        )));
    }

    // Mark all existing validators as inactive
    let existing: Vec<(String, ValidatorInfo)> = VALIDATORS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|r| r.map_err(ContractError::from))
        .collect::<Result<Vec<_>, _>>()?;

    for (addr, mut v) in existing {
        v.is_active = false;
        VALIDATORS.save(deps.storage, addr.clone(), &v)?;
    }

    // Upsert new validators as active with provided weights
    for vp in validators {
        let _ = deps.api.addr_validate(&vp.address)?; // validate bech32
        let info = VALIDATORS
            .may_load(deps.storage, vp.address.clone())?
            .unwrap_or(ValidatorInfo {
                address: vp.address.clone(),
                delegated_amount: Uint128::zero(),
                weight: vp.weight,
                last_reward_claim: env.block.time,
                slashing_events: 0,
                uptime_percentage: Decimal::percent(100),
                commission_rate: Decimal::percent(10),
                is_active: true,
            });

        VALIDATORS.save(
            deps.storage,
            info.address.clone(),
            &ValidatorInfo {
                is_active: true,
                weight: vp.weight,
                ..info
            },
        )?;
    }

    Ok(Response::new()
        .add_attribute("method", "update_validators")
        .add_attribute("count", VALIDATORS.keys(deps.storage, None, None, Order::Ascending).count().to_string()))
}

pub fn execute_pause(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    config.pause_contract = true;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("method", "pause"))
}

pub fn execute_resume(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    config.pause_contract = false;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("method", "resume"))
}

pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin: Option<String>,
    fee_rate: Option<Decimal>,
    max_validators: Option<u32>,
    dregen_token: Option<String>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(new_admin) = admin {
        config.admin = deps.api.addr_validate(&new_admin)?;
    }
    if let Some(new_fee) = fee_rate {
        if new_fee > Decimal::percent(20) {
            return Err(ContractError::InvalidFeeRate {});
        }
        config.fee_rate = new_fee;
    }
    if let Some(mv) = max_validators {
        config.max_validators = mv;
    }
    if let Some(tok) = dregen_token {
        config.dregen_token = deps.api.addr_validate(&tok)?;
    }

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("method", "update_config"))
}

// Helper functions
fn get_active_validators(deps: Deps) -> Result<Vec<(String, Decimal)>, ContractError> {
    let validators: Result<Vec<_>, ContractError> = VALIDATORS
        .range(deps.storage, None, None, Order::Ascending)
        .filter_map(|item| match item {
            Ok((addr, info)) if info.is_active => Some(Ok((addr, info.weight))),
            Ok(_) => None,
            Err(e) => Some(Err(ContractError::from(e))),
        })
        .collect();
    validators
}

fn get_active_validators_with_delegations(deps: Deps) -> Result<Vec<(String, Decimal)>, ContractError> {
    let validators: Result<Vec<_>, ContractError> = VALIDATORS
        .range(deps.storage, None, None, Order::Ascending)
        .filter_map(|item| match item {
            Ok((addr, info)) if info.is_active && !info.delegated_amount.is_zero() => {
                let weight = Decimal::from_ratio(info.delegated_amount, Uint128::one());
                Some(Ok((addr, weight)))
            }
            Ok(_) => None,
            Err(e) => Some(Err(ContractError::from(e))),
        })
        .collect();
    validators
}

fn query_total_rewards(_deps: Deps, _env: &Env) -> Result<Uint128, ContractError> {
    // TODO: query staking module distribution rewards; placeholder for now
    Ok(Uint128::zero())
}