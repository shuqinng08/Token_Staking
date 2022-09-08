use crate::msg::{
    QueryMsg
};
use crate::state::{
     State, CONFIG, UserInfo,  SaleInfo, USERINFO, SALEINFO
};
use cosmwasm_std::{entry_point, to_binary, Addr, Binary, Deps, Env, Order, StdResult, Uint128, Decimal};
use cw_storage_plus::{Bound, PrefixBound};

// Query limits
const DEFAULT_QUERY_LIMIT: u32 = 10;
const MAX_QUERY_LIMIT: u32 = 30;


#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetStateInfo { } => to_binary(&query_state_info(deps)?),
        QueryMsg::GetUserInfo { address} => to_binary(&query_user_info(deps,address)?),
        QueryMsg::GetSaleInfo { } => to_binary(&query_sale_info(deps)?), 
        QueryMsg::GetClaimableAmount { address } => to_binary(&query_claimable_amount(deps,env,address)?),
        QueryMsg::GetClaimableTime { address } => to_binary(&query_claimable_time(deps,env,address)?)
    }
}

pub fn query_state_info(deps:Deps) -> StdResult<State>{
    let state =  CONFIG.load(deps.storage)?;
    Ok(state)
}

pub fn query_user_info(deps:Deps, address:String) -> StdResult<UserInfo>{
    let user_info = USERINFO.may_load(deps.storage, &address)?;
    deps.api.addr_validate(&address)?;
    match user_info {
        Some(user_info) =>{
            Ok(user_info)
        },
        None => {
            let user_info = UserInfo{
                address,
                total_claim_amount: Uint128::zero(),
                sent_atom: Uint128::zero(),
                sent_juno: Uint128::zero(),
                claimed_amount: Uint128::zero(),
                vesting_step: 0,
                last_received: 0
            };
            Ok(user_info)
        }
    }
}

fn query_sale_info(deps: Deps) -> StdResult<SaleInfo>{
    let sale_info = SALEINFO.load(deps.storage)?;
    Ok(sale_info)
}

fn query_claimable_amount(deps: Deps,env: Env,sender: String ) -> StdResult<Uint128>{
    let state = CONFIG.load(deps.storage)?;

    //presale start validation check
    let crr_time = env.block.time.seconds();
    let presale_end = state.presale_start + state.presale_period;
    let user_info = USERINFO.may_load(deps.storage, &sender)?;

    
    let first_portion = Decimal::from_ratio(1 as u128, 10 as u128);
    let default_portion = Decimal::from_ratio(15 as u128, 100 as u128);

    if crr_time < presale_end{
          Ok(Uint128::zero())
    }
    else{
          if crr_time <  state.claim_start{
            
            match user_info {
                Some(user_info) =>{
                    if user_info.vesting_step == 0{
                        let token_amount_to_send = first_portion * user_info.total_claim_amount;
                        Ok(token_amount_to_send)
                    }
                    else{
                        Ok(Uint128::zero())
                    }
                },
                None =>{
                    Ok(Uint128::zero())
                }
            }
        }  else{
            match  user_info {
                Some(user_info) => {
                    let mut expect_step = (crr_time - state.claim_start)/state.vesting_step_period + 2;
                    if expect_step > 7 {
                       expect_step = 7;
                    }
                    if user_info.vesting_step == expect_step{
                        Ok(Uint128::zero())
                    }
                    else{
                    
                        if user_info.vesting_step == 0{
                            let token_amount_to_send = first_portion * user_info.total_claim_amount +  Uint128::from((expect_step-1) as u128) * user_info.total_claim_amount * default_portion;
                            Ok(token_amount_to_send)
                        }
                        else{
                            let token_amount_to_send = Uint128::from((expect_step-1) as u128) * user_info.total_claim_amount * default_portion;
                            Ok(token_amount_to_send)
                        }
                    
                    }
                },  
                None => {
                    Ok(Uint128::zero())
                }
            }
        }
       
    }
  
}

fn query_claimable_time(deps: Deps,env:Env, sender: String) -> StdResult<u64>{
    let claimable_amount = query_claimable_amount(deps,env.clone(),sender.clone())?;
    let state = CONFIG.load(deps.storage)?;
    let crr_time = env.block.time.seconds();
    if claimable_amount > Uint128::zero(){
        Ok(0)
    }
    else{
        let user_info = USERINFO.may_load(deps.storage, &sender)?;
        match user_info {
            Some(_user_info) => {
                let mut claimable_time:u64 = 0;
                for i in 0..6{
                    if crr_time < state.claim_start + state.vesting_step_period*i{
                        claimable_time = state.claim_start + state.vesting_step_period*i;
                        break;
                    }
                }
                Ok(claimable_time)
            },
            None =>{
                Ok(0)
            }
        }
    }
    
}