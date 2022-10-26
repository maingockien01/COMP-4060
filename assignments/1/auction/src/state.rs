
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Timestamp};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
        pub owner: Addr,
        pub denom: String,
}

#[cw_serde]
pub struct Resource {
        pub seller_id: Addr,
        pub resource_id: u64,
        pub volume: u64,
        pub price: u64,
        pub status: Status,
        pub highest_bid: Option<Bid>,
        pub expire: Timestamp,
        pub bidders: Vec<Addr>,
}

#[cw_serde]
pub enum Status {
        Init,
        Bidding,
        Sold,
        Canceled,
        Approved,
        Transfered,
}

#[cw_serde]
pub struct Bid {
        pub buyer_id: Addr,
        pub resource_id: u64,
        pub price: u64,
}

#[cw_serde]
pub struct ResourceDeposit {
        pub resource_id: u64,
        pub deposit: Coin,
        pub seller_id: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const RESOURCES: Map<u64,Resource> = Map::new("resources");
pub const RESOURCE_ID: Item<u64> = Item::new("resource_id");

pub const BUYER_DEPOSIT_ACCOUNT: Map<(Addr, u64), Coin> = Map::new("buyer_deposits"); // Todo: make the map index by 2 keys: buyer and resouce id
// pub const SELLER_DEPOSITS: Map<(Addr, u64), ResourceDeposit> = Map::new("resource_deposits"); // Todo: make the map index by 2 keys: seller and resouce id

