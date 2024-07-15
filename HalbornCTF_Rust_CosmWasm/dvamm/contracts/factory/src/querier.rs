use dvamm::asset::PairInfo;
use dvamm::pair::QueryMsg;
use cosmwasm_std::{to_json_binary, Addr, Deps, QueryRequest, StdResult, WasmQuery};

pub fn query_pair_info(deps: Deps, pair_contract: &Addr) -> StdResult<PairInfo> {
    deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: pair_contract.to_string(),
        msg: to_json_binary(&QueryMsg::Pair {})?,
    }))
}
