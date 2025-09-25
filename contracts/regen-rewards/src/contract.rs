use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdError, StdResult, Uint128,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use crate::msg::{
    ClaimHistoryResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg,
    RewardHistoryResponse,
};
use crate::state::{
    ClaimRecord, Config, RewardRecord, CLAIM_HISTORY, CONFIG, NEXT_REWARD_ID, REWARD_HISTORY,
};

const CONTRACT_NAME: &str = "crates.io:regen-rewards";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, StdError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let admin: Addr = deps.api.addr_validate(&msg.admin)?;
    let distributor: Addr = deps.api.addr_validate(&msg.distributor)?;

    CONFIG.save(
        deps.storage,
        &Config {
            admin: admin.clone(),
            distributor,
            reward_denom: msg.reward_denom,
        },
    )?;
    NEXT_REWARD_ID.save(deps.storage, &0u64)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", admin)
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION))
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> Result<Response, StdError> {
    match msg {
        ExecuteMsg::SetDistributor { distributor } => {
            let mut cfg = CONFIG.load(deps.storage)?;
            ensure_admin(&info.sender, &cfg)?;
            cfg.distributor = deps.api.addr_validate(&distributor)?;
            CONFIG.save(deps.storage, &cfg)?;
            Ok(Response::new()
                .add_attribute("action", "set_distributor")
                .add_attribute("distributor", distributor))
        }
        ExecuteMsg::TransferAdmin { new_admin } => {
            let mut cfg = CONFIG.load(deps.storage)?;
            ensure_admin(&info.sender, &cfg)?;
            cfg.admin = deps.api.addr_validate(&new_admin)?;
            CONFIG.save(deps.storage, &cfg)?;
            Ok(Response::new()
                .add_attribute("action", "transfer_admin")
                .add_attribute("new_admin", new_admin))
        }
        ExecuteMsg::RecordReward { validator, amount } => {
            let cfg = CONFIG.load(deps.storage)?;
            ensure_distributor(&info.sender, &cfg)?;

            let mut next_id = NEXT_REWARD_ID.load(deps.storage)?;
            let rec = RewardRecord {
                id: next_id,
                validator,
                amount,
                timestamp: env.block.time,
            };
            REWARD_HISTORY.save(deps.storage, next_id, &rec)?;
            next_id += 1;
            NEXT_REWARD_ID.save(deps.storage, &next_id)?;

            Ok(Response::new()
                .add_attribute("action", "record_reward")
                .add_attribute("id", (next_id - 1).to_string())
                .add_attribute("amount", amount))
        }
        ExecuteMsg::RecordClaim { user, amount } => {
            let cfg = CONFIG.load(deps.storage)?;
            // Restrict to distributor to avoid spoofing; can be relaxed if needed.
            ensure_distributor(&info.sender, &cfg)?;
            let addr = deps.api.addr_validate(&user)?;

            // Use timestamp nanos as a pseudo key to keep append-only ordering
            // In practice you might track NEXT_CLAIM_ID as well; simplified here.
            let key = env.block.time.nanos();
            let rec = ClaimRecord {
                user: addr.clone(),
                amount,
                timestamp: env.block.time,
            };
            CLAIM_HISTORY.save(deps.storage, key, &rec)?;
            Ok(Response::new()
                .add_attribute("action", "record_claim")
                .add_attribute("user", addr)
                .add_attribute("amount", amount))
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::RewardHistory { start_after, limit } => {
            to_binary(&query_reward_history(deps, start_after, limit)?)
        }
        QueryMsg::ClaimHistory { start_after, limit } => {
            to_binary(&query_claim_history(deps, start_after, limit)?)
        }
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        admin: cfg.admin.to_string(),
        distributor: cfg.distributor.to_string(),
        reward_denom: cfg.reward_denom,
    })
}

fn query_reward_history(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<RewardHistoryResponse> {
    let limit = limit.unwrap_or(50).min(200) as usize;
    let start = start_after.map(Bound::exclusive);
    let items = REWARD_HISTORY
        .range(deps.storage, start, None, Order::Descending)
        .take(limit)
        .map(|r| r.map(|(_, v)| v))
        .collect::<StdResult<Vec<_>>>()?;
    Ok(RewardHistoryResponse { records: items })
}

fn query_claim_history(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<ClaimHistoryResponse> {
    let limit = limit.unwrap_or(50).min(200) as usize;
    let start = start_after.map(Bound::exclusive);
    let items = CLAIM_HISTORY
        .range(deps.storage, start, None, Order::Descending)
        .take(limit)
        .map(|r| r.map(|(_, v)| v))
        .collect::<StdResult<Vec<_>>>()?;
    Ok(ClaimHistoryResponse { records: items })
}

fn ensure_admin(sender: &Addr, cfg: &Config) -> Result<(), StdError> {
    if sender != &cfg.admin {
        return Err(StdError::generic_err("unauthorized"));
    }
    Ok(())
}

fn ensure_distributor(sender: &Addr, cfg: &Config) -> Result<(), StdError> {
    if sender != &cfg.distributor {
        return Err(StdError::generic_err("unauthorized"));
    }
    Ok(())
}