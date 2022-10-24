
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr};
use cosmwasm_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
        pub owner: Addr,
}

#[cw_serde]
pub struct Seller {
        pub id: Addr,
        pub name: String,
}

#[cw_serde]
pub struct Resource {
        pub seller_id: Addr,
        pub resource_id: u64,
        pub volume: f64,
        pub price: f64,
        pub status: Status,
        pub bids: Option<Vec<Bid>>,
}

#[cw_serde]
pub struct Buyer {
        pub id: Addr,
        pub name: String,
}

#[cw_serde]
pub enum Status {
        Init,
        Bidding,
        Sold,
        Canceled,
}

#[cw_serde]
pub struct Bid {
        pub buyer_id: Addr,
        pub resource_id: u64,
        pub price: f64,
        pub bid_id: u64,
}

#[cw_serde]
pub struct ResourceDeposit {
        pub resource_id: u64,
        pub deposit: Coin,
        pub seller_id: Addr,
}

#[cw_serde]
pub struct BidDeposit {
        pub resource_id: u64,
        pub deposit: Vec<Coin>,
        pub buyer_id: Addr,
        pub bid_id: u64,
}

#[cw_serde]
pub enum CoinType {
        Native,
        Usd,
        Cad,
}

pub CONFIG: Item<Config> = Item::new("config");
pub SELLERS: Map<&Addr, Seller> = Map::new("sellers");
pub RESOURCES: Map<u64,Resource> = List::new("resources");
pub BID_DEPOSITS: Map<(Addr, u64), BidDeposit> = Map::new("bid_deposits"); // Todo: make the map index by 2 keys: buyer and resouce id
pub RESOURCE_DEPOSITS: Map<(Addr, u64), ResourceDeposit> = Map::new("resource_deposits"); // Todo: make the map index by 2 keys: seller and resouce id

