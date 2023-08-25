use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("FastPriceFeed: invalid lengths")]
    InvalidLength {},

    #[error("PriceFeed: forbidden")]
    Forbidden {},

    #[error("FastPriceFeed: invalid priceDuration")]
    InvalidPriceDuration {},

    #[error("FastPriceFeed: already initialized")]
    AlreadyInitialized {},

    #[error("FastPriceFeed: already voted")]
    AlreadyVoted {},

    #[error("FastPriceFeed: already enabled")]
    AlreadyEnabled {},
}
