

use cosmwasm_schema::cw_serde;

use cosmwasm_std::{Addr, Decimal, StdResult, Storage, Uint128};
use cosmwasm_storage::{singleton, singleton_read, Bucket, ReadonlyBucket};
use cw_storage_plus::{Item, Map};




static KEY_CONFIG: &[u8] = b"config";
static KEY_STATE: &[u8] = b"state";

static PREFIX_REWARD: &[u8] = b"reward";


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


pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    singleton(storage, KEY_CONFIG).save(config)
}


pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    singleton_read(storage, KEY_CONFIG).load()
}



#[cw_serde]
pub struct StakeAmount {
    pub ninja: Uint128,
    pub scientist: Uint128
}

#[cw_serde]
pub struct PendingRewards {
    pub ninja: Uint128,
    pub scientist: Uint128
}

#[cw_serde]
pub struct RewardIndex {
    pub ninja: Decimal,
    pub scientist: Decimal
}


#[cw_serde]
pub struct RewardClaimed {
    pub ninja: Uint128,
    pub scientist: Uint128
}


#[cw_serde]
pub struct LastDistributed {
    pub ninja: u64,
    pub scientist: u64
}


#[cw_serde]
pub struct State {

    pub total_staker: u64,
    pub total_stake_amount: StakeAmount,
    pub last_distributed: LastDistributed,
    pub global_reward_index: RewardIndex
    
}

pub fn store_state(storage: &mut dyn Storage, state: &State) -> StdResult<()> {
    singleton(storage, KEY_STATE).save(state)
}

pub fn read_state(storage: &dyn Storage) -> StdResult<State> {
    singleton_read(storage, KEY_STATE).load()
}




#[cw_serde]
pub struct StakerInfo {
    pub stake_amount: StakeAmount,
    pub pending_rewards: PendingRewards,
    pub reward_index: RewardIndex,
    pub reward_claimed: RewardClaimed,
    pub ninja_locked: bool,
    pub scientist_locked: bool
}

/// returns return staker_info of the given owner
pub fn store_staker_info(
    storage: &mut dyn Storage,
    owner: &Addr,
    staker_info: &StakerInfo,
) -> StdResult<()> {
    Bucket::new(storage, PREFIX_REWARD).save(owner.as_bytes(), staker_info)
}

/// remove staker_info of the given owner
pub fn remove_staker_info(storage: &mut dyn Storage, owner: &Addr) {
    Bucket::<StakerInfo>::new(storage, PREFIX_REWARD).remove(owner.as_bytes())
}

/// returns rewards owned by this owner
/// (read-only version for queries)
pub fn read_staker_info(storage: &dyn Storage, owner: &Addr) -> StdResult<StakerInfo> {
    match ReadonlyBucket::new(storage, PREFIX_REWARD).may_load(owner.as_bytes())? {
        Some(staker_info) => Ok(staker_info),
        None => Ok(StakerInfo {
            stake_amount:  StakeAmount{
                ninja: Uint128::zero(),
                scientist: Uint128::zero()
            },
            pending_rewards: PendingRewards {
                ninja: Uint128::zero(),
                scientist: Uint128::zero()
            },
            reward_claimed: RewardClaimed {
                ninja: Uint128::zero(),
                scientist: Uint128::zero()
            },
            reward_index: RewardIndex {
                ninja: Decimal::zero(),
                scientist: Decimal::zero()
            },
            ninja_locked: false,
            scientist_locked: false,
        }),
    }
}



