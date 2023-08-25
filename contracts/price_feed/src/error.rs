use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("PriceFeed: cannot migrate from {previous_contract}")]
    CannotMigrate { previous_contract: String },

    #[error("PriceFeed: cannot migrate from {previous_version}")]
    CannotMigrateVersion { previous_version: String },

    #[error("PriceFeed: forbidden")]
    Forbidden {},
}
