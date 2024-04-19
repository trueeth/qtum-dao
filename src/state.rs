
use cosmwasm_schema::cw_serde;

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;

#[cw_serde]
pub struct DaoState {
    pub count: i32,
    pub owner: Addr,

    pub scientist_nft_addr: Addr,
    pub ninja_nft_addr: Addr,

    pub nft_price: Uint128,

    pub qtum_addr : Addr,

    pub scientist_holders: u64,
    pub ninja_holders: u64

}

pub const DAO_STATE: Item<DaoState> = Item::new("state");
