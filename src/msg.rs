use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, Uint128};
use cw20::Cw20ReceiveMsg;
use cw721::Cw721ReceiveMsg;

use crate::state::Config;

#[cw_serde]
pub struct InstantiateMsg {
    pub scientist_nft_addr: String,
    pub ninja_nft_addr: String,

    pub qtum_addr: String,
    pub xqtum_addr: String,

    pub usdt_denom: String,

    pub nft_price: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    ReceiveNft(Cw721ReceiveMsg),
    Unlock {
        id: String,
    },
    Unstake {
        amount: Uint128,
    },
    // withdraw pending rewards
    Withdraw {},
    SetDistribution {
        nft_addr: String,
        start_date: u64,
        end_date: u64,
        amount: Uint128,
    },
    UpdateConfig {
        config: Config,
    },
}

#[cw_serde]
pub enum Cw20HookMsg {
    Mint { id: String, nft_addr: String },
    Stake {},
}

#[cw_serde]
pub enum Cw721HookMsg {
    Lock { id: String },
}

// query msgs

#[cw_serde]
pub enum QueryMsg {
    Config {},
    State { block_time: Option<u64> },
    StakerInfo { staker: String },
}

// We define a custom struct for each query response
#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,

    pub scientist_nft_addr: String,
    pub ninja_nft_addr: String,

    pub qtum_addr: String,
    pub xqtum_addr: String,

    pub nft_price: Uint128,

    pub ninja_distribution_schedule: Vec<(u64, u64, Uint128)>,
    pub scientist_distribution_schedule: Vec<(u64, u64, Uint128)>,
}

#[cw_serde]
pub struct StateResponse {
    pub ninja_total_staker: u64,
    pub ninja_total_stake_amount: Uint128,
    pub ninja_last_distributed: u64,
    pub ninja_global_reward_index: Decimal,
    pub scientist_total_staker: u64,
    pub scientist_total_stake_amount: Uint128,
    pub scientist_last_distributed: u64,
    pub scientist_global_reward_index: Decimal,
}

#[cw_serde]
pub struct StakerInfoResponse {
    pub stake_amount: Uint128,
    pub pending_rewards: Uint128,
    pub reward_claimed: Uint128,
    pub nft_addr: Option<String>,
    pub token_id: Option<String>,
}
