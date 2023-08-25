use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("FastPriceFeed: cannot migrate from {previous_contract}")]
    CannotMigrate { previous_contract: String },

    #[error("FastPriceFeed: cannot migrate from {previous_version}")]
    CannotMigrateVersion { previous_version: String },

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

    #[error("FastPriceFeed: timestamp below allowed range")]
    TimestampBelowAllowedRange {},

    #[error("FastPriceFeed: timestamp exceeds allowed range")]
    TimestamExceedsAllowedRange {},

    #[error("FastPriceFeed: minBlockInterval not yet passeds")]
    MinblockInterval {},
}
