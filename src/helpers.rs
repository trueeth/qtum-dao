use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    to_json_binary, Addr, CosmosMsg, CustomQuery, Querier, QuerierWrapper, StdResult, WasmMsg, WasmQuery,
};

use crate::msg::{CountResponse, ExecuteMsg, QueryMsg};

/// CwTemplateContract is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[cw_serde]
pub struct CwTemplateContract(pub Addr);

impl CwTemplateContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
        let msg = to_json_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
        }
        .into())
    }

    /// Get Count
    pub fn count<Q, T, CQ>(&self, querier: &Q) -> StdResult<CountResponse>
    where
        Q: Querier,
        T: Into<String>,
        CQ: CustomQuery,
    {
        let msg = QueryMsg::GetCount {};
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_json_binary(&msg)?,
        }
        .into();
        let res: CountResponse = QuerierWrapper::<CQ>::new(querier).query(&query)?;
        Ok(res)
    }
}
