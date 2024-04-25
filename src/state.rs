use cosmwasm_schema::cw_serde;

use cosmwasm_std::{Addr, Decimal, Deps, Response, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owner: Addr,

    pub scientist_nft_addr: Addr,
    pub ninja_nft_addr: Addr,

    pub qtum_addr: Addr,
    pub xqtum_addr: Addr,

    pub nft_price: Uint128,

    pub usdt_denom: String,

    pub ninja_distribution_schedule: Vec<(u64, u64, Uint128)>,
    pub scientist_distribution_schedule: Vec<(u64, u64, Uint128)>,
}

#[cw_serde]
pub struct GuildState {
    pub total_rewards_distributed: Uint128,
    pub total_staker: u64,
    pub total_stake_amount: Uint128,
    pub last_distributed: u64,
    pub global_reward_index: Decimal,
}

#[cw_serde]
pub struct StakerInfo {
    pub stake_amount: Uint128,
    pub pending_rewards: Uint128,
    pub reward_index: Decimal,
    pub reward_claimed: Uint128,
    pub nft_addr: Option<String>,
    pub token_id: Option<String>,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const NINJA_GUILD: Item<GuildState> = Item::new("ninja_state");
pub const SCIENTIST_GUILD: Item<GuildState> = Item::new("scientist_state");
pub const USER_STAKING: Map<&str, StakerInfo> = Map::new("ninja_user_info");

pub fn user_staking(deps: Deps, sender: &str) -> StdResult<StakerInfo> {
    let staking_info = USER_STAKING.may_load(deps.storage, sender).unwrap();

    match staking_info {
        Some(staking_info) => Ok(staking_info),
        None => Ok(StakerInfo {
            stake_amount: Uint128::zero(),
            pending_rewards: Uint128::zero(),
            reward_claimed: Uint128::zero(),
            reward_index: Decimal::zero(),
            nft_addr: None,
            token_id: None,
        }),
    }
}

pub fn store_user_staking(
    storage: &mut dyn Storage,
    owner: &str,
    staker_info: &StakerInfo,
) -> StdResult<Response> {
    USER_STAKING.save(storage, owner, staker_info)?;
    Ok(Response::new())
}

pub fn remove_user_staking(storage: &mut dyn Storage, owner: &str) -> StdResult<Response> {
    USER_STAKING.remove(storage, owner);
    Ok(Response::new())
}

pub fn guild_state(deps: Deps, nft_addr: String) -> StdResult<GuildState> {
    let guild_info: GuildState;

    let config = CONFIG.load(deps.storage)?;

    if nft_addr == config.ninja_nft_addr {
        guild_info = NINJA_GUILD.load(deps.storage).unwrap();
    } else {
        guild_info = SCIENTIST_GUILD.load(deps.storage).unwrap();
    }

    return Ok(guild_info);
}

pub fn store_guild_state(
    storage: &mut dyn Storage,
    guild_state: &GuildState,
    nft_addr: String,
) -> StdResult<Response> {
    let config = CONFIG.load(storage)?;

    if nft_addr == config.ninja_nft_addr {
        NINJA_GUILD.save(storage, guild_state).unwrap();
    } else {
        SCIENTIST_GUILD.save(storage, guild_state).unwrap();
    }

    return Ok(Response::new());
}
