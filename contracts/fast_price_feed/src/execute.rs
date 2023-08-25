use crate::errors::ContractError;
use crate::state::*;
use cosmwasm_std::{
    to_binary, Addr, BlockInfo, CosmosMsg, DepsMut, Env, Response, StdError, StdResult, Storage,
    Uint256, Uint64, WasmMsg,
};

use crate::helpers::*;

const CUMULATIVE_DELTA_PRECISION: Uint256 = Uint256::from_u128(10 * 1000 * 1000u128);
pub const PRICE_PRECISION: Uint256 = Uint256::from_u128(10u128.pow(30));

// 30 mins * 60 seconds (max price duration in seconds)
const MAX_PRICE_DURATION: Uint64 = Uint64::new(30 * 60 * 1_000_000_000u64);

pub fn init(deps: DepsMut, config: Config) -> Result<Response, ContractError> {
    if config.price_duration <= Uint64::zero() {
        return Err(ContractError::InvalidPriceDuration {});
    }

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("method", "init"))
}

pub fn initialize(
    deps: DepsMut,
    sender: Addr,
    min_auth: Uint256,
    signers: Vec<Addr>,
    updaters: Vec<Addr>,
) -> Result<Response, ContractError> {
    is_gov(deps.as_ref(), &sender)?;

    if IS_INITIALIZED.may_load(deps.storage)?.unwrap_or_default() {
        return Err(ContractError::AlreadyInitialized {});
    }

    for signer in signers.iter() {
        IS_SIGNER.save(deps.storage, signer, &true)?;
    }

    for updater in updaters.iter() {
        IS_UPDATER.save(deps.storage, updater, &true)?;
    }

    MIN_AUTH.save(deps.storage, &min_auth)?;

    IS_INITIALIZED.save(deps.storage, &true)?;

    Ok(Response::new().add_attribute("method", "intialize"))
}

pub fn set_signer(
    deps: DepsMut,
    sender: Addr,
    account: Addr,
    is_active: bool,
) -> Result<Response, ContractError> {
    is_gov(deps.as_ref(), &sender)?;

    IS_SIGNER.save(deps.storage, &account, &is_active)?;

    Ok(Response::new()
        .add_attribute("method", "set_signer")
        .add_attribute("signer", account)
        .add_attribute("is_active", is_active.to_string()))
}

pub fn set_updater(
    deps: DepsMut,
    sender: Addr,
    account: Addr,
    is_active: bool,
) -> Result<Response, ContractError> {
    is_gov(deps.as_ref(), &sender)?;

    IS_UPDATER.save(deps.storage, &account, &is_active)?;

    Ok(Response::new()
        .add_attribute("method", "set_updater")
        .add_attribute("account", account)
        .add_attribute("is_active", is_active.to_string()))
}

pub fn set_fast_price_events(
    deps: DepsMut,
    sender: Addr,
    fast_price_events: Addr,
) -> Result<Response, ContractError> {
    is_gov(deps.as_ref(), &sender)?;
    CONFIG.update(
        deps.storage,
        |mut config| -> Result<Config, ContractError> {
            config.fast_price_events = fast_price_events.clone();

            Ok(config)
        },
    )?;

    Ok(Response::new()
        .add_attribute("method", "set_fast_price_event")
        .add_attribute("fast_price_events", fast_price_events))
}

pub fn set_vault_price_feed(
    deps: DepsMut,
    sender: Addr,
    vault_price_feed: Addr,
) -> Result<Response, ContractError> {
    is_gov(deps.as_ref(), &sender)?;
    VAULT_ADDRESS.save(deps.storage, &vault_price_feed)?;

    Ok(Response::new()
        .add_attribute("method", "set_vault_price_feed")
        .add_attribute("vault_price_feed", vault_price_feed))
}

pub fn set_max_time_deviation(
    deps: DepsMut,
    sender: Addr,
    max_time_deviation: Uint64,
) -> Result<Response, ContractError> {
    is_gov(deps.as_ref(), &sender)?;
    MAX_TIME_DEVIATION.save(deps.storage, &max_time_deviation.u64())?;

    Ok(Response::new()
        .add_attribute("method", "set_max_time_deviation")
        .add_attribute("max_time_deviation", max_time_deviation))
}

pub fn set_price_duration(
    deps: DepsMut,
    sender: Addr,
    price_duration: Uint64,
) -> Result<Response, ContractError> {
    is_gov(deps.as_ref(), &sender)?;
    if price_duration.ge(&MAX_PRICE_DURATION) {
        return Err(ContractError::InvalidPriceDuration {});
    }

    CONFIG.update(
        deps.storage,
        |mut config| -> Result<Config, ContractError> {
            config.price_duration = price_duration;
            Ok(config)
        },
    )?;

    Ok(Response::new()
        .add_attribute("method", "set_price_duration")
        .add_attribute("price_duration", price_duration))
}

pub fn set_max_price_update_delay(
    deps: DepsMut,
    sender: Addr,
    max_price_update_delay: Uint64,
) -> Result<Response, ContractError> {
    is_gov(deps.as_ref(), &sender)?;
    CONFIG.update(
        deps.storage,
        |mut config| -> Result<Config, ContractError> {
            config.max_price_update_delay = max_price_update_delay;
            Ok(config)
        },
    )?;

    Ok(Response::new()
        .add_attribute("method", "set_max_price_update_delay")
        .add_attribute("max_price_update_delay", max_price_update_delay))
}

pub fn set_spread_basis_points_if_inactive(
    deps: DepsMut,
    sender: Addr,
    spread_basis_points_if_inactive: Uint256,
) -> Result<Response, ContractError> {
    is_gov(deps.as_ref(), &sender)?;
    SPREAD_BASIS_POINT_STATE.update(
        deps.storage,
        |mut basis_point: SpreadBasisPoint| -> Result<SpreadBasisPoint, ContractError> {
            basis_point.spread_basis_points_if_inactive = spread_basis_points_if_inactive;
            Ok(basis_point)
        },
    )?;

    Ok(Response::new()
        .add_attribute("method", "set_spread_basis_points_if_inactive")
        .add_attribute(
            "spread_basis_points_if_inactive",
            spread_basis_points_if_inactive,
        ))
}

pub fn set_spread_basis_points_if_chain_error(
    deps: DepsMut,
    sender: Addr,
    spread_basis_points_if_chain_error: Uint256,
) -> Result<Response, ContractError> {
    is_gov(deps.as_ref(), &sender)?;
    SPREAD_BASIS_POINT_STATE.update(
        deps.storage,
        |mut basis_point: SpreadBasisPoint| -> Result<SpreadBasisPoint, ContractError> {
            basis_point.spread_basis_points_if_chain_error = spread_basis_points_if_chain_error;
            Ok(basis_point)
        },
    )?;

    Ok(Response::new()
        .add_attribute("method", "set_spread_basis_points_if_chain_error")
        .add_attribute(
            "spread_basis_points_if_chain_error",
            spread_basis_points_if_chain_error,
        ))
}

pub fn set_min_block_interval(
    deps: DepsMut,
    sender: Addr,
    min_block_interval: Uint64,
) -> Result<Response, ContractError> {
    is_gov(deps.as_ref(), &sender)?;
    CONFIG.update(
        deps.storage,
        |mut config| -> Result<Config, ContractError> {
            config.min_block_interval = min_block_interval;
            Ok(config)
        },
    )?;

    Ok(Response::new()
        .add_attribute("method", "set_min_block_interval")
        .add_attribute("set_min_block_interval", min_block_interval))
}

pub fn set_is_spread_enabled(
    deps: DepsMut,
    sender: Addr,
    spread_enabled: bool,
) -> Result<Response, ContractError> {
    is_gov(deps.as_ref(), &sender)?;
    SPREAD_ENABLED.save(deps.storage, &spread_enabled)?;

    Ok(Response::new()
        .add_attribute("method", "set_is_spread_enabled")
        .add_attribute("spread_enabled", spread_enabled.to_string()))
}

pub fn set_last_updated_at(
    deps: DepsMut,
    sender: Addr,
    last_updated_at: Uint64,
) -> Result<Response, ContractError> {
    is_gov(deps.as_ref(), &sender)?;
    LAST_UPDATED.update(
        deps.storage,
        |mut last_updated| -> Result<LastUpdated, ContractError> {
            last_updated.last_updated_at = last_updated_at.u64();
            Ok(last_updated)
        },
    )?;

    Ok(Response::new()
        .add_attribute("method", "set_last_updated_at")
        .add_attribute("last_updated_at", last_updated_at))
}

pub fn set_token_manager(
    deps: DepsMut,
    sender: Addr,
    token_manager: Addr,
) -> Result<Response, ContractError> {
    only_token_manager(deps.as_ref(), &sender)?;

    TOKEN_MANAGER.save(deps.storage, &token_manager)?;

    Ok(Response::new()
        .add_attribute("method", "set_token_manager")
        .add_attribute("token_manager", token_manager.to_string()))
}

pub fn set_max_deviation_basis_points(
    deps: DepsMut,
    sender: Addr,
    max_deviation_basis_points: Uint256,
) -> Result<Response, ContractError> {
    only_token_manager(deps.as_ref(), &sender)?;

    CONFIG.update(
        deps.storage,
        |mut config| -> Result<Config, ContractError> {
            config.max_deviation_basis_points = max_deviation_basis_points;
            Ok(config)
        },
    )?;

    Ok(Response::new()
        .add_attribute("method", "set_max_deviation_basis_points")
        .add_attribute(
            "max_deviation_basis_points",
            max_deviation_basis_points.to_string(),
        ))
}

pub fn set_max_cumulative_delta_diffs(
    deps: DepsMut,
    sender: Addr,
    tokens: Vec<Addr>,
    max_cumulative_delta_diffs: Vec<Uint256>,
) -> Result<Response, ContractError> {
    only_token_manager(deps.as_ref(), &sender)?;

    // Iterate and update the values
    for (i, token) in tokens.iter().enumerate() {
        let diff = max_cumulative_delta_diffs[i];
        MAX_CUMULATIVE_DELTA_DIFFS.save(deps.storage, token, &diff)?;
    }

    Ok(Response::new()
        .add_attribute("method", "set_max_cumulative_delta_diffs")
        .add_attribute("num_tokens_updated", tokens.len().to_string()))
}

pub fn set_price_data_interval(
    deps: DepsMut,
    sender: Addr,
    price_data_interval: Uint64,
) -> Result<Response, ContractError> {
    only_token_manager(deps.as_ref(), &sender)?;

    PRICE_DATA_INTERVAL.save(deps.storage, &price_data_interval)?;

    Ok(Response::new()
        .add_attribute("method", "set_price_data_interval")
        .add_attribute("price_data_interval", price_data_interval.to_string()))
}

pub fn set_min_authorizations(
    deps: DepsMut,
    sender: Addr,
    min_authorizations: Uint256,
) -> Result<Response, ContractError> {
    only_token_manager(deps.as_ref(), &sender)?;

    MIN_AUTH.save(deps.storage, &min_authorizations)?;

    Ok(Response::new()
        .add_attribute("method", "set_min_authorizations")
        .add_attribute("min_authorizations", min_authorizations.to_string()))
}

pub fn set_tokens(
    deps: DepsMut,
    _env: Env,
    sender: Addr,
    tokens: Vec<Addr>,
    token_precision: Vec<Uint256>,
) -> Result<Response, ContractError> {
    only_updater(deps.as_ref(), &sender)?;
    if tokens.len() != token_precision.len() {
        return Err(ContractError::InvalidLength {});
    }
    let mut token_data: Vec<TokenData> = Vec::with_capacity(tokens.len());
    for (i, token) in tokens.iter().enumerate() {
        token_data[i] = TokenData::new(token.clone(), token_precision[i]);
    }

    TOKEN_DATA.save(deps.storage, &token_data)?;
    Ok(Response::new().add_attribute("method", "set_tokens"))
}

pub fn set_prices(
    deps: DepsMut,
    sender: Addr,
    env: Env,
    tokens: Vec<Addr>,
    prices: Vec<Uint256>,
    timestamp: Uint64,
) -> Result<Response, ContractError> {
    only_updater(deps.as_ref(), &sender)?;

    let should_update = set_last_updated_values(deps.storage, &env.block, timestamp.u64())?;
    let vault_address = VAULT_ADDRESS.load(deps.storage)?;
    let fast_price_event = CONFIG.load(deps.storage)?.fast_price_events;
    if should_update {
        for (i, token) in tokens.iter().enumerate() {
            set_price(
                deps.storage,
                Uint64::from(env.block.time.seconds()),
                token,
                prices[i],
                &vault_address,
                &fast_price_event,
            )?;
        }
    }

    Ok(Response::new().add_attribute("method", "set_prices"))
}

pub fn set_compacted_prices(
    deps: DepsMut,
    env: Env,
    price_bit_array: Vec<Uint256>,
    timestamp: Uint64,
) -> Result<Response, ContractError> {
    let should_update = set_last_updated_values(deps.storage, &env.block, timestamp.u64())?; // Assuming this function is ported                                        //TODO: match and fix both compacted rpice and prices with bits functions
    let vault_address = VAULT_ADDRESS.load(deps.storage)?;
    let fast_price_event = CONFIG.load(deps.storage)?.fast_price_events;
    if should_update {
        let tokens = TOKEN_DATA.load(deps.storage)?;
        for (i, &price_bits) in price_bit_array.iter().enumerate() {
            let prices = price_bits.to_le_bytes();
            for j in 0..4 {
                let index = i * 4 + j;
                if index >= TOKEN_DATA.load(deps.storage)?.len() {
                    return Ok(Response::new());
                }

                let token = &tokens[index];
                let price = Uint256::from(u64::from_le_bytes(
                    prices[j * 8..j * 8 + 8].try_into().unwrap(),
                ));
                let adjusted_price = price.multiply_ratio(PRICE_PRECISION, token.token_precision);
                set_price(
                    deps.storage,
                    Uint64::from(env.block.time.seconds()),
                    &token.token,
                    adjusted_price,
                    &vault_address,
                    &fast_price_event,
                )?;
            }
        }
    }

    Ok(Response::new().add_attribute("method", "set_compacted_prices"))
}

pub fn set_prices_with_bits(
    deps: DepsMut,
    env: Env,
    price_bits: Uint256,
    timestamp: Uint64,
) -> Result<Response, ContractError> {
    _set_prices_with_bits(deps, env, price_bits, timestamp)?;
    Ok(Response::new().add_attribute("method", "set_prices_with_bits"))
}

#[allow(clippy::too_many_arguments)]
pub fn set_prices_with_bits_and_execute(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    position_router_addr: Addr,
    price_bits: Uint256,
    timestamp: Uint64,
    end_index_for_increase_positions: Uint256,
    end_index_for_decrease_positions: Uint256,
    max_increase_positions: Uint256,
    max_decrease_positions: Uint256,
) -> Result<Response, ContractError> {
    only_updater(deps.as_ref(), &sender)?;
    _set_prices_with_bits(deps, env, price_bits, timestamp)?;

    let position_router = load_position_router(&position_router_addr);
    let max_end_index_for_increase =
        position_router.increase_position_request_keys_start + max_increase_positions;
    let max_end_index_for_decrease =
        position_router.decrease_position_request_keys_start + max_decrease_positions;

    let _adjusted_end_index_for_increase_positions =
        if end_index_for_increase_positions > max_end_index_for_increase {
            max_end_index_for_increase
        } else {
            end_index_for_increase_positions
        };

    let _adjusted_end_index_for_decrease_positions =
        if end_index_for_decrease_positions > max_end_index_for_decrease {
            max_end_index_for_decrease
        } else {
            end_index_for_decrease_positions
        };

    //TODO: change msg for interface
    let execute_increase_positions_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: position_router_addr.to_string().clone(),
        msg: to_binary("sample msg for the interface")?,
        funds: vec![], // No funds sent
    });

    let execute_decrease_positions_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: position_router_addr.to_string(),
        msg: to_binary("smaple msg required")?,
        funds: vec![], // No funds sent
    });

    Ok(Response::new()
        .add_attribute("method", "set_prices_with_bits_and_execute")
        .add_message(execute_increase_positions_msg)
        .add_message(execute_decrease_positions_msg))
}

pub fn disable_fast_price(
    deps: DepsMut,
    _env: Env,
    sender: Addr,
) -> Result<Response, ContractError> {
    only_signer(deps.as_ref(), &sender)?;

    // Check if the signer has already voted
    if DISABLE_FAST_PRICE_VOTES
        .may_load(deps.storage, &sender)?
        .unwrap_or(false)
    {
        return Err(ContractError::AlreadyVoted {});
    }

    DISABLE_FAST_PRICE_VOTES.save(deps.storage, &sender, &true)?;
    let current_vote_count = DISABLE_FAST_PRICE_VOTE_COUNT
        .load(deps.storage)
        .unwrap_or_default();

    DISABLE_FAST_PRICE_VOTE_COUNT.save(
        deps.storage,
        &(current_vote_count.checked_add(Uint256::one()).unwrap()),
    )?;

    Ok(Response::new()
        .add_attribute("method", "disable_fast_price")
        .add_attribute("sender", sender))
}

pub fn enable_fast_price(
    deps: DepsMut,
    _env: Env,
    sender: Addr,
) -> Result<Response, ContractError> {
    // Ensure the caller is a signer
    only_signer(deps.as_ref(), &sender)?;

    // Check if the signer has already voted
    if !DISABLE_FAST_PRICE_VOTES
        .may_load(deps.storage, &sender)?
        .unwrap_or(false)
    {
        return Err(ContractError::AlreadyEnabled {});
    }

    DISABLE_FAST_PRICE_VOTES.save(deps.storage, &sender, &false)?;
    let current_vote_count = DISABLE_FAST_PRICE_VOTE_COUNT
        .load(deps.storage)
        .unwrap_or_default();

    DISABLE_FAST_PRICE_VOTE_COUNT.save(
        deps.storage,
        &(current_vote_count.checked_sub(Uint256::one()).unwrap()),
    )?;

    Ok(Response::new()
        .add_attribute("method", "enable_fast_price")
        .add_attribute("sender", sender))
}

fn set_price(
    store: &mut dyn Storage,
    block_timestamp: Uint64,
    token: &Addr,
    _price: Uint256,
    vault_price_feed: &Addr,
    //TODO: use fast price events
    _fast_price_events: &Addr,
) -> Result<Response, ContractError> {
    // Assuming we have an equivalent function for getLatestPrimaryPrice
    let ref_price = get_latest_primary_price(vault_price_feed, token)?;

    let fast_price = PRICES.load(store, token).unwrap_or_default();
    let price_data_interval = PRICE_DATA_INTERVAL.load(store)?;
    // Fetch price data
    let PriceDataItem {
        ref_price: prev_ref_price,
        ref_time,
        mut cumulative_ref_delta,
        mut cumulative_fast_delta,
    } = PRICE_DATA.load(store, token).unwrap_or_default();

    if !ref_price.is_zero() {
        let (ref_delta_amount, fast_delta_amount) = if prev_ref_price > Uint256::zero() {
            let ref_delta_amount = if ref_price > prev_ref_price {
                ref_price.checked_sub(prev_ref_price).unwrap()
            } else {
                prev_ref_price.checked_sub(ref_price).unwrap()
            };

            let fast_delta_amount = if fast_price > _price {
                fast_price.checked_sub(_price).unwrap()
            } else {
                _price.checked_sub(fast_price).unwrap()
            };

            (ref_delta_amount, fast_delta_amount)
        } else {
            (Uint256::zero(), Uint256::zero())
        };

        if ref_time.checked_div(price_data_interval).unwrap()
            != block_timestamp.checked_div(price_data_interval).unwrap()
        {
            cumulative_ref_delta = Uint256::zero();
            cumulative_fast_delta = Uint256::zero();
        }

        //TODO: fix unwrap stuff here
        cumulative_ref_delta = cumulative_ref_delta
            .checked_add(
                ref_delta_amount
                    .checked_mul(CUMULATIVE_DELTA_PRECISION)
                    .unwrap()
                    .checked_div(prev_ref_price)
                    .unwrap(),
            )
            .unwrap();
        cumulative_fast_delta = cumulative_fast_delta
            .checked_add(
                fast_delta_amount
                    .checked_mul(CUMULATIVE_DELTA_PRECISION)
                    .unwrap()
                    .checked_div(fast_price)
                    .unwrap(),
            )
            .unwrap();
    }

    let new_price_data_item = PriceDataItem {
        ref_price,
        ref_time: block_timestamp,
        cumulative_ref_delta,
        cumulative_fast_delta,
    };

    PRICE_DATA.save(store, token, &new_price_data_item)?;
    PRICES.save(store, token, &_price)?;

    //TODO: Call the emit price event function...
    let msg = emit_price_event(_fast_price_events, token, Uint256::one())?;

    Ok(Response::new()
        .add_attribute("method", "set_price")
        .add_message(msg))
}

pub fn _set_prices_with_bits(
    deps: DepsMut,
    env: Env,
    price_bits: Uint256,
    _timestamp: Uint64,
) -> Result<Response, ContractError> {
    let should_update = set_last_updated_values(deps.storage, &env.block, _timestamp.u64())?;

    if should_update {
        let _fast_price_events: Addr = CONFIG.load(deps.storage)?.fast_price_events;
        let _vault_price_feed: Addr = VAULT_ADDRESS.load(deps.storage)?;
        let tokens: Vec<TokenData> = TOKEN_DATA.load(deps.storage)?;
        let price_array = price_bits.to_le_bytes();
        for j in 0..4 {
            if j >= tokens.len() {
                break;
            }

            let price = Uint256::from(u64::from_le_bytes(
                price_array[j * 8..j * 8 + 8].try_into().unwrap(),
            ));
            let token = &tokens[j].token;
            let token_precision = tokens[j].token_precision;
            let adjusted_price = price.multiply_ratio(PRICE_PRECISION, token_precision);

            set_price(
                deps.storage,
                Uint64::from(env.block.time.seconds()),
                token,
                adjusted_price,
                &_vault_price_feed,
                &_fast_price_events,
            )?;
        }
    }

    Ok(Response::new().add_attribute("method", "set_prices_with_bits"))
}

// for similicity we are considering timestamp represented in seconds instead of nano seconds
fn set_last_updated_values(
    store: &mut dyn Storage,
    block: &BlockInfo,
    timestamp: u64,
) -> Result<bool, StdError> {
    // Check for minBlockInterval
    let min_block_interval = CONFIG.load(store)?.min_block_interval;
    let mut last_updated = LAST_UPDATED.load(store).unwrap_or_default();

    if min_block_interval > Uint64::zero() {
        let blocks_passed = block
            .height
            .checked_sub(last_updated.last_updated_block)
            .ok_or(StdError::generic_err(
                "Overflow while calculating blocks passed",
            ))?;
        if blocks_passed < min_block_interval.u64() {
            return Err(StdError::generic_err(
                "FastPriceFeed: minBlockInterval not yet passed",
            ));
        }
    }

    let max_time_deviation = MAX_TIME_DEVIATION.load(store).unwrap_or_default();
    if max_time_deviation > block.time.seconds() {
        return Err(StdError::generic_err(
            "FastPriceFeed: _timestamp below allowed range",
        ));
    }

    let lower_bound = block.time.minus_seconds(max_time_deviation);
    let upper_bound = block.time.plus_seconds(max_time_deviation);

    if timestamp < lower_bound.seconds() {
        return Err(StdError::generic_err(
            "FastPriceFeed: _timestamp below allowed range",
        ));
    }

    if timestamp > upper_bound.seconds() {
        return Err(StdError::generic_err(
            "FastPriceFeed: _timestamp exceeds allowed range",
        ));
    }

    // Do not update prices if _timestamp is before the current lastUpdatedAt value
    if timestamp < last_updated.last_updated_at {
        return Ok(false);
    }

    last_updated.last_updated_at = timestamp;
    last_updated.last_updated_block = block.height;

    LAST_UPDATED.save(store, &last_updated)?;
    Ok(true)
}

//TODO: actual contract call with price event emission
fn emit_price_event(fast_price_events: &Addr, _: &Addr, _: Uint256) -> StdResult<CosmosMsg> {
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: fast_price_events.to_string(),
        msg: to_binary("required price")?,
        funds: vec![],
    });

    Ok(msg)
}
