use crate::query::GetRoundDataResult;
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Uint128,
};
use cw2::{get_contract_version, set_contract_version};

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    state::{
        PriceFeedState, LATEST_ROUND, PRICE_FEED_ADMINS, PRICE_FEED_ANSWERS, PRICE_FEED_GOV,
        PRICE_FEED_STATE,
    },
};

use semver::Version;

// version info
const CONTRACT_NAME: &str = "crates.io:{{project-name}}";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    PRICE_FEED_GOV.save(deps.storage, &info.sender)?;
    PRICE_FEED_ADMINS.save(deps.storage, info.sender.clone(), &true)?;
    LATEST_ROUND.save(deps.storage, &PriceFeedState::default())?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let version: Version = CONTRACT_VERSION.parse().map_err(from_semver)?;

    // Current contract version
    let stored_info = get_contract_version(deps.storage)?;

    // Stored contract version
    let stored_version: Version = stored_info.version.parse().map_err(from_semver)?;

    // version less than stored
    if stored_version > version {
        return Err(ContractError::CannotMigrateVersion {
            previous_version: stored_info.version,
        });
    }

    // check contract type
    if CONTRACT_NAME != stored_info.contract {
        return Err(ContractError::CannotMigrate {
            previous_contract: stored_info.contract,
        });
    }

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetAdmin { admin, status } => {
            let gov = PRICE_FEED_GOV.load(deps.storage)?;
            if gov != info.sender {
                return Err(ContractError::Forbidden {});
            }

            PRICE_FEED_ADMINS.save(deps.storage, admin.clone(), &status)?;
            Ok(Response::new()
                .add_attribute("method", "set_admin")
                .add_attribute("admin", admin)
                .add_attribute("status", status.to_string()))
        }
        ExecuteMsg::SetLatestAnswer(answer) => {
            check_admin(deps.as_ref(), info.sender)?;

            let mut new_round = Uint128::zero();
            LATEST_ROUND.update(
                deps.storage,
                |mut state: PriceFeedState| -> Result<PriceFeedState, ContractError> {
                    state.round_id = state.round_id.checked_add(Uint128::one()).unwrap();
                    state.answer = answer;
                    new_round = state.round_id;
                    Ok(state)
                },
            )?;

            PRICE_FEED_ANSWERS.save(deps.storage, new_round.u128(), &answer)?;
            Ok(Response::new()
                .add_attribute("method", "set_latest_answer")
                .add_attribute("answer", answer.to_string()))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetLatestRound => to_binary(&PRICE_FEED_STATE.load(deps.storage)?.round_id),
        QueryMsg::GetLatestAnswer => to_binary(&PRICE_FEED_STATE.load(deps.storage)?.answer),
        QueryMsg::GetRoundData { round_id } => {
            let answer = PRICE_FEED_ANSWERS
                .load(deps.storage, round_id.u128())
                .unwrap_or_default();
            to_binary(&GetRoundDataResult {
                round_id,
                answer,
                started_at: Uint128::zero(),
                updated_at: Uint128::zero(),
                answered_in_round: Uint128::zero(),
            })
        }
    }
}

pub fn check_admin(deps: Deps, sender: Addr) -> Result<(), ContractError> {
    let is_admin = PRICE_FEED_ADMINS
        .may_load(deps.storage, sender)?
        .unwrap_or_default();

    if is_admin {
        Ok(())
    } else {
        Err(ContractError::Forbidden {})
    }
}

fn from_semver(err: semver::Error) -> StdError {
    StdError::generic_err(format!("Semver: {}", err))
}
