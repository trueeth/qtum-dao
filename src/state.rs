

use cosmwasm_schema::cw_serde;

use cosmwasm_std::{Addr, Decimal,  Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {

    pub owner: Addr,

    pub scientist_nft_addr: Addr,
    pub ninja_nft_addr: Addr,

    pub qtum_addr: Addr,
    pub xqtum_addr: Addr,

    pub nft_price: Uint128,

    pub ninja_distribution_schedule: Vec<(u64, u64, Uint128)>,
    pub scientist_distribution_schedule: Vec<(u64, u64, Uint128)>
}



#[cw_serde]
pub struct GuildState {

    pub total_staker: u64,
    pub total_stake_amount: Uint128,
    pub last_distributed: u64,
    pub global_reward_index: Decimal
    
}


#[cw_serde]
pub struct StakerInfo {
    pub stake_amount: Uint128,
    pub pending_rewards: Uint128,
    pub reward_index: Decimal,
    pub reward_claimed: Uint128,
    pub ninja_locked: bool,
    pub scientist_locked: bool
}



pub const CONFIG : Item<Config> = Item::new("config");
pub const NINJA_STATE : Item<GuildState> = Item::new("ninja_state");
pub const SCIENTIST_STATE : Item<GuildState> = Item::new("scientist_state");
pub const NINJA_USERS: Map<&str, StakerInfo> = Map::new("ninja_user_info");
pub const SCIENTIST_USERS: Map<&str, StakerInfo> = Map::new("scientist_user_info");