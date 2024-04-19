use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw20::Cw20ReceiveMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub count: i32,
}

#[cw_serde]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    Increment {},
    Reset { count: i32 },
}

#[cw_serde]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    GetCount {},
}

// We define a custom struct for each query response
#[cw_serde]
pub struct CountResponse {
    pub count: i32,
}



#[cw_serde]
pub enum Cw20HookMsg {
    Mint {
        id: String,
        nft_address: Addr
    }, 
}

