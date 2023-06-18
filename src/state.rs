use cosmwasm_std::{Addr, Coin, Decimal};
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BaseInfo {
    pub owner: Addr,
    pub commodity: String,
    pub bidding_denom: String,
    pub commission: Decimal,
    pub active: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Bid {
    pub address: Addr,
    pub bid: Coin,
}

pub const BASE_INFO: Item<BaseInfo> = Item::new("base_info");
pub const BIDS: Map<Addr, Coin> = Map::new("bids");
pub const HIGHEST_BID: Item<Bid> = Item::new("highest_bid");
