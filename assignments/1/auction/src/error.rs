use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.

    #[error("Resources Are Not Bidding")]
    NotBidding {},

    #[error("Bid Too Low")]
    BidTooLow {},

    #[error("Resource Not Found")]
    NotInit {},

    #[error("Bid Not Found")]
    BidNotFound {},

    #[error("Not Enough Deposit")]
    InsufficientDeposit {},

    #[error("Resource Already Sold")]
    AlreadySold {},


}
