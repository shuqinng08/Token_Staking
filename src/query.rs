use crate::contract::{compute_reward, compute_staker_reward};
use crate::msg::{ConfigResponse, QueryMsg, StakerInfoResponse, StateResponse};
use crate::state::{staker_info_key, staker_info_storage, CONFIG, STATE};
use cosmwasm_std::{entry_point, to_binary, Binary, Decimal, Deps, Env, Order, StdResult, Uint128};
use cw_storage_plus::Bound;

// Query limits
const DEFAULT_QUERY_LIMIT: u32 = 10;
const MAX_QUERY_LIMIT: u32 = 30;

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::State { block_time } => to_binary(&query_state(deps, block_time)?),
        QueryMsg::StakerInfo { staker, block_time } => {
            to_binary(&query_staker_info(deps, staker, block_time)?)
        }
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        lp_token_contract: config.lp_token_contract,
        reward_token_contract: config.reward_token_contract,
        distribution_schedule: config.distribution_schedule,
        admin: config.admin,
    })
}

pub fn query_state(deps: Deps, block_time: Option<u64>) -> StdResult<StateResponse> {
    let mut state = STATE.load(deps.storage)?;
    if let Some(block_time) = block_time {
        let config = CONFIG.load(deps.storage)?;
        compute_reward(&config, &mut state, block_time);
    }

    Ok(StateResponse {
        last_distributed: state.last_distributed,
        total_bond_amount: state.total_bond_amount,
        global_reward_index: state.global_reward_index,
    })
}

pub fn query_staker_info(
    deps: Deps,
    staker: String,
    block_time: Option<u64>,
) -> StdResult<StakerInfoResponse> {
    let staker_info_key = staker_info_key(&staker);
    match staker_info_storage().may_load(deps.storage, staker_info_key)? {
        Some(some_staker_info) => {
            let mut staker_info = some_staker_info;
            if let Some(block_time) = block_time {
                let config = CONFIG.load(deps.storage)?;
                let mut state = STATE.load(deps.storage)?;

                compute_reward(&config, &mut state, block_time);
                compute_staker_reward(&state, &mut staker_info)?;
            }

            Ok(StakerInfoResponse {
                staker,
                reward_index: staker_info.reward_index,
                bond_amount: staker_info.bond_amount,
                pending_reward: staker_info.pending_reward,
            })
        }
        None => Ok(StakerInfoResponse {
            staker,
            reward_index: Decimal::zero(),
            bond_amount: Uint128::zero(),
            pending_reward: Uint128::zero(),
        }),
    }
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
