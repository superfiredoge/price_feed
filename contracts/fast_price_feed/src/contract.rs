use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use crate::{
    errors::ContractError,
    execute::*,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    query::*,
};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "crates.io:{{project-name}}";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
use crate::execute::init;
use crate::msg::ExecuteMsg::*;
use crate::state::*;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _: Env,
    _: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(
        deps.storage,
        format!("crates.io:{CONTRACT_NAME}"),
        CONTRACT_VERSION,
    )?;

    init(deps, msg.config)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let sender = info.sender;
    match msg {
        Initialize {
            min_auth,
            signers,
            updaters,
        } => initialize(deps, sender, min_auth, signers, updaters),
        SetSigner { account, is_active } => set_signer(deps, sender, account, is_active),
        SetUpdater { account, is_active } => set_updater(deps, sender, account, is_active),
        SetFastPriceEvents { fast_price_events } => {
            set_fast_price_events(deps, sender, fast_price_events)
        }
        SetVaultPriceFeed { vault_price_feed } => {
            set_vault_price_feed(deps, sender, vault_price_feed)
        }
        SetMaxTimeDeviation { max_time_deviation } => {
            set_max_time_deviation(deps, sender, max_time_deviation)
        }
        SetPriceDuration { price_duration } => set_price_duration(deps, sender, price_duration),
        SetMaxPriceUpdateDelay {
            max_price_update_delay,
        } => set_max_price_update_delay(deps, sender, max_price_update_delay),
        SetSpreadBasisPointsIfInactive {
            spread_basis_points_if_inactive,
        } => set_spread_basis_points_if_inactive(deps, sender, spread_basis_points_if_inactive),
        SetSpreadBasisPointsIfChainError {
            spread_basis_points_if_chain_error,
        } => {
            set_spread_basis_points_if_chain_error(deps, sender, spread_basis_points_if_chain_error)
        }
        SetMinBlockInterval { min_block_interval } => {
            set_min_block_interval(deps, sender, min_block_interval)
        }
        SetIsSpreadEnabled { spread_enabled } => {
            set_is_spread_enabled(deps, sender, spread_enabled)
        }
        SetLastUpdatedAt { last_updated_at } => set_last_updated_at(deps, sender, last_updated_at),
        SetTokenManager { token_manager } => set_token_manager(deps, sender, token_manager),
        SetMaxDeviationBasisPoints {
            max_deviation_basis_points,
        } => set_max_deviation_basis_points(deps, sender, max_deviation_basis_points),
        SetMaxCumulativeDeltaDiffs {
            tokens,
            max_cumulative_delta_diffs,
        } => set_max_cumulative_delta_diffs(deps, sender, tokens, max_cumulative_delta_diffs),
        SetPriceDataInterval {
            price_data_interval,
        } => set_price_data_interval(deps, sender, price_data_interval),
        SetMinAuthorizations { min_authorizations } => {
            set_min_authorizations(deps, sender, min_authorizations)
        }
        SetTokens {
            tokens,
            token_precision,
        } => set_tokens(deps, env, sender, tokens, token_precision),
        SetPrices {
            tokens,
            prices,
            timestamp,
        } => set_prices(deps, sender, env, tokens, prices, timestamp),
        SetCompactedPrices {
            price_bit_array,
            timestamp,
        } => set_compacted_prices(deps, env, price_bit_array, timestamp),
        SetPricesWithBits {
            price_bits,
            timestamp,
        } => set_prices_with_bits(deps, env, price_bits, timestamp),
        SetPricesWithBitsAndExecute {
            sender,
            position_router_addr,
            price_bits,
            timestamp,
            end_index_for_increase_positions,
            end_index_for_decrease_positions,
            max_increase_positions,
            max_decrease_positions,
        } => set_prices_with_bits_and_execute(
            deps,
            env,
            sender,
            position_router_addr,
            price_bits,
            timestamp,
            end_index_for_increase_positions,
            end_index_for_decrease_positions,
            max_increase_positions,
            max_decrease_positions,
        ),
        DisableFastPrice => disable_fast_price(deps, env, sender),
        EnableFastPrice => enable_fast_price(deps, env, sender),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPrice {
            token,
            block_timestamp,
            ref_price,
            maximise,
        } => {
            let response = get_price(deps, token, block_timestamp, ref_price, maximise)?;
            to_binary(&response)
        }
        QueryMsg::FavorFastPrice { token } => to_binary(&favor_fast_price(deps, token)?),
        QueryMsg::GetPriceData { token } => to_binary(&get_price_data(deps, token)?),
        QueryMsg::GetConfig => to_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::IsUpdater { address } => to_binary(&IS_UPDATER.load(deps.storage, &address)?),
        QueryMsg::Prices { address } => to_binary(&PRICES.load(deps.storage, &address)?),
        QueryMsg::PriceData { address } => to_binary(&PRICE_DATA.load(deps.storage, &address)?),
        QueryMsg::MaxCumulativeDeltaDiffs { address } => {
            to_binary(&MAX_CUMULATIVE_DELTA_DIFFS.load(deps.storage, &address)?)
        }
        QueryMsg::IsSigner { address } => to_binary(&IS_SIGNER.load(deps.storage, &address)?),
        QueryMsg::DisableFastPriceVotes { address } => {
            to_binary(&DISABLE_FAST_PRICE_VOTES.load(deps.storage, &address)?)
        }
        QueryMsg::MinAuthorizations => to_binary(&MIN_AUTH.load(deps.storage)?),
        QueryMsg::MaxTimeDeviation => to_binary(&MAX_TIME_DEVIATION.load(deps.storage)?),
        QueryMsg::SpreadBasisPoint => to_binary(&SPREAD_BASIS_POINT_STATE.load(deps.storage)?),
        QueryMsg::TokenData => to_binary(&TOKEN_DATA.load(deps.storage)?),
        QueryMsg::DisableFastPriceVoteCount => to_binary(&DISABLE_FAST_PRICE_VOTE_COUNT.load(deps.storage)?),
    }
}
