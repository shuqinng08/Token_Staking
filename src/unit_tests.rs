#[cfg(test)]
use crate::contract::{execute, instantiate};
use crate::msg::{Cw20HookMsg, ExecuteMsg, InstantiateMsg};
use crate::query::query_staker_info;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{to_binary, BankMsg, Coin, CosmosMsg, DepsMut, Env, Uint128, WasmMsg};

use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

fn setup_contract(deps: DepsMut, env: Env) {
    let instantiate_msg = InstantiateMsg {
        lp_token_contract: "lp_token_contract".to_string(),
        reward_token_contract: "reward_token_contract".to_string(),
        distribution_schedule: vec![(
            env.block.time.seconds(),
            env.block.time.seconds() + 86400,
            Uint128::new(100000000),
        )],
        lock_duration: 3600,
    };
    let info = mock_info("owner", &[]);
    let res = instantiate(deps, mock_env(), info, instantiate_msg).unwrap();
    assert_eq!(res.messages.len(), 0);
}

#[test]
fn test_earned_and_unbond() {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    setup_contract(deps.as_mut(), env.clone());

    let info = mock_info("lp_token_contract", &[]);
    let hook_msg = Cw20HookMsg::Bond {};
    let cw20_rcv_msg = Cw20ReceiveMsg {
        sender: "user1".to_string(),
        amount: Uint128::new(100),
        msg: to_binary(&hook_msg).unwrap(),
    };
    let msg = ExecuteMsg::Receive(cw20_rcv_msg);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let staker_info = query_staker_info(deps.as_ref(), env.clone(), "user1".to_string()).unwrap();
    println!("{:?}", staker_info)
}
