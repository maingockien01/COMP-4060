use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{to_binary, Addr, CosmosMsg, StdResult, WasmMsg, Coin};

use crate::error::ContractError;
use crate::msg::ExecuteMsg;

/// CwTemplateContract is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct CwTemplateContract(pub Addr);

impl CwTemplateContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
        let msg = to_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
        }
        .into())
    }
}

pub fn extract_coin(sent_funds: &[Coin], denom: &str) -> Result<Coin, ContractError> {
    if sent_funds.len() != 1 {
        return Err(ContractError::WrongCoinSent {});
    }
    if sent_funds[0].denom != *denom {
        return Err(ContractError::WrongFundCoin {
            expected: denom.to_string(),
            got: sent_funds[0].denom.clone(),
        });
    }
    Ok(sent_funds[0].clone())
}