

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal,  Uint128};
use cw20::Cw20ReceiveMsg;
use cw721::Cw721ReceiveMsg;

use crate::state::Config;

#[cw_serde]
pub struct InstantiateMsg {
    pub scientist_nft_addr: String,
    pub ninja_nft_addr: String,

    pub qtum_addr: String,
    pub xqtum_addr: String,

    pub nft_price: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    ReceiveNft(Cw721ReceiveMsg),
    Unstake {
        amount: Uint128,
        nft_addr: String
    },
    // withdraw pending rewards
    Withdraw{ nft_addr: String},
    SetDistribution { nft_addr: String, schedule: Vec<(u64, u64, Uint128)>},
    UpdateConfig {config: Config}
}




#[cw_serde]
pub enum Cw20HookMsg {
    Mint {
        id: String,
        nft_addr: String
    }, 
    Stake {
        nft_addr: String
    }
}

#[cw_serde]
pub enum Cw721HookMsg {
    Mint {
        id: String
    }
}




// query msgs


#[cw_serde]
pub enum QueryMsg {
    Config {},
    State {
        block_time: Option<u64>,
    },
    StakerInfo {
        staker: String,
        block_time: Option<u64>,
    },
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
    pub scientist_distribution_schedule: Vec<(u64, u64, Uint128)>
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
    pub scientist_global_reward_index: Decimal
}

#[cw_serde]
pub struct StakerInfoResponse {
    pub ninja_stake_amount: Uint128,
    pub ninja_pending_rewards: Uint128,
    pub ninja_reward_claimed: Uint128,
    pub scientist_stake_amount: Uint128,
    pub scientist_pending_rewards: Uint128,
    pub scientist_reward_claimed: Uint128,
    pub ninja_locked: bool,
    pub scientist_locked: bool
}