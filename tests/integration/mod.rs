use cosmwasm_std::{coins, Addr, Decimal, Uint128};
use cw_multi_test::{App, ContractWrapper, Executor};

use regen_liquid_staking::contract::{execute, instantiate, query};
use regen_liquid_staking::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

#[test]
fn test_full_staking_cycle() {
    let mut app = App::default();

    // Store contract code
    let code = ContractWrapper::new(execute, instantiate, query);
    let code_id = app.store_code(Box::new(code));

    // Instantiate contract
    let msg = InstantiateMsg {
        admin: "admin".to_string(),
        fee_rate: Decimal::percent(5),
        unbonding_period: 1814400, // 21 days
        max_validators: 10,
        min_delegation: Uint128::new(1_000_000),
        validators: vec![],
    };

    let contract_addr = app
        .instantiate_contract(
            code_id,
            Addr::unchecked("creator"),
            &msg,
            &[],
            "regen-liquid-staking",
            None,
        )
        .unwrap();

    // Test staking
    let stake_msg = ExecuteMsg::Stake {};
    app.execute_contract(
        Addr::unchecked("user"),
        contract_addr.clone(),
        &stake_msg,
        &coins(10_000_000, "uregen"),
    )
    .unwrap();

    // Test unbonding
    let unbond_msg = ExecuteMsg::Unbond {
        dregen_amount: Uint128::new(5_000_000),
    };
    app.execute_contract(
        Addr::unchecked("user"),
        contract_addr.clone(),
        &unbond_msg,
        &[],
    )
    .unwrap();

    // Verify state changes
    let state_query = QueryMsg::State {};
    let state_res: regen_liquid_staking::msg::StateResponse = app
        .wrap()
        .query_wasm_smart(contract_addr, &state_query)
        .unwrap();

    assert!(!state_res.total_regen_staked.is_zero());
}