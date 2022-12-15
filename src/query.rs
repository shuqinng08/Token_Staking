use crate::msg::{QueryMsg, TimeInfo, UserInfoResponse, UserInfosResponse};
use crate::state::{user_info_key, user_info_storage, SaleInfo, State, UserInfo, CONFIG, SALEINFO};
use cosmwasm_std::{entry_point, to_binary, Binary, Decimal, Deps, Env, Order, StdResult, Uint128};
use cw_storage_plus::Bound;

// Query limits
const DEFAULT_QUERY_LIMIT: u32 = 10;
const MAX_QUERY_LIMIT: u32 = 30;

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetStateInfo {} => to_binary(&query_state_info(deps)?),
        QueryMsg::GetUserInfo { address } => to_binary(&query_user_info(deps, address)?),
        QueryMsg::GetSaleInfo {} => to_binary(&query_sale_info(deps)?),
        QueryMsg::GetClaimableAmount { address } => {
            to_binary(&query_claimable_amount(deps, env, address)?)
        }
        QueryMsg::GetClaimableTime { address } => {
            to_binary(&query_claimable_time(deps, env, address)?)
        }
        QueryMsg::GetUserInfos { start_after, limit } => {
            to_binary(&query_get_user_infos(deps, start_after, limit)?)
        }
    }
}

pub fn query_state_info(deps: Deps) -> StdResult<State> {
    let state = CONFIG.load(deps.storage)?;
    Ok(state)
}

pub fn query_user_info(deps: Deps, address: String) -> StdResult<UserInfoResponse> {
    let user_info_key = user_info_key(&address);
    let user_info = user_info_storage().may_load(deps.storage, user_info_key)?;
    deps.api.addr_validate(&address)?;
    match user_info {
        Some(user_info) => Ok(UserInfoResponse { user_info }),
        None => {
            let user_info = UserInfo {
                address,
                total_claim_amount: Uint128::zero(),
                sent_atom: Uint128::zero(),
                sent_juno: Uint128::zero(),
                sent_usdc: Uint128::zero(),
                claimed_amount: Uint128::zero(),
                vesting_step: 0,
                last_received: 0,
            };
            Ok(UserInfoResponse { user_info })
        }
    }
}

pub fn query_sale_info(deps: Deps) -> StdResult<SaleInfo> {
    let sale_info = SALEINFO.load(deps.storage)?;
    Ok(sale_info)
}

pub fn query_claimable_amount(deps: Deps, env: Env, sender: String) -> StdResult<Uint128> {
    let state = CONFIG.load(deps.storage)?;

    //presale start validation check
    let crr_time = env.block.time.seconds();
    let presale_end = state.presale_start + state.presale_period;
    let user_info_key = user_info_key(&sender);
    let user_info = user_info_storage().may_load(deps.storage, user_info_key)?;

    let first_portion = Decimal::from_ratio(1 as u128, 10 as u128);
    let default_portion = Decimal::from_ratio(15 as u128, 100 as u128);

    if crr_time < presale_end {
        Ok(Uint128::zero())
    } else {
        if crr_time < state.claim_start {
            match user_info {
                Some(user_info) => {
                    if user_info.vesting_step == 0 {
                        let token_amount_to_send = first_portion * user_info.total_claim_amount;
                        Ok(token_amount_to_send)
                    } else {
                        Ok(Uint128::zero())
                    }
                }
                None => Ok(Uint128::zero()),
            }
        } else {
            match user_info {
                Some(user_info) => {
                    let mut expect_step =
                        (crr_time - state.claim_start) / state.vesting_step_period + 2;
                    if expect_step > 7 {
                        expect_step = 7;
                    }
                    if user_info.vesting_step == expect_step {
                        Ok(Uint128::zero())
                    } else {
                        if user_info.vesting_step == 0 {
                            let token_amount_to_send = first_portion * user_info.total_claim_amount
                                + Uint128::from((expect_step - 1) as u128)
                                    * user_info.total_claim_amount
                                    * default_portion;
                            Ok(token_amount_to_send)
                        } else {
                            let token_amount_to_send =
                                Uint128::from((expect_step - user_info.vesting_step) as u128)
                                    * user_info.total_claim_amount
                                    * default_portion;
                            Ok(token_amount_to_send)
                        }
                    }
                }
                None => Ok(Uint128::zero()),
            }
        }
    }
}

pub fn query_claimable_time(deps: Deps, env: Env, sender: String) -> StdResult<TimeInfo> {
    let claimable_amount = query_claimable_amount(deps, env.clone(), sender.clone())?;
    let state = CONFIG.load(deps.storage)?;
    let crr_time = env.block.time.seconds();

    if claimable_amount > Uint128::zero() {
        Ok(TimeInfo {
            claimable_time: 0,
            crr_time,
        })
    } else {
        let user_info_key = user_info_key(&sender);
        let user_info = user_info_storage().may_load(deps.storage, user_info_key)?;
        match user_info {
            Some(_user_info) => {
                let mut claimable_time: u64 = 0;
                for i in 0..6 {
                    if crr_time < state.claim_start + state.vesting_step_period * i {
                        claimable_time = state.claim_start + state.vesting_step_period * i;
                        break;
                    }
                }
                Ok(TimeInfo {
                    claimable_time,
                    crr_time,
                })
            }
            None => Ok(TimeInfo {
                claimable_time: 0,
                crr_time,
            }),
        }
    }
}

pub fn query_get_user_infos(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<UserInfosResponse> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

    let user_info = user_info_storage()
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;
    Ok(UserInfosResponse { user_info })
}
