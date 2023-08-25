use crate::state::{
    PriceDataItem, CONFIG, DISABLE_FAST_PRICE_VOTE_COUNT, LAST_UPDATED, MAX_CUMULATIVE_DELTA_DIFFS,
    MIN_AUTH, PRICES, PRICE_DATA, SPREAD_BASIS_POINT_STATE, SPREAD_ENABLED,
};
use cosmwasm_std::{Addr, Deps, StdResult, Uint256, Uint64};

const BASIS_POINTS_DIVISOR: Uint256 = Uint256::from_u128(10000u128);

pub fn get_price(
    deps: Deps,
    token: Addr,
    block_timestamp: Uint64,
    ref_price: Uint256,
    maximise: bool,
) -> StdResult<Uint256> {
    let config = CONFIG.load(deps.storage)?;
    let spread = SPREAD_BASIS_POINT_STATE.load(deps.storage)?;

    let current_time: Uint64 = block_timestamp;
    let last_updated_at = LAST_UPDATED.load(deps.storage)?.last_updated_at;
    let max_price_update_delay = config.max_price_update_delay;
    let price_duration: Uint64 = config.price_duration;

    let basis_points_divisor = BASIS_POINTS_DIVISOR;
    let spread_basis_points_if_chain_error = spread.spread_basis_points_if_chain_error;
    let spread_basis_points_if_inactive = spread.spread_basis_points_if_inactive;
    let max_deviation_basis_points = config.max_deviation_basis_points;

    if current_time.u64() > (last_updated_at + max_price_update_delay.u64()) {
        return calculate_price_with_spread(
            ref_price,
            spread_basis_points_if_chain_error,
            basis_points_divisor,
            maximise,
        );
    }

    if current_time.u64() > (last_updated_at + price_duration.u64()) {
        return calculate_price_with_spread(
            ref_price,
            spread_basis_points_if_inactive,
            basis_points_divisor,
            maximise,
        );
    }

    let fast_price = PRICES.load(deps.storage, &token)?;

    if fast_price == Uint256::zero() {
        return Ok(ref_price);
    }

    let diff_basis_points = if ref_price > fast_price {
        (ref_price - fast_price) * basis_points_divisor / ref_price
    } else {
        (fast_price - ref_price) * basis_points_divisor / ref_price
    };

    let result = favor_fast_price(deps, token)?;
    let has_spread = !result || diff_basis_points > max_deviation_basis_points;

    if has_spread {
        let result_price = if maximise {
            if ref_price > fast_price {
                ref_price
            } else {
                fast_price
            }
        } else if ref_price < fast_price {
                ref_price
            } else {
                fast_price
            };
        

        return Ok(result_price);
    }

    Ok(fast_price)
}

pub fn favor_fast_price(deps: Deps, token: Addr) -> StdResult<bool> {
    let is_spread_enabled = SPREAD_ENABLED.load(deps.storage)?;

    if is_spread_enabled {
        return Ok(false);
    }

    let disable_fast_price_vote_count = DISABLE_FAST_PRICE_VOTE_COUNT.load(deps.storage)?;
    let min_authorizations = MIN_AUTH.load(deps.storage)?;

    if disable_fast_price_vote_count >= min_authorizations {
        return Ok(false);
    }

    let price_data = PRICE_DATA.load(deps.storage, &token)?;

    let max_cumulative_delta_diff = MAX_CUMULATIVE_DELTA_DIFFS.load(deps.storage, &token)?;

    if price_data
        .cumulative_fast_delta
        .gt(&price_data.cumulative_ref_delta)
        && price_data
            .cumulative_fast_delta
            .checked_sub(price_data.cumulative_ref_delta)
            .unwrap()
            .gt(&max_cumulative_delta_diff)
    {
        return Ok(false);
    }

    Ok(true)
}

pub fn get_price_data(deps: Deps, token: Addr) -> StdResult<PriceDataItem> {
    let price_data = PRICE_DATA.load(deps.storage, &token)?;
    Ok(price_data)
}

fn calculate_price_with_spread(
    ref_price: Uint256,
    spread: Uint256,
    divisor: Uint256,
    maximise: bool,
) -> StdResult<Uint256> {
    let result_price = if maximise {
        ref_price * (divisor + spread) / divisor
    } else {
        ref_price * (divisor - spread) / divisor
    };

    Ok(result_price)
}
