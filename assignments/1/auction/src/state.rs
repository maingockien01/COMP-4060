
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr};

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
        pub resource_id: String,
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
        pub price: f64
}