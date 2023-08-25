use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Int256, Uint128};

#[cw_serde]
pub struct GetRoundDataResult {
    pub round_id: Uint128,
    pub answer: Int256,
    pub started_at: Uint128,
    pub updated_at: Uint128,
    pub answered_in_round: Uint128,
}
