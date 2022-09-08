
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::state::{ SaleInfo};
use cosmwasm_std::{Decimal, Timestamp, Uint128};


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
  pub token_cost_atom: Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    BuyToken{},
    ClaimToken{},
    ChangeAdmin{ address:String } 
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetStateInfo{}
}
