#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Order, 
    to_binary, attr, from_binary, Timestamp, Uint128,
    CosmosMsg, BankMsg};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, QueryResourcesResponse};
use crate::state::{Config, CONFIG, RESOURCES, Status, BUYER_DEPOSIT_ACCOUNT, Resource, Bid, RESOURCE_ID};
use crate::helpers::{extract_coin};

use std::ops::Add;

use std::cmp::Ordering;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:auction";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(_deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = _msg.owner
        .and_then(|address| _deps.api.addr_validate(address.as_str()).ok())
        .unwrap_or(_info.sender);

        let config = Config {
            owner: owner.clone(),
            denom: _msg.denom,
        };

        CONFIG.save(_deps.storage, &config)?;

        RESOURCE_ID.save(_deps.storage, &0u64)?;

        Ok(Response::new().add_attribute("method", "instantiate")
            .add_attribute("owner", owner))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match _msg {
        ExecuteMsg::NewResource { seller_id, volume, price } => 
            new_resource(_deps, _env, _info, seller_id, volume, price),
        
        ExecuteMsg::PlaceBid { resource_id, buyer_id, price } => 
            place_bid(_deps, _env, _info, resource_id, buyer_id, price),
        
        ExecuteMsg::CancelResource { resource_id } => 
            cancel_resource(_deps, _info, resource_id),
        
        ExecuteMsg::StartBidding { resource_id } => 
            start_bidding(_deps, _info, resource_id),
        
        ExecuteMsg::FinalizeBids {} => 
            finalize_bids(_deps, _env, _info),

        ExecuteMsg::FinalizeBid { resource_id } => 
            finalize_bid(_deps, _env, _info, resource_id),
    }
}

pub fn new_resource (deps: DepsMut, env: Env, info: MessageInfo, seller_id: Option<String>, volume: u64, price: u64) -> Result<Response, ContractError> {
    let seller_id = seller_id
        .and_then(|address| deps.api.addr_validate(&address).ok())
        .unwrap_or(info.sender);

    let resource_id = RESOURCE_ID.update::<_, cosmwasm_std::StdError>(deps.storage, |id| Ok(id.add(1)))?;

    let current_time = env.block.time;

    let init_expire = current_time.clone().plus_seconds(60 * 60 * 24 * 3); //3 days

    let resource = Resource {
        resource_id: resource_id,
        seller_id: seller_id.clone(),
        volume: volume,
        price: price,
        status: Status::Init,
        highest_bid: None,
        expire: init_expire,
        bidders: vec![],
    };
        
    // Todo: Check if seller makes a security deposit

    RESOURCES.save(deps.storage, resource_id, &resource)?;

    Ok(Response::new().add_attribute("method", "new_resource")
        .add_attribute("seller_id", seller_id)
        .add_attribute("resource_id", resource_id.to_string())
        .add_attribute("volume", volume.to_string())
        .add_attribute("price", price.to_string())
        .add_attribute("expire", init_expire.seconds().to_string()))
}

pub fn place_bid (deps: DepsMut, env: Env, info: MessageInfo, resource_id: u64, buyer_id: Option<String>, price: u64) -> Result<Response, ContractError> {
    let buyer_id = buyer_id
        .and_then(|address| deps.api.addr_validate(&address).ok())
        .unwrap_or(info.sender);

    let current_time = env.block.time;

    let mut resource = RESOURCES.load(deps.storage, resource_id)?;

    if resource.status != Status::Bidding {
        return Err(ContractError::NotBidding {});
    }

    if buyer_id == resource.seller_id {
        return Err(ContractError::Unauthorized {});
    }

    if current_time.seconds() > resource.expire.seconds() {
        return Err(ContractError::ResourceExpired{});
    }

    let bid = Bid {
        buyer_id: buyer_id.clone(),
        price: price,
        resource_id: resource_id,
    };

    // // Check if new bid is higher than current bid
    if resource.highest_bid != None {
        let current_bid = resource.highest_bid.unwrap();
        if current_bid.price >= price {
            return Err(ContractError::BidTooLow {});
        }
    }

    // Add new highest bid to resource
    resource.highest_bid = Some(bid.clone());

    // Extend expire time: extend to 1 hour if left time is less than 1 hours
    if resource.expire.seconds() - current_time.seconds() < 3600 {
        resource.expire = current_time.clone().plus_seconds(3600);
    }
    
    // Todo: handle money
    // Get amount sent in the btransaction
    let config = CONFIG.load(deps.storage)?;
    let sent_deposit = extract_coin(&info.funds, &config.denom)?;

    // Get amount in the current account
    let current_deposit = BUYER_DEPOSIT_ACCOUNT.load(deps.storage, (buyer_id.clone(), resource_id));

    let mut new_deposit = sent_deposit.clone();

    if !current_deposit.is_err() {
        new_deposit.amount += current_deposit.unwrap().amount;
    }

    // Check if amount is sufficient

    if new_deposit.amount < Uint128::from(resource.price * resource.volume) {
        return Err(ContractError::InsufficientDeposit{});
    }

    // Add amount into buyer-resource account with key of (buyer_id, resource_id)

    if !resource.bidders.contains(&buyer_id) {
        resource.bidders.push(buyer_id.clone());
    }

    RESOURCES.save(deps.storage, resource_id, &resource)?;

    Ok(Response::new().add_attribute("method", "place_bid")
        .add_attribute("buyer_id", buyer_id)
        .add_attribute("resource_id", resource_id.to_string())
        .add_attribute("price", price.to_string())
    )
}

pub fn cancel_resource (deps: DepsMut, info: MessageInfo, resource_id: u64)-> Result<Response, ContractError> {
    let mut resource = RESOURCES.load(deps.storage, resource_id)?;

    if resource.status == Status::Sold {
        return Err(ContractError::AlreadySold {});
    }

    let sender = info.sender;

    if resource.seller_id != sender {
        return Err(ContractError::Unauthorized {});
    }

    resource.status = Status::Canceled;

    RESOURCES.save(deps.storage, resource_id, &resource)?;

    // Todo: refund deposit to all bidders

    // Todo: refund deposit to seller

    Ok(Response::new().add_attribute("method", "cancel_resource")
        .add_attribute("resource_id", resource_id.to_string()))
}

pub fn start_bidding (deps: DepsMut, info: MessageInfo, resource_id: u64) -> Result<Response, ContractError> {
    
    let mut resource = RESOURCES.load(deps.storage, resource_id)?;
    
    if info.sender != resource.seller_id {
        return Err(ContractError::Unauthorized {});
    }
    
    if resource.status != Status::Init {
        return Err(ContractError::NotInit {});
    }

    resource.status = Status::Bidding;

    RESOURCES.save(deps.storage, resource.resource_id, &resource)?;

    Ok(Response::new().add_attribute("method", "start_bidding")
        .add_attribute("resource_id", resource_id.to_string()))
}

pub fn finalize_bids (deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {

    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized{});
    }

    let current_time = env.block.time;

    let resources = RESOURCES.range(deps.storage, None, None, Order::Ascending).map(|item| item.map(|(_, v)| v))
    .collect::<StdResult<Vec<Resource>>>()?;

    todo!();

    Ok(Response::new().add_attribute("method", "finalize_bids"))
}

pub fn finalize_bid (deps: DepsMut, env: Env, info: MessageInfo, resource_id: u64) -> Result<Response, ContractError> {

    let config = CONFIG.load(deps.storage)?;
    
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized{});
    }

    let current_time = env.block.time;
    let mut resource = RESOURCES.load(deps.storage, resource_id)?;

    if current_time.seconds() < resource.expire.seconds() {
        return Err(ContractError::NotExpire{});
    }

    if resource.status != Status::Bidding {
        return Err(ContractError::NotBidding{});
    }
    let mut refund_msg = vec![];

    //Get bank account
    if !resource.clone().highest_bid.is_none() {
        
        let highest_bid = resource.highest_bid.clone().unwrap();

        let highest_bidder = highest_bid.buyer_id;

        //Refund all bidders except highest bidder -> final buyer
        let mut bidders = resource.clone().bidders;


        for bidder in bidders {
            if bidder != highest_bidder {
                let deposit = BUYER_DEPOSIT_ACCOUNT.load(deps.storage, (bidder.clone(), resource.resource_id));
                if deposit.is_ok() {
                    let deposit_amount = deposit.unwrap();

                    refund_msg.push(CosmosMsg::Bank(BankMsg::Send{
                        to_address: bidder.to_string(),
                        amount: vec![deposit_amount],
                    }));
                }
            }
        }
    }
    //Change status to sold

    resource.status = Status::Sold;

    RESOURCES.save(deps.storage, resource.resource_id, &resource)?;

    Ok(Response::new()
        .add_messages(refund_msg)
        .add_attribute("method", "finalize_bid")
        .add_attribute("resource", resource_id.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    match _msg {
        QueryMsg::QueryResources {} => query_resources(_deps),
    }
}

pub fn query_resources (deps: Deps) -> StdResult<Binary> {
    let resources = RESOURCES
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| item.map(|(_, v)| v))
        .collect::<StdResult<Vec<Resource>>>()?;

    let mut sorted_resources = resources.clone();
    sorted_resources.sort_by(|a, b| {
        if a.price < b.price {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });

    let resp = QueryResourcesResponse {
        resources: sorted_resources
    };

    to_binary(&resp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Addr, Coin, CosmosMsg, StdError, Uint128};

    fn proper_instantiate() {

    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        //no owner specified in the instantiation message
        let msg = InstantiateMsg { owner: None, denom: String::from("umgl") };
        let env = mock_env();
        let info = mock_info("creator", &[]);

        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let state = CONFIG.load(&deps.storage).unwrap();
        assert_eq!(
            state,
            Config {
                owner: Addr::unchecked("creator".to_string()),
                denom: String::from("umgl"),
            }
        );
        //specifying an owner address in the instantiation message
        let msg = InstantiateMsg {
            owner: Some("specified_owner".to_string()),
            denom: String::from("umgl"),
        };

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let state = CONFIG.load(&deps.storage).unwrap();
        assert_eq!(
            state,
            Config {
                owner: Addr::unchecked("specified_owner".to_string()),
                denom: String::from("umgl"),
            }
        );
    }


    #[test]
    fn test_new_resource() {
        let mut deps = mock_dependencies();
        //no owner specified in the instantiation message
        let msg = InstantiateMsg { owner: None, denom: String::from("umgl") };
        let env = mock_env();
        let info = mock_info("creator", &[]);

        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let info = mock_info("addr0000", &coins(2, "umgl"));
        let msg = ExecuteMsg::NewResource {
            seller_id: None,
            volume: 100,
            price: 1,
        };
        let mut env = mock_env();

        env.block.time = Timestamp::from_seconds(0);

        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(
            res.attributes,
            vec![
                attr("method", "new_resource"),
                attr("seller_id", "addr0000"),
                attr("resource_id", "1"),
                attr("volume", "100"),
                attr("price", "1"),
                attr("expire", "259200"),

            ]
        );
    }

    #[test]
    fn test_start_bidding() {
        let mut deps = mock_dependencies();
        //no owner specified in the instantiation message
        let msg = InstantiateMsg { owner: None, denom: String::from("umgl") };
        let env = mock_env();
        let info = mock_info("creator", &[]);

        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let info = mock_info("addr0000", &coins(2, "umgl"));
        let msg = ExecuteMsg::NewResource {
            seller_id: None,
            volume: 100,
            price: 1,
        };

        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        

        // Start bid with different address of seller
        let info = mock_info("addr0001", &coins(2, "umgl"));
        let msg = ExecuteMsg::StartBidding {
            resource_id: 1,
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
        assert_eq!(res.map_err(|e| e), Err(ContractError::Unauthorized{}));
        
        // Start bid with correct addres
        let info = mock_info("addr0000", &coins(2, "umgl"));
        let msg = ExecuteMsg::StartBidding {
            resource_id: 1,
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(
            res.attributes,
            vec![
                attr("method", "start_bidding"),
                attr("resource_id", "1")

            ]
        );
    }

    #[test]
    fn test_place_bid() {
        let mut deps = mock_dependencies();
        //no owner specified in the instantiation message
        let msg = InstantiateMsg { owner: None, denom: String::from("umgl") };
        let env = mock_env();
        let info = mock_info("creator", &[]);

        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        // Create new resource
        let info = mock_info("addr0000", &coins(2, "umgl"));
        let msg = ExecuteMsg::NewResource {
            seller_id: None,
            volume: 100,
            price: 1,
        };
        let init_time = 0;
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(init_time);

        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Place bid - Should be error since bid has not started yet
        let info = mock_info("addr0002", &coins(2, "umgl"));
        let msg = ExecuteMsg::PlaceBid {
            resource_id: 1,
            buyer_id: None,
            price: 2,
        };
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(init_time + 1);

        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
        assert!(res.is_err());
        
        // Start bid - Change status of resource to bidding
        let info = mock_info("addr0000", &coins(2, "umgl"));
        let msg = ExecuteMsg::StartBidding {
            resource_id: 1,
        };
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(init_time + 1);
        
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(
            res.attributes,
            vec![
                attr("method", "start_bidding"),
                attr("resource_id", "1")

            ]
        );

        // Place bid should be success
        let info = mock_info("addr0002", &coins(200, "umgl"));
        let msg = ExecuteMsg::PlaceBid {
            resource_id: 1,
            buyer_id: None,
            price: 2,
        };
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(init_time + 1);
        
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());
        assert_eq!(
            res.attributes,
            vec![
                attr("method", "place_bid"),
                attr("buyer_id", "addr0002"),
                attr("resource_id", "1"),
                attr("price", "2")
            ]
        );

        // Place bid after expire - Should be fail
        let info = mock_info("addr0002", &coins(200, "umgl"));
        let msg = ExecuteMsg::PlaceBid {
            resource_id: 1,
            buyer_id: None,
            price: 2,
        };
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(init_time + 259200 + 1);
        
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
        assert_eq!(res.map_err(|e| e), Err(ContractError::ResourceExpired {}));


        // Place bid - invalid resource id - should be fail
        let info = mock_info("addr0002", &coins(200, "umgl"));
        let msg = ExecuteMsg::PlaceBid {
            resource_id: 2,
            buyer_id: None,
            price: 2,
        };
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(init_time + 1);
        
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
        assert_eq!(res.is_err(), true);

        // Place bid low price - should be fail

        let info = mock_info("addr0002", &coins(200, "umgl"));
        let msg = ExecuteMsg::PlaceBid {
            resource_id: 1,
            buyer_id: None,
            price: 1,
        };
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(init_time + 1);
        
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);

        assert_eq!(res.map_err(|e| e), Err(ContractError::BidTooLow {}));

        // Place bid insufficient amount should fail
        let info = mock_info("addr0003", &coins(1, "umgl"));
        let msg = ExecuteMsg::PlaceBid {
            resource_id: 1,
            buyer_id: None,
            price: 3,
        };
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(init_time + 1);
        
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
        assert_eq!(res.map_err(|e| e), Err(ContractError::InsufficientDeposit {}));

    }


    #[test]
    fn test_place_bid_add_up_deposit() {
        let mut deps = mock_dependencies();
        //no owner specified in the instantiation message
        let msg = InstantiateMsg { owner: None, denom: String::from("umgl") };
        let env = mock_env();
        let info = mock_info("creator", &[]);

        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        // Create new resource
        let info = mock_info("addr0000", &coins(2, "umgl"));
        let msg = ExecuteMsg::NewResource {
            seller_id: None,
            volume: 100,
            price: 1,
        };
        let init_time = 0;
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(init_time);

        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Start bid - Change status of resource to bidding
        let info = mock_info("addr0000", &coins(2, "umgl"));
        let msg = ExecuteMsg::StartBidding {
            resource_id: 1,
        };
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(init_time + 1);
        
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Place bid should be success
        let info = mock_info("addr0002", &coins(200, "umgl"));
        let msg = ExecuteMsg::PlaceBid {
            resource_id: 1,
            buyer_id: None,
            price: 2,
        };
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(init_time + 1);
        
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());
        assert_eq!(
            res.attributes,
            vec![
                attr("method", "place_bid"),
                attr("buyer_id", "addr0002"),
                attr("resource_id", "1"),
                attr("price", "2")
            ]
        );

        // Place bid should be success
        let info = mock_info("addr0002", &coins(200, "umgl"));
        let msg = ExecuteMsg::PlaceBid {
            resource_id: 1,
            buyer_id: None,
            price: 4,
        };
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(init_time + 1);
        
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());
        assert_eq!(
            res.attributes,
            vec![
                attr("method", "place_bid"),
                attr("buyer_id", "addr0002"),
                attr("resource_id", "1"),
                attr("price", "4")
            ]
        );
    }

    #[test]
    fn test_query_resources() {
        let mut deps = mock_dependencies();
        //no owner specified in the instantiation message
        let msg = InstantiateMsg { owner: None, denom: String::from("umgl") };
        let env = mock_env();
        let info = mock_info("creator", &[]);

        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let info = mock_info("addr0000", &coins(2, "token"));
        let msg = ExecuteMsg::NewResource {
            seller_id: None,
            volume: 100,
            price: 1,
        };

        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        
        let info = mock_info("addr0001", &coins(2, "token"));
        let msg = ExecuteMsg::NewResource {
            seller_id: None,
            volume: 100,
            price: 3,
        };

        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let info = mock_info("addr0002", &coins(2, "token"));
        let msg = ExecuteMsg::NewResource {
            seller_id: None,
            volume: 100,
            price: 2,
        };

        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), env.clone(), QueryMsg::QueryResources{}).unwrap();

        let value: QueryResourcesResponse = from_binary(&res).unwrap();
        // Test length
        assert_eq!(value.resources.len(), 3);
        // Test orders
        assert_eq!(value.resources[0].price, 1);
        assert_eq!(value.resources[1].price, 2);
        assert_eq!(value.resources[2].price, 3);

    }

    #[test]
    fn test_place_bid_extend_expire() {
        todo!();
    }
}
