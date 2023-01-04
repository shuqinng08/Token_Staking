use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CONFIG: Item<Config> = Item::new("config_config");
pub const STATE: Item<State> = Item::new("config_state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub token_contract: String,
    pub distribution_schedule: Vec<(u64, u64, Uint128)>,
    pub admin: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub last_distributed: u64,
    pub total_bond_amount: Uint128,
    pub global_reward_index: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakerInfo {
    pub address: String,
    pub reward_index: Decimal,
    pub bond_amount: Uint128,
    pub pending_reward: Uint128,
}

pub type StakerInfoKey<'a> = String;

pub fn staker_info_key<'a>(address: &'a String) -> StakerInfoKey<'a> {
    address.clone()
}

pub struct StakerInfoIndicies<'a> {
    pub address: MultiIndex<'a, String, StakerInfo, StakerInfoKey<'a>>,
}

impl<'a> IndexList<StakerInfo> for StakerInfoIndicies<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<StakerInfo>> + '_> {
        let v: Vec<&dyn Index<StakerInfo>> = vec![&self.address];
        Box::new(v.into_iter())
    }
}

pub fn staker_info_storage<'a>(
) -> IndexedMap<'a, StakerInfoKey<'a>, StakerInfo, StakerInfoIndicies<'a>> {
    let indexes = StakerInfoIndicies {
        address: MultiIndex::new(
            |d: &StakerInfo| d.address.clone(),
            "staker_info",
            "staker_info__collection",
        ),
    };
    IndexedMap::new("staker_info", indexes)
}
