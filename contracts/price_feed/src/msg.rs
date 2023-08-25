use crate::query::GetRoundDataResult;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Int256, Uint128};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    SetAdmin { admin: Addr, status: bool },
    SetLatestAnswer(Int256),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Uint128)]
    GetLatestRound,

    #[returns(Int256)]
    GetLatestAnswer,
    #[returns(GetRoundDataResult)]
    GetRoundData { round_id: Uint128 },
}
