#![cfg(test)]

use super::*;
use crate::contract::{execute, instantiate, query};
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, ValidatorParams,
};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coins, from_binary, Decimal, Uint128};

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies();

    let msg = InstantiateMsg {
        admin: "admin".to_string(),
        fee_rate: Decimal::percent(5),
        unbonding_period: 21 * 24 * 60 * 60, // 21 days
        max_validators: 10,
        min_delegation: Uint128::new(1_000_000),
        validators: vec![
            ValidatorParams {
                address: "regenvaloper1test1".to_string(),
                weight: Decimal::percent(50),
            },
            ValidatorParams {
                address: "regenvaloper1test2".to_string(),
                weight: Decimal::percent(50),
            },
        ],
    };

    let info = mock_info("creator", &coins(1000, "uregen"));
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_eq!(0, res.messages.len());

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let cfg: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!("admin", cfg.admin);
    assert_eq!(Decimal::percent(5), cfg.fee_rate);
}

#[test]
fn test_stake_flow() {
    let mut deps = mock_dependencies();

    // instantiate
    let inst = InstantiateMsg {
        admin: "admin".to_string(),
        fee_rate: Decimal::percent(5),
        unbonding_period: 21 * 24 * 60 * 60,
        max_validators: 10,
        min_delegation: Uint128::new(1_000_000),
        validators: vec![ValidatorParams {
            address: "regenvaloper1test1".to_string(),
            weight: Decimal::percent(100),
        }],
    };
    let info = mock_info("creator", &[]);
    instantiate(deps.as_mut(), mock_env(), info, inst).unwrap();

    // stake
    let stake_info = mock_info("user", &coins(10_000_000, "uregen"));
    let res = execute(
        deps.as_mut(),
        mock_env(),
        stake_info,
        ExecuteMsg::Stake {},
    )
    .unwrap();

    // expect delegate and mint messages
    assert!(res.messages.len() >= 2);
}