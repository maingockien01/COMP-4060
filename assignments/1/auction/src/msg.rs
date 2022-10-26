use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::state::{Resource, Bid};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    NewResource {
        seller_id: Option<String>,
        volume: u64,
        price: u64,
    },

    PlaceBid {
        resource_id: u64,
        buyer_id: Option<String>,
        price: u64,
    },
    
    CancelResource {
        resource_id: u64,
    },

    StartBidding {
        resource_id: u64,
    },

    FinalizeBids {
    },

    FinalizeBid {
        resource_id: u64,
    },

    // WithdrawExpiredResources {
    //     resouce_id: String,
    // },

    // ApproveResource {
    //     resouce_id: String,
    // },

    // TransferDeposit {
    //     resouce_id: String
    // }

}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {

    #[returns(QueryResourcesResponse)]
    QueryResources {},

    // #[returns(QueryResourceResponse)]
    // QueryResource {
    //     resource_id: String,
    // },

    // #[returns(QueryBidsResponse)]
    // QueryBids {
    //     resource_id: String,
    // },
}

#[cw_serde]
pub struct QueryResourcesResponse {
    pub resources: Vec<Resource>,
}

// #[cw_serde]
// pub struct QueryResourceResponse {
//     pub resource: Resource,
//     pub highest_bid: Option<Bid>,
// }

// #[cw_serde]
// pub struct QueryBidsResponse {
//     pub bids: Vec<Bid>,
// }


