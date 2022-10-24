#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG, RESOURCES, CoinType};

use uuid::Uuid;

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
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let ownder = msg.owner
        .and_then(|address| deps.api.addr_validate(&address).ok())
        .unwrap_or(info.sender);

        let config = Config {
            owner: owner.Clone(),
        };

        CONFIG.save(deps.storage, &config)?;

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
    match msg {
        ExecuteMsg::NewResource { seller_id, resource_id, volume, price } => 
            new_resource(_deps, _info, seller_id, resource_id, volume, price),
        
        ExecuteMsg::PlaceBid { resource_id, buyer_id, price } => 
            place_bid(_deps, _info, resource_id, buyer_id, price),
        
        ExecuteMsg::CancelResource { resource_id } => 
            cancel_resource(_deps, _info, resource_id),
        
        ExecuteMsg::StartBidding { resource_id } => 
            start_bidding(_deps, _info, resource_id),
        
    }
}

pub fn new_resource (deps: DepsMut, info: MessageInfo, seller_id: Option<String>, resource_id: Option<String>, volume: f64, price: f64) -> Result<Response, ContractError> {
    let seller_id = seller_id
        .and_then(|address| deps.api.addr_validate(&address).ok())
        .unwrap_or(info.sender);

    let resource_id = RESOURCES.update::<_, cosmwasm_std::StdError>(deps.storage, | id | => Ok(id.add(1)))?;

    let resource = Resource {
        id: resource_id,
        seller_id: seller_id,
        volume: volume,
        price: price,
        status: Status::Init,
    };
        
    // Todo: Check if seller makes a security deposit

    RESOURCES.save(deps.storage, resource_id, &resource)?;

    Ok(Response::new().add_attribute("method", "new_resource")
        .add_attribute("seller_id", seller_id)
        .add_attribute("resource_id", resource_id)
        .add_attribute("volume", volume)
        .add_attribute("price", price))
}

pub fn place_bid (deps: DepsMut, info: MessageInfo, resource_id: Option<String>, buyer_id: Option<String>, price: f64) -> Result<Response, ContractError> {
    let buyer_id = buyer_id
        .and_then(|address| deps.api.addr_validate(&address).ok())
        .unwrap_or(info.sender);

    let mut resource = RESOURCES.load(deps.storage, resource_id)?;

    if resource.status != Status::Bidding {
        return Err(ContractError::NotBidding {});
    }

    let bid = Bid {
        buyer_id: buyer_id.clone(),
        price: price,
    };

    // Check if new bid is higher than current bid
    if let Some(bids) = resource.bids {
        if bids.len() > 0 {
            let current_bid = bids.last().unwrap();
            if current_bid.price >= price {
                return Err(ContractError::BidTooLow {});
            }
        }
    }

    // Todo: updatre getting deposit using 2 indexes
    let mut bid_deposit = BUYER_DEPOSITS.load(deps.storage, buyer_id)?;
    let prev_deposit = bid_deposit?.deposit;

    //Check if buyer made a security deposit for the bid which is new amount with previous deposit
    let deposit = info.funds.iter().find(|coin| coin.denom == CoinType.Native).unwrap();

    let total = resouce.price * resource.volume;
    
    if deposit.amount + prev_deposit.amount < total {
        return Err(ContractError::InsufficientDeposit {});
    }

    // Add new highest bid to resource
    resource.bids.push(bid);
    
    // Update buyer deposit
    bid_deposit.deposit.push(deposit);

    // Todo: update new deposit
    BUYER_DEPOSITS.save(deps.storage, buyer_id, , &bid_deposit)?;

    RESOURCES.save(deps.storage, resource_id.as_bytes(), &resource)?;

    Ok(Response::new().add_attribute("method", "place_bid")
        .add_attribute("buyer_id", buyer_id)
        .add_attribute("resource_id", resource_id)
        .add_attribute("price", price)
        .add_attribute("refund", refund.len() == 0 ? "No" : "Yes")
        .add_message(BankMsg::Send {
            to_address: buyer_id,
            refund,
        })
}

pub fn cancel_resource (deps: DepsMut, info: MessageInfo, resource_id: String) -> Result<Response, ContractError> {
    let mut resource = RESOURCES.load(deps.storage, resource_id)?;

    if resource.status == Status::Sold {
        return Err(ContractError::AlreadySold {});
    }

    let sender = info.sender;

    if resource.seller_id != sender {
        return Err(ContractError::Unauthorized {});
    }

    resource.status = Status::Cancelled;

    RESOURCES.save(deps.storage, resource_id.as_bytes(), &resource)?;

    // Todo: refund deposit to all bidders

    // Todo: refund deposit to seller

    Ok(Response::new().add_attribute("method", "cancel_resource")
        .add_attribute("resource_id", resource_id))
}

pub fn start_bidding (deps: DepsMut, info: MessageInfo, resource_id: Option<String>) -> Result<Response, ContractError> {
    
    let mut resource = RESOURCES.load(deps.storage, resource_id)?;
    
    if info.sender != resouce.seller_id {
        return Err(ContractError::Unauthorized {});
    }
    
    if resource.status != Status::Init {
        return Err(ContractError::NotInit {});
    }

    resource.status = Status::Bidding;

    RESOURCES.save(deps.storage, resource_id.as_bytes(), &resource)?;

    Ok(Response::new().add_attribute("method", "start_bidding")
        .add_attribute("resource_id", resource_id))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryResources {} => to_binary(&query_resources(_deps)?),
    }
}

pub fn query_resources (deps: Deps) -> StdResult<Vec<Resource>> {
    let resources = RESOURCES
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| item.map(|(_, v)| v))
        .collect::<StdResult<Vec<Resource>>>()?
        .sort_by(|a, b| a.price.cmp(&b.price));

    Ok(resources)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Addr, Coin, CosmosMsg, StdError, Uint128};

    #[test]
    fn test_new_resource() {
        let mut deps = mock_dependencies(&[]);

        let info = mock_info("addr0000", &coins(2, "token"));

        let msg = ExecuteMsg::NewResource {
            seller_id: None,
            resource_id: None,
            volume: 100.0,
            price: 1.0,
        };

        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::QueryResources {}).unwrap();
        let value: Vec<Resource> = from_binary(&res).unwrap();
        assert_eq!(1, value.len());
        assert_eq!("addr0000", value[0].seller_id.as_str());
        assert_eq!(100.0, value[0].volume);
        assert_eq!(1.0, value[0].price);
    }

    #[test]
    fn test_place_bid() {
        // let mut deps = mock_dependencies(&[]);

        // let info = mock_info("addr0000", &coins(2, "token"));

        // let msg = ExecuteMsg::NewResource {
        //     seller_id: None,
        //     resource_id: None,
        //     volume: 100.0,
        //     price: 1.0,
        // };

        // let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        // assert_eq!(0, res.messages.len());

        // // it worked, let's query the state
        // let res = query(deps.as_ref(), mock_env(), QueryMsg::Resources {}).unwrap();
        // let value: Vec<Resource> = from_binary(&res).unwrap();
        // assert_eq!(1, value.len());
        // assert_eq!("addr0000", value[0].seller_id.as_str());
        // assert_eq!(100.0, value[0].volume);
        // assert_eq!(1.0, value[0].price);

        // let info = mock_info("addr0001", &coins(2, "token"));

        // let msg = ExecuteMsg::PlaceBid {
        //     resource_id
        //     buyer_id: None,
        //     price: 1.0,
        // }
}
