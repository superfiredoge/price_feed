use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Int256, Uint128};
use cw_storage_plus::{Item, Map};

pub const PRICE_FEED_STATE: Item<PriceFeedState> = Item::new("price_feed_state");
pub const LATEST_ROUND: Item<PriceFeedState> = Item::new("price_feed_state");
pub const PRICE_FEED_ANSWERS: Map<u128, Int256> = Map::new("price_feed_answers");
pub const PRICE_FEED_ADMINS: Map<Addr, bool> = Map::new("price_feed_admins");
pub const PRICE_FEED_GOV: Item<Addr> = Item::new("price_feed_gov");

#[cw_serde]
pub struct PriceFeedState {
    pub round_id: Uint128,
    pub answer: Int256,
}

impl Default for PriceFeedState {
    fn default() -> Self {
        Self {
            round_id: Uint128::zero(),
            answer: Int256::zero(),
        }
    }
}
