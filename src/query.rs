use crate::msg::{
    QueryMsg
};
use crate::state::{
     State, CONFIG, UserInfo,  SaleInfo
};
use cosmwasm_std::{entry_point, to_binary, Addr, Binary, Deps, Env, Order, StdResult, Uint128};
use cw_storage_plus::{Bound, PrefixBound};

// Query limits
const DEFAULT_QUERY_LIMIT: u32 = 10;
const MAX_QUERY_LIMIT: u32 = 30;


#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let api = deps.api;

    match msg {
        QueryMsg::GetStateInfo {} => to_binary(&query_state_info(deps)?),
       
    }
}

pub fn query_state_info(deps:Deps) -> StdResult<State>{
    let state =  CONFIG.load(deps.storage)?;
    Ok(state)
}
