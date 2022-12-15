
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::Uint128;

use crate::state::{UserInfo, State};


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
  pub admin: String,
  pub token_address: String,
  pub total_supply: Uint128,
  pub presale_start: u64,
  pub presale_period: u64,
  pub vesting_step_period: u64,
  pub claim_start: u64,
  pub token_cost_juno: Uint128,
  pub token_cost_atom: Uint128,
  pub token_cost_usdc: Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    BuyToken{},
    ClaimToken{},
    ChangeAdmin{ address:String },
    UpdateConfig{ state: State},
    WithdrawTokenByAdmin {} 
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetStateInfo{},
    GetUserInfo{ address:String},
    GetSaleInfo{},
    GetClaimableAmount{ address:String },
    GetClaimableTime{ address:String },
    GetUserInfos{ start_after: Option<String>, limit:Option<u32>}
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserInfosResponse{
  pub user_info: Vec<UserInfo>
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserInfoResponse{
 pub user_info: UserInfo
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TimeInfo{
 pub crr_time: u64,
 pub claimable_time: u64
}
