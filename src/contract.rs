use cosmwasm_std::{
    entry_point, from_binary, to_binary, Addr, Binary, CanonicalAddr, CosmosMsg, Decimal, Deps,
    DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128, WasmMsg,
};

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
    StakerInfoResponse, StateResponse,
};
use crate::state::{
    staker_info_key, staker_info_storage, Config, StakerInfo, State, CONFIG, STATE,
};

use cw2::{get_contract_version, set_contract_version};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

use std::collections::BTreeMap;

const CONTRACT_NAME: &str = "Hope_Staking";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    deps.api.addr_validate(&msg.lp_token_contract)?;
    deps.api.addr_validate(&msg.reward_token_contract)?;

    CONFIG.save(
        deps.storage,
        &Config {
            lp_token_contract: msg.lp_token_contract,
            reward_token_contract: msg.reward_token_contract,
            distribution_schedule: msg.distribution_schedule,
            admin: info.sender.to_string(),
        },
    )?;

    STATE.save(
        deps.storage,
        &State {
            last_distributed: env.block.time.seconds(),
            total_bond_amount: Uint128::zero(),
            global_reward_index: Decimal::zero(),
        },
    )?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::Unbond { amount } => unbond(deps, env, info, amount),
        ExecuteMsg::Withdraw {} => withdraw(deps, env, info),
        ExecuteMsg::UpdateConfig {
            distribution_schedule,
        } => update_config(deps, env, info, distribution_schedule),
        ExecuteMsg::MigrateStaking {
            new_staking_contract,
        } => migrate_staking(deps, env, info, new_staking_contract),
        ExecuteMsg::UpdateAdmin { admin } => update_admin(deps, info, admin),
        ExecuteMsg::UpdateTokenContract {
            lp_token_contract,
            reward_token_contract,
        } => update_token_contract(deps, info, lp_token_contract, reward_token_contract),
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let token_contract = info.sender.to_string();

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Bond {}) => {
            // only staking token contract can execute this message
            if config.lp_token_contract != token_contract {
                return Err(ContractError::WrongContractError {});
            }

            let cw20_sender = cw20_msg.sender;
            bond(deps, env, cw20_sender, cw20_msg.amount)
        }
        Err(_) => return Err(ContractError::DataShouldBeGiven {}),
    }
}

pub fn bond(
    deps: DepsMut,
    env: Env,
    sender_addr: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;

    let staker_info_key = staker_info_key(&sender_addr);
    let mut staker_info: StakerInfo;
    match staker_info_storage().may_load(deps.storage, staker_info_key.clone())? {
        Some(some_staker_info) => staker_info = some_staker_info,
        None => {
            staker_info = StakerInfo {
                reward_index: Decimal::zero(),
                bond_amount: Uint128::zero(),
                pending_reward: Uint128::zero(),
                address: sender_addr.clone(),
            }
        }
    };

    // Compute global reward & staker reward
    compute_reward(&config, &mut state, env.block.time.seconds());
    compute_staker_reward(&state, &mut staker_info)?;

    // Increase bond_amount
    increase_bond_amount(&mut state, &mut staker_info, amount);

    // Store updated state with staker's staker_info
    staker_info_storage().save(deps.storage, staker_info_key.clone(), &staker_info)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "bond"),
        ("owner", sender_addr.as_str()),
        ("amount", amount.to_string().as_str()),
    ]))
}

pub fn unbond(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;
    let sender_addr = info.sender.to_string();

    let staker_info_key = staker_info_key(&sender_addr);
    let mut staker_info: StakerInfo;
    match staker_info_storage().may_load(deps.storage, staker_info_key.clone())? {
        Some(some_staker_info) => staker_info = some_staker_info,
        None => return Err(ContractError::NotStaked {}),
    };

    if staker_info.bond_amount < amount {
        return Err(ContractError::ExceedBondAmount {});
    }

    // Compute global reward & staker reward
    compute_reward(&config, &mut state, env.block.time.seconds());
    compute_staker_reward(&state, &mut staker_info)?;

    // Decrease bond_amount
    decrease_bond_amount(&mut state, &mut staker_info, amount)?;

    // Store or remove updated rewards info
    // depends on the left pending reward and bond amount
    if staker_info.pending_reward.is_zero() && staker_info.bond_amount.is_zero() {
        staker_info_storage().remove(deps.storage, staker_info_key)?;
    } else {
        staker_info_storage().save(deps.storage, staker_info_key, &staker_info)?;
    }

    // Store updated state
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.lp_token_contract,
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: info.sender.to_string(),
                amount,
            })?,
            funds: vec![],
        })])
        .add_attributes(vec![
            ("action", "unbond"),
            ("owner", info.sender.as_str()),
            ("amount", amount.to_string().as_str()),
        ]))
}

// withdraw rewards to executor
pub fn withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let sender_addr = info.sender.to_string();

    let config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;

    let staker_info_key = staker_info_key(&sender_addr);
    let mut staker_info: StakerInfo;
    match staker_info_storage().may_load(deps.storage, staker_info_key.clone())? {
        Some(some_staker_info) => staker_info = some_staker_info,
        None => return Err(ContractError::NotStaked {}),
    };

    // Compute global reward & staker reward
    compute_reward(&config, &mut state, env.block.time.seconds());
    compute_staker_reward(&state, &mut staker_info)?;

    let amount = staker_info.pending_reward;
    staker_info.pending_reward = Uint128::zero();

    // Store or remove updated rewards info
    // depends on the left pending reward and bond amount
    if staker_info.bond_amount.is_zero() {
        staker_info_storage().remove(deps.storage, staker_info_key)?;
    } else {
        staker_info_storage().save(deps.storage, staker_info_key, &staker_info)?;
    }

    // Store updated state
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.reward_token_contract,
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: info.sender.to_string(),
                amount,
            })?,
            funds: vec![],
        })])
        .add_attributes(vec![
            ("action", "withdraw"),
            ("owner", info.sender.as_str()),
            ("amount", amount.to_string().as_str()),
        ]))
}

pub fn migrate_staking(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    new_staking_contract: String,
) -> Result<Response, ContractError> {
    let sender_addr = info.sender.to_string();

    let mut config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;

    if sender_addr != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    // compute global reward, sets last_distributed_seconds to env.block.time.seconds
    compute_reward(&config, &mut state, env.block.time.seconds());

    let total_distribution_amount: Uint128 =
        config.distribution_schedule.iter().map(|item| item.2).sum();

    let block_time = env.block.time.seconds();
    // eliminate distribution slots that have not started
    config
        .distribution_schedule
        .retain(|slot| slot.0 < block_time);

    let mut distributed_amount = Uint128::zero();
    for s in config.distribution_schedule.iter_mut() {
        if s.1 < block_time {
            // all distributed
            distributed_amount += s.2;
        } else {
            // partially distributed slot
            let whole_time = s.1 - s.0;
            let distribution_amount_per_second: Decimal = Decimal::from_ratio(s.2, whole_time);

            let passed_time = block_time - s.0;
            let distributed_amount_on_slot =
                distribution_amount_per_second * Uint128::from(passed_time as u128);
            distributed_amount += distributed_amount_on_slot;

            // modify distribution slot
            s.1 = block_time;
            s.2 = distributed_amount_on_slot;
        }
    }

    // update config
    CONFIG.save(deps.storage, &config)?;
    // update state
    STATE.save(deps.storage, &state)?;

    let remaining_lp = total_distribution_amount
        .checked_sub(distributed_amount)
        .unwrap_or_default();

    Ok(Response::new()
        .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.lp_token_contract,
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: new_staking_contract,
                amount: remaining_lp,
            })?,
            funds: vec![],
        })])
        .add_attributes(vec![
            ("action", "migrate_staking"),
            ("distributed_amount", &distributed_amount.to_string()),
            ("remaining_amount", &remaining_lp.to_string()),
        ]))
}

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    distribution_schedule: Vec<(u64, u64, Uint128)>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;

    authcheck(deps.as_ref(), &info)?;

    assert_new_schedules(&config, &state, distribution_schedule.clone())?;

    let new_config = Config {
        admin: config.admin,
        lp_token_contract: config.lp_token_contract,
        reward_token_contract: config.reward_token_contract,
        distribution_schedule,
    };
    CONFIG.save(deps.storage, &new_config)?;

    Ok(Response::new().add_attributes(vec![("action", "update_config")]))
}

pub fn update_admin(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    deps.api.addr_validate(&address)?;

    authcheck(deps.as_ref(), &info)?;
    config.admin = address;

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![("action", "update_admin")]))
}

pub fn update_token_contract(
    deps: DepsMut,
    info: MessageInfo,
    lp_contract: String,
    reward_contract: String,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    deps.api.addr_validate(&lp_contract)?;
    deps.api.addr_validate(&reward_contract)?;

    authcheck(deps.as_ref(), &info)?;
    config.reward_token_contract = reward_contract;
    config.lp_token_contract = lp_contract;

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![("action", "update_admin")]))
}

fn authcheck(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

pub fn assert_new_schedules(
    config: &Config,
    state: &State,
    distribution_schedule: Vec<(u64, u64, Uint128)>,
) -> Result<(), ContractError> {
    if distribution_schedule.len() < config.distribution_schedule.len() {
        return Err(ContractError::NotIncludeAllDistributionSchedule {});
    }

    let mut existing_counts: BTreeMap<(u64, u64, Uint128), u32> = BTreeMap::new();
    for schedule in config.distribution_schedule.clone() {
        let counter = existing_counts.entry(schedule).or_insert(0);
        *counter += 1;
    }

    let mut new_counts: BTreeMap<(u64, u64, Uint128), u32> = BTreeMap::new();
    for schedule in distribution_schedule {
        let counter = new_counts.entry(schedule).or_insert(0);
        *counter += 1;
    }

    for (schedule, count) in existing_counts.into_iter() {
        // if began ensure its in the new schedule
        if schedule.0 <= state.last_distributed {
            if count > *new_counts.get(&schedule).unwrap_or(&0u32) {
                return Err(ContractError::NewScheduleRemovePastDistribution {});
            }
            // after this new_counts will only contain the newly added schedules
            *new_counts.get_mut(&schedule).unwrap() -= count;
        }
    }

    for (schedule, count) in new_counts.into_iter() {
        if count > 0 && schedule.0 <= state.last_distributed {
            return Err(ContractError::NewScheduleAddPastDistribution {});
        }
    }
    Ok(())
}

// compute distributed rewards and update global reward index
pub fn compute_reward(config: &Config, state: &mut State, block_time: u64) {
    if state.total_bond_amount.is_zero() {
        state.last_distributed = block_time;
        return;
    }

    let mut distributed_amount: Uint128 = Uint128::zero();
    for s in config.distribution_schedule.iter() {
        if s.0 > block_time || s.1 < state.last_distributed {
            continue;
        }

        // min(s.1, block_time) - max(s.0, last_distributed)
        let passed_time =
            std::cmp::min(s.1, block_time) - std::cmp::max(s.0, state.last_distributed);

        let time = s.1 - s.0;
        let distribution_amount_per_second: Decimal = Decimal::from_ratio(s.2, time);
        distributed_amount += distribution_amount_per_second * Uint128::from(passed_time as u128);
    }

    state.last_distributed = block_time;
    state.global_reward_index = state.global_reward_index
        + Decimal::from_ratio(distributed_amount, state.total_bond_amount);
}

// withdraw reward to pending reward
pub fn compute_staker_reward(state: &State, staker_info: &mut StakerInfo) -> StdResult<()> {
    let pending_reward = (staker_info.bond_amount * state.global_reward_index)
        .checked_sub(staker_info.bond_amount * staker_info.reward_index)?;

    staker_info.reward_index = state.global_reward_index;
    staker_info.pending_reward += pending_reward;
    Ok(())
}

fn increase_bond_amount(state: &mut State, staker_info: &mut StakerInfo, amount: Uint128) {
    state.total_bond_amount += amount;
    staker_info.bond_amount += amount;
}

fn decrease_bond_amount(
    state: &mut State,
    staker_info: &mut StakerInfo,
    amount: Uint128,
) -> StdResult<()> {
    state.total_bond_amount = state.total_bond_amount.checked_sub(amount)?;
    staker_info.bond_amount = staker_info.bond_amount.checked_sub(amount)?;
    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let version = get_contract_version(deps.storage)?;
    if version.contract != CONTRACT_NAME {
        return Err(ContractError::CannotMigrate {
            previous_contract: version.contract,
        });
    }
    if version.version != CONTRACT_VERSION {
        return Err(ContractError::CannotMigrate {
            previous_contract: version.contract,
        });
    }
    Ok(Response::default())
}
