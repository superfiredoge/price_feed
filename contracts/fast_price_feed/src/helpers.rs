use crate::errors::ContractError;
use crate::state::{GOV, IS_SIGNER, IS_UPDATER, TOKEN_MANAGER};
use cosmwasm_std::{Addr, Deps, Uint256};

pub struct PositionRouterState {
    pub increase_position_request_keys_start: Uint256,
    pub decrease_position_request_keys_start: Uint256,
}

//TODO: cosmwasm query msg
pub fn load_position_router(_: &Addr) -> PositionRouterState {
    PositionRouterState {
        increase_position_request_keys_start: Uint256::zero(),
        decrease_position_request_keys_start: Uint256::zero(),
    }
}

pub fn is_gov(deps: Deps, address: &Addr) -> Result<(), ContractError> {
    if !(address.eq(&GOV.load(deps.storage).unwrap())) {
        return Err(ContractError::Forbidden {});
    }

    Ok(())
}

pub fn only_updater(deps: Deps, address: &Addr) -> Result<(), ContractError> {
    if !IS_UPDATER
        .may_load(deps.storage, address)?
        .unwrap_or_default()
    {
        return Err(ContractError::Forbidden {});
    }

    Ok(())
}

pub fn only_signer(deps: Deps, address: &Addr) -> Result<(), ContractError> {
    if !IS_SIGNER
        .may_load(deps.storage, address)?
        .unwrap_or_default()
    {
        return Err(ContractError::Forbidden {});
    }

    Ok(())
}

pub fn only_token_manager(deps: Deps, address: &Addr) -> Result<(), ContractError> {
    if !(address.eq(&TOKEN_MANAGER.load(deps.storage).unwrap())) {
        return Err(ContractError::Forbidden {});
    }

    Ok(())
}

// Assuming you have an equivalent of getLatestPrimaryPrice
pub fn get_latest_primary_price(
    _vault_price_feed: &Addr,
    _token: &Addr,
) -> Result<Uint256, ContractError> {
    // Logic to get the latest primary price from vault_price_feed...
    Ok(Uint256::zero()) // Just a placeholder for now
}
