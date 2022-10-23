use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub ownder: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    NewResource {
        seller_id: Option<String>,
        resource_id: Option<String>,
        volume: f64,
        price: f64,
    },

    PlaceBid {
        resource_id: String,
        buyer_id: Option<String>,
        price: f64,
    },
    
    CancelResource {
        resource_id: String,
    },

    StartBidding {
        resource_id: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {

    #[returns(QueryMeritResponse)]
    QueryMerit {},

    #[returns(QueryResourceResponse)]
    QueryResource {
        resource_id: String,
    },

    #[returns(QueryBidsResponse)]
    QueryBids {
        resource_id: String,
    },
}

#[cw_serde]
pub struct QueryMeritResponse {
    pub resources: Vec<Resource>,
}

#[cw_serde]
pub struct QueryResourceResponse {
    pub resource: Resource,
    pub highest_bid: Option<Bid>,
}

#[cw_serde]
pub struct QueryBidsResponse {
    pub bids: Vec<Bid>,
}


