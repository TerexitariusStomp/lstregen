use cosmwasm_std::testing::{mock_dependencies, MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{OwnedDeps, SystemError, SystemResult};

/// Build a default OwnedDeps with standard mocks. Extend as needed per test case.
pub fn mock_deps() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
    // Prefer the canonical helper, ensures version compatibility
    mock_dependencies()
}

/// Helper to set bank balances in the mock querier if needed in tests.
/// Example:
///   set_bank_balances(&mut deps.querier, &[("addr1", &[(1_000_000u128, "uregen")])]);
pub fn set_bank_balances(querier: &mut MockQuerier, balances: &[(&str, &[(u128, &str)])]) {
    use cosmwasm_std::coin;
    let mut bank = vec![];
    for (addr, bals) in balances {
        let mut cs = vec![];
        for (amt, denom) in *bals {
            cs.push(coin(*amt, *denom));
        }
        bank.push((addr.to_string(), cs));
    }
    querier.update_balance(bank);
}

/// Example custom query handler plumbing if you later introduce custom queries.
/// Currently returns unimplemented.
pub fn handle_custom_query(_bin: &[u8]) -> SystemResult<cosmwasm_std::Binary> {
    SystemResult::Err(SystemError::UnsupportedRequest {
        kind: "custom query not implemented".to_string(),
    })
}