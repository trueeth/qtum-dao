use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, Cw20HookMsg, Cw721HookMsg, ExecuteMsg, InstantiateMsg, QueryMsg,
    StakerInfoResponse, StateResponse,
};
use crate::state::{
    guild_state, remove_user_staking, store_guild_state, store_user_staking, user_staking, Config,
    GuildState, StakerInfo, CONFIG, NINJA_GUILD, SCIENTIST_GUILD,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_json, to_json_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut,
    Empty, Env, MessageInfo, Response, StdResult, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use cw721::Cw721ReceiveMsg;
use cw721_base::{ExecuteMsg as Cw721ExecuteMsg, MintMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:qtuamdao";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.save(
        deps.storage,
        &Config {
            owner: info.sender,
            scientist_nft_addr: deps.api.addr_validate(&msg.scientist_nft_addr)?,
            ninja_nft_addr: deps.api.addr_validate(&msg.ninja_nft_addr)?,
            qtum_addr: deps.api.addr_validate(&msg.qtum_addr)?,
            xqtum_addr: deps.api.addr_validate(&msg.xqtum_addr)?,
            usdt_denom: msg.usdt_denom,
            nft_price: msg.nft_price,
            ninja_distribution_schedule: vec![],
            scientist_distribution_schedule: vec![],
        },
    )?;

    NINJA_GUILD.save(
        deps.storage,
        &GuildState {
            total_rewards_distributed: Uint128::zero(),
            total_staker: 0,
            last_distributed: env.block.time.seconds(),
            total_stake_amount: Uint128::zero(),
            global_reward_index: Decimal::zero(),
        },
    )?;

    SCIENTIST_GUILD.save(
        deps.storage,
        &GuildState {
            total_rewards_distributed: Uint128::zero(),
            total_staker: 0,
            last_distributed: env.block.time.seconds(),
            total_stake_amount: Uint128::zero(),
            global_reward_index: Decimal::zero(),
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => cw20_receive(deps, env, info, msg),
        ExecuteMsg::ReceiveNft(msg) => cw721_receive(deps, env, info, msg),
        ExecuteMsg::Unlock { id } => unlock_nft(deps, env, info.sender, id),
        ExecuteMsg::Unstake { amount } => unstake_xqtum(deps, env, info.sender, amount),
        ExecuteMsg::Withdraw {} => withdraw_reward(deps, env, info),
        ExecuteMsg::SetDistribution {
            nft_addr,
            start_date,
            end_date,
            amount,
        } => set_distribution_schedule(deps, env, info, nft_addr, start_date, end_date, amount),
        ExecuteMsg::UpdateConfig { config } => update_config(deps, env, info, config),
    }
}

pub fn cw20_receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    match from_json(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Mint { id, nft_addr }) => {
            // only qtum token contract can execute this message
            if config.qtum_addr != deps.api.addr_validate(info.sender.as_str())? {
                return Err(ContractError::UnsupportedToken {});
            };

            if config.nft_price > cw20_msg.amount {
                return Err(ContractError::InsufficientToken {});
            };

            let cw20_sender = deps.api.addr_validate(&cw20_msg.sender)?;
            mint_nft(cw20_sender, id, nft_addr)
        }

        Ok(Cw20HookMsg::Stake {}) => {
            // only qtum token contract can execute this message
            if config.xqtum_addr != deps.api.addr_validate(info.sender.as_str())? {
                return Err(ContractError::UnsupportedToken {});
            };

            let cw20_sender = deps.api.addr_validate(&cw20_msg.sender)?;
            stake_xqtum(deps, env, cw20_sender, cw20_msg.amount)
        }

        Err(_) => Err(ContractError::Unauthorized {}),
    }
}

pub fn cw721_receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw721_msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    match from_json(&cw721_msg.msg) {
        Ok(Cw721HookMsg::Lock { id }) => {
            // only qtum token contract can execute this message
            let nft_addr = deps.api.addr_validate(info.sender.as_str())?;

            if config.ninja_nft_addr != nft_addr && config.scientist_nft_addr != nft_addr {
                return Err(ContractError::UnsupportedToken {});
            };

            let sender = deps.api.addr_validate(&cw721_msg.sender)?;
            lock_nft(deps, env, sender, id, nft_addr.to_string())
        }

        Err(_) => Err(ContractError::Unauthorized {}),
    }
}

pub fn stake_xqtum(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let mut staker_info: StakerInfo = user_staking(deps.as_ref(), sender.as_str())?;

    if staker_info.token_id == None {
        return Err(ContractError::CustomError {
            msg: "lock  nft before you staking xqtum".to_string(),
        });
    }

    let nft_addr = staker_info.nft_addr.clone().unwrap();
    let mut guild_state: GuildState = guild_state(deps.as_ref(), nft_addr.clone())?;

    compute_reward(
        &config,
        &mut guild_state,
        env.block.time.seconds(),
        nft_addr.clone(),
    );
    compute_staker_reward(&guild_state, &mut staker_info)?;
    // Increase bond_amount
    increase_stake_amount(&mut guild_state, &mut staker_info, amount);

    store_user_staking(deps.storage, sender.as_str(), &staker_info)?;
    store_guild_state(deps.storage, &guild_state, nft_addr.clone())?;

    Ok(Response::new().add_attributes(vec![
        ("action", "bond"),
        ("owner", sender.as_str()),
        ("amount", amount.to_string().as_str()),
    ]))
}

pub fn unstake_xqtum(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let mut staker_info: StakerInfo = user_staking(deps.as_ref(), sender.as_str())?;

    if staker_info.stake_amount < amount {
        return Err(ContractError::InsufficientToken {});
    }

    let nft_addr = staker_info.nft_addr.clone().unwrap();
    let mut guild_state: GuildState = guild_state(deps.as_ref(), nft_addr.clone())?;
    compute_reward(
        &config,
        &mut guild_state,
        env.block.time.seconds(),
        nft_addr.clone(),
    );

    compute_staker_reward(&guild_state, &mut staker_info)?;
    // decrease bond_amount
    decrease_stake_amount(&mut guild_state, &mut staker_info, amount);

    store_user_staking(deps.storage, sender.as_str(), &staker_info)?;
    store_guild_state(deps.storage, &guild_state, nft_addr.clone())?;

    Ok(Response::new()
        .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.xqtum_addr.to_string(),
            msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
                recipient: sender.to_string(),
                amount,
            })?,
            funds: vec![],
        })])
        .add_attributes(vec![
            ("action", "unbond"),
            ("owner", sender.to_string().as_str()),
            ("amount", amount.to_string().as_str()),
        ]))
}

pub fn mint_nft(sender: Addr, id: String, nft_addr: String) -> Result<Response, ContractError> {
    let mint_msg: Cw721ExecuteMsg<Empty, Empty> = Cw721ExecuteMsg::Mint(MintMsg {
        owner: sender.to_string(),
        token_id: id.to_string(),
        // Some(format!("{}/{}", config.base_token_uri, mintable_token_id))
        token_uri: None,
        extension: Empty {},
    });

    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: nft_addr.to_string(),
        msg: to_json_binary(&mint_msg)?,
        funds: vec![],
    });

    let res = Response::new()
        .add_message(msg)
        .add_attribute("action", "mint")
        .add_attribute("nft_address", nft_addr)
        .add_attribute("token_id", id)
        .add_attribute("to", sender);

    Ok(res)
}

pub fn lock_nft(
    deps: DepsMut,
    _env: Env,
    sender: Addr,
    id: String,
    nft_addr: String,
) -> Result<Response, ContractError> {
    let mut staker_info: StakerInfo = user_staking(deps.as_ref(), sender.as_str())?;
    let mut guild_state: GuildState = guild_state(deps.as_ref(), nft_addr.clone())?;

    if staker_info.token_id != None {
        return Err(ContractError::CustomError {
            msg: "You already locked nft".to_string(),
        });
    }

    staker_info.token_id = Some(id.clone());
    staker_info.nft_addr = Some(nft_addr.clone());
    guild_state.total_staker += 1;

    store_user_staking(deps.storage, sender.as_str(), &staker_info)?;
    store_guild_state(deps.storage, &guild_state, nft_addr.clone())?;

    Ok(Response::new()
        .add_attribute("action", "lock_nft")
        .add_attribute("owner", sender)
        .add_attribute("nft_address", nft_addr.clone())
        .add_attribute("token_id", id))
}

pub fn unlock_nft(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    id: String,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    let mut staker_info: StakerInfo = user_staking(deps.as_ref(), sender.as_str())?;

    if staker_info.token_id != Some(id.clone()) {
        return Err(ContractError::Unauthorized {});
    }

    let nft_addr = staker_info.nft_addr.clone().unwrap();
    let mut guild_state: GuildState = guild_state(deps.as_ref(), nft_addr.clone())?;
    let mut unlock_msg: Vec<CosmosMsg> = vec![];

    // Compute global reward & staker reward
    compute_reward(
        &config,
        &mut guild_state,
        env.block.time.seconds(),
        nft_addr.clone(),
    );
    compute_staker_reward(&guild_state, &mut staker_info)?;

    let reward_amount = staker_info.pending_rewards;

    if staker_info.stake_amount > Uint128::zero() {
        let unstake_msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.xqtum_addr.to_string(),
            msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
                recipient: sender.to_string(),
                amount: staker_info.stake_amount,
            })?,
            funds: vec![],
        });
        unlock_msg.push(unstake_msg);
    }

    if staker_info.pending_rewards > Uint128::zero() {
        let claim_reward_msg;

        if nft_addr == config.ninja_nft_addr {
            claim_reward_msg = CosmosMsg::Bank(BankMsg::Send {
                to_address: sender.to_string(),
                amount: vec![Coin::new(reward_amount.u128(), "inj")],
            });
        } else {
            claim_reward_msg = CosmosMsg::Bank(BankMsg::Send {
                to_address: sender.to_string(),
                amount: vec![Coin::new(reward_amount.u128(), config.usdt_denom)],
            });
        }
        unlock_msg.push(claim_reward_msg);
    }

    let msg: Cw721ExecuteMsg<Empty, Empty> = Cw721ExecuteMsg::TransferNft {
        recipient: sender.to_string(),
        token_id: id.clone(),
    };

    let transfer_nft_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: nft_addr.clone(),
        msg: to_json_binary(&msg)?,
        funds: vec![],
    });

    unlock_msg.push(transfer_nft_msg);

    guild_state.total_staker -= 1;
    guild_state.total_stake_amount -= staker_info.stake_amount;

    remove_user_staking(deps.storage, sender.as_str())?;
    store_guild_state(deps.storage, &guild_state, nft_addr.clone())?;

    Ok(Response::new()
        .add_messages(unlock_msg)
        .add_attribute("action", "unlock_nft")
        .add_attribute("to", sender)
        .add_attribute("nft_address", nft_addr.clone())
        .add_attribute("token_id", id))
}

// withdraw rewards to executor
pub fn withdraw_reward(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    let mut staker_info: StakerInfo = user_staking(deps.as_ref(), info.sender.as_str())?;

    let nft_addr = staker_info.nft_addr.clone().unwrap();
    let mut guild_state: GuildState = guild_state(deps.as_ref(), nft_addr.clone())?;

    let withdraw_msg: CosmosMsg<Empty>;

    // Compute global reward & staker reward
    compute_reward(
        &config,
        &mut guild_state,
        env.block.time.seconds(),
        nft_addr.clone(),
    );
    compute_staker_reward(&guild_state, &mut staker_info)?;

    let amount = staker_info.pending_rewards;
    staker_info.pending_rewards = Uint128::zero();
    staker_info.reward_claimed += amount;

    if nft_addr.clone() == config.ninja_nft_addr {
        withdraw_msg = CosmosMsg::Bank(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![Coin::new(amount.u128(), "inj")],
        });
    } else {
        withdraw_msg = CosmosMsg::Bank(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![Coin::new(amount.u128(), config.usdt_denom)],
        });
    }

    store_user_staking(deps.storage, info.sender.as_str(), &staker_info)?;
    store_guild_state(deps.storage, &guild_state, nft_addr.clone())?;

    // Store updated state

    Ok(Response::new()
        .add_message(withdraw_msg)
        .add_attributes(vec![
            ("action", "withdraw"),
            ("owner", info.sender.as_str()),
            ("amount", amount.to_string().as_str()),
        ]))
}

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_config: Config,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    CONFIG.save(deps.storage, &new_config)?;

    Ok(Response::new().add_attributes(vec![("action", "update_config")]))
}

pub fn set_distribution_schedule(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    nft_addr: String,
    start_date: u64,
    end_date: u64,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    let mut guild_state: GuildState = guild_state(deps.as_ref(), nft_addr.clone())?;

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    if nft_addr == config.ninja_nft_addr {
        config
            .ninja_distribution_schedule
            .push((start_date, end_date, amount));
    } else {
        config
            .scientist_distribution_schedule
            .push((start_date, end_date, amount));
    };

    guild_state.total_rewards_distributed += amount;

    store_guild_state(deps.storage, &guild_state, nft_addr)?;

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![("action", "update_config")]))
}

fn compute_reward(config: &Config, state: &mut GuildState, block_time: u64, nft_addr: String) {
    if state.total_stake_amount.is_zero() {
        state.last_distributed = block_time;
        return;
    };

    let mut distributed_amount = Uint128::zero();

    let distribution_schedule = if nft_addr == config.ninja_nft_addr {
        &config.ninja_distribution_schedule
    } else {
        &config.scientist_distribution_schedule
    };

    for s in distribution_schedule.iter() {
        if s.0 > block_time || s.1 < state.last_distributed {
            continue;
        }

        let passed_time =
            std::cmp::min(s.1, block_time) - std::cmp::max(s.0, state.last_distributed);

        let time = s.1 - s.0;
        let distribution_amount_per_second = Decimal::from_ratio(s.2, time);
        distributed_amount += distribution_amount_per_second * Uint128::from(passed_time as u128);
    }

    state.last_distributed = block_time;
    state.global_reward_index = state.global_reward_index
        + Decimal::from_ratio(distributed_amount, state.total_stake_amount);
}

fn compute_staker_reward(state: &GuildState, staker_info: &mut StakerInfo) -> StdResult<()> {
    let pending_rewards = (staker_info.stake_amount * state.global_reward_index)
        .checked_sub(staker_info.stake_amount * staker_info.reward_index)?;

    staker_info.reward_index = state.global_reward_index;
    staker_info.pending_rewards += pending_rewards;
    Ok(())
}

fn increase_stake_amount(state: &mut GuildState, staker_info: &mut StakerInfo, amount: Uint128) {
    state.total_stake_amount += amount;
    staker_info.stake_amount += amount;
}

fn decrease_stake_amount(state: &mut GuildState, staker_info: &mut StakerInfo, amount: Uint128) {
    state.total_stake_amount -= amount;
    staker_info.stake_amount -= amount;
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::State { block_time } => to_json_binary(&query_state(deps, block_time)?),
        QueryMsg::StakerInfo { staker } => to_json_binary(&query_staker_info(deps, staker)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    let resp = ConfigResponse {
        owner: config.owner.to_string(),
        scientist_nft_addr: config.scientist_nft_addr.to_string(),
        ninja_nft_addr: config.ninja_nft_addr.to_string(),
        qtum_addr: config.qtum_addr.to_string(),
        xqtum_addr: config.xqtum_addr.to_string(),
        nft_price: config.nft_price,
        ninja_distribution_schedule: config.ninja_distribution_schedule,
        scientist_distribution_schedule: config.scientist_distribution_schedule,
    };

    Ok(resp)
}

pub fn query_state(deps: Deps, block_time: Option<u64>) -> StdResult<StateResponse> {
    let mut ninja_state = NINJA_GUILD.load(deps.storage)?;
    let mut scientist_state = SCIENTIST_GUILD.load(deps.storage)?;

    if let Some(block_time) = block_time {
        let config = CONFIG.load(deps.storage)?;
        compute_reward(
            &config,
            &mut ninja_state,
            block_time,
            config.ninja_nft_addr.to_string(),
        );
        compute_reward(
            &config,
            &mut scientist_state,
            block_time,
            config.scientist_nft_addr.to_string(),
        );
    }

    Ok(StateResponse {
        ninja_total_staker: ninja_state.total_staker,
        ninja_total_stake_amount: ninja_state.total_stake_amount,
        ninja_last_distributed: ninja_state.last_distributed,
        ninja_global_reward_index: ninja_state.global_reward_index,
        scientist_total_staker: scientist_state.total_staker,
        scientist_total_stake_amount: scientist_state.total_stake_amount,
        scientist_last_distributed: scientist_state.last_distributed,
        scientist_global_reward_index: scientist_state.global_reward_index,
    })
}

pub fn query_staker_info(deps: Deps, staker: String) -> StdResult<StakerInfoResponse> {
    let staker = deps.api.addr_validate(&staker)?;

    let user_info: StakerInfo = user_staking(deps, staker.as_str())?;

    Ok(StakerInfoResponse {
        stake_amount: user_info.stake_amount,
        pending_rewards: user_info.pending_rewards,
        reward_claimed: user_info.reward_claimed,
        nft_addr: user_info.nft_addr,
        token_id: user_info.token_id,
    })
}
