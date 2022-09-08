use cosmwasm_std::{Uint128, Decimal, Timestamp, BlockInfo};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw_storage_plus::{Item,Map,MultiIndex,IndexList,Index,IndexedMap};

pub const CONFIG: Item<State> = Item::new("config_state");
pub const SALEINFO: Item<SaleInfo> = Item::new("config_sale_info");
pub const COININFO: Map<&str, bool> = Map::new("config_token_info");
pub const USERINFO: Map<&str, UserInfo> = Map::new("config_user_info");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
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
pub struct SaleInfo {
  pub token_sold_amount: Uint128,
  pub earned_juno: Uint128,
  pub earned_atom: Uint128
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserInfo {
  pub address: String,
  pub total_claim_amount: Uint128,
  pub sent_atom: Uint128,
  pub sent_juno: Uint128,
  pub claimed_amount: Uint128,
  pub vesting_step: u64,
  pub last_received: u64
}


