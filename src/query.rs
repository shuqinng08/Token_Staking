use crate::msg::{ConfigResponse, QueryMsg};
use crate::state::CONFIG;
use cosmwasm_std::{entry_point, to_binary, Binary, Decimal, Deps, Env, Order, StdResult, Uint128};
use cw_storage_plus::Bound;

// Query limits
const DEFAULT_QUERY_LIMIT: u32 = 10;
const MAX_QUERY_LIMIT: u32 = 30;

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        token_address: config.token_contract,
        distribution_schedule: config.distribution_schedule,
        admin: config.admin,
    })
}

// pub fn query_get_user_infos(
//     deps: Deps,
//     start_after: Option<String>,
//     limit: Option<u32>,
// ) -> StdResult<UserInfosResponse> {
//     let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
//     let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

//     let user_info = user_info_storage()
//         .range(deps.storage, start, None, Order::Ascending)
//         .take(limit)
//         .map(|res| res.map(|item| item.1))
//         .collect::<StdResult<Vec<_>>>()?;
//     Ok(UserInfosResponse { user_info })
// }
