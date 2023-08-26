use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint256, Uint64};
use cw_storage_plus::{Item, Map};

pub const IS_INITIALIZED: Item<bool> = Item::new("is_initialized");
pub const GOV: Item<Addr> = Item::new("GOV");
pub const VAULT_ADDRESS: Item<Addr> = Item::new("vault_address");
pub const MIN_AUTH: Item<Uint256> = Item::new("MIN_AUTH");
pub const CONFIG: Item<Config> = Item::new("config");
pub const PRICE_DATA_STATE: Item<PriceDataItem> = Item::new("price_data_item");
pub const SPREAD_BASIS_POINT_STATE: Item<SpreadBasisPoint> = Item::new("spread_basis_point");
pub const SPREAD_ENABLED: Item<bool> = Item::new("spread_enabled");
pub const DISABLE_FAST_PRICE_VOTE_COUNT: Item<Uint256> = Item::new("disableFastPriceVotecount");
pub const TOKEN_DATA: Item<Vec<TokenData>> = Item::new("token_data");
pub const LAST_UPDATED: Item<LastUpdated> = Item::new("last_updated");
pub const MAX_TIME_DEVIATION: Item<u64> = Item::new("max_time_deviation");
pub const TOKEN_MANAGER: Item<Addr> = Item::new("token_manager");
pub const PRICE_DATA_INTERVAL: Item<Uint64> = Item::new("price_data_interval");

pub const IS_UPDATER: Map<&Addr, bool> = Map::new("isUpdater");
pub const IS_SIGNER: Map<&Addr, bool> = Map::new("isSigner");
pub const PRICES: Map<&Addr, Uint256> = Map::new("prices");
pub const DISABLE_FAST_PRICE_VOTES: Map<&Addr, bool> = Map::new("disableFastPriceVotes");
pub const MAX_CUMULATIVE_DELTA_DIFFS: Map<&Addr, Uint256> = Map::new("maxCumulativeDeltaDiffs");
pub const PRICE_DATA: Map<&Addr, PriceDataItem> = Map::new("priceData");

#[cw_serde]
pub struct Config {
    pub price_duration: Uint64,
    pub max_price_update_delay: Uint64,
    pub min_block_interval: Uint64,
    pub max_deviation_basis_points: Uint256,
    pub fast_price_events: Addr,
    pub token_manager: Addr,
}

#[cw_serde]
#[derive(Default)]
pub struct PriceDataItem {
    pub ref_price: Uint256,
    pub ref_time: Uint64,
    pub cumulative_ref_delta: Uint256,
    pub cumulative_fast_delta: Uint256,
}

#[cw_serde]
#[derive(Default)]
pub struct SpreadBasisPoint {
    pub spread_basis_points_if_inactive: Uint256,
    pub spread_basis_points_if_chain_error: Uint256,
}

#[cw_serde]
#[derive(Default)]
pub struct LastUpdated {
    pub last_updated_at: u64,
    pub last_updated_block: u64,
}

#[cw_serde]
pub struct TokenData {
    pub token: Addr,
    pub token_precision: Uint256,
}

impl TokenData {
    pub fn new(token: Addr, token_precision: Uint256) -> Self {
        Self {
            token,
            token_precision,
        }
    }
}
