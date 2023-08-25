use crate::state::{Config, PriceDataItem, SpreadBasisPoint, TokenData};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint256, Uint64};

#[cw_serde]
pub struct InstantiateMsg {
    pub config: Config,
}

#[cw_serde]
pub enum ExecuteMsg {
    Initialize {
        min_auth: Uint256,
        signers: Vec<Addr>,
        updaters: Vec<Addr>,
    },

    SetSigner {
        account: Addr,
        is_active: bool,
    },

    SetUpdater {
        account: Addr,
        is_active: bool,
    },

    SetFastPriceEvents {
        fast_price_events: Addr,
    },

    SetVaultPriceFeed {
        vault_price_feed: Addr,
    },

    SetMaxTimeDeviation {
        max_time_deviation: Uint64,
    },
    SetPriceDuration {
        price_duration: Uint64,
    },
    SetMaxPriceUpdateDelay {
        max_price_update_delay: Uint64,
    },

    SetSpreadBasisPointsIfInactive {
        spread_basis_points_if_inactive: Uint256,
    },
    SetSpreadBasisPointsIfChainError {
        spread_basis_points_if_chain_error: Uint256,
    },
    SetMinBlockInterval {
        min_block_interval: Uint64,
    },
    SetIsSpreadEnabled {
        spread_enabled: bool,
    },
    SetLastUpdatedAt {
        last_updated_at: Uint64,
    },
    SetTokenManager {
        token_manager: Addr,
    },
    SetMaxDeviationBasisPoints {
        max_deviation_basis_points: Uint256,
    },
    SetMaxCumulativeDeltaDiffs {
        tokens: Vec<Addr>,
        max_cumulative_delta_diffs: Vec<Uint256>,
    },
    SetPriceDataInterval {
        price_data_interval: Uint64,
    },
    SetMinAuthorizations {
        min_authorizations: Uint256,
    },
    SetTokens {
        tokens: Vec<Addr>,
        token_precision: Vec<Uint256>,
    },
    SetPrices {
        tokens: Vec<Addr>,
        prices: Vec<Uint256>,
        timestamp: Uint64,
    },
    SetCompactedPrices {
        price_bit_array: Vec<Uint256>,
        timestamp: Uint64,
    },
    SetPricesWithBits {
        price_bits: Uint256,
        timestamp: Uint64,
    },
    SetPricesWithBitsAndExecute {
        sender: Addr,
        position_router_addr: Addr,
        price_bits: Uint256,
        timestamp: Uint64,
        end_index_for_increase_positions: Uint256,
        end_index_for_decrease_positions: Uint256,
        max_increase_positions: Uint256,
        max_decrease_positions: Uint256,
    },
    DisableFastPrice,
    EnableFastPrice,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Uint256)]
    GetPrice {
        token: Addr,
        block_timestamp: Uint64,
        ref_price: Uint256,
        maximise: bool,
    },

    #[returns(bool)]
    FavorFastPrice { token: Addr },

    #[returns(PriceDataItem)]
    GetPriceData { token: Addr },

    #[returns(Config)]
    GetConfig,

    #[returns(bool)]
    IsUpdater { address: Addr },

    #[returns(Uint256)]
    Prices { address: Addr },

    #[returns(PriceDataItem)]
    PriceData { address: Addr },

    #[returns(Uint256)]
    MaxCumulativeDeltaDiffs { address: Addr },

    #[returns(bool)]
    IsSigner { address: Addr },

    #[returns(bool)]
    DisableFastPriceVotes { address: Addr },

    #[returns(Uint256)]
    DisableFastPriceVoteCount,

    #[returns(Uint256)]
    MinAuthorizations,

    #[returns(Uint256)]
    MaxTimeDeviation,

    #[returns(SpreadBasisPoint)]
    SpreadBasisPoint,

    #[returns(Vec< TokenData >)]
    TokenData,
}
