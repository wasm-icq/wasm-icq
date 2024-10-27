use cosmwasm_schema::cw_serde;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    RegisterInterchainQueryChannel {
        connection_id: String,
    },
    RemoveInterchainQuery {
        query_id: u64,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Balance { query_id: u64 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MigrateMsg {}

#[cw_serde]
pub struct IbcRegisterBalanceQuery {
    pub addr: String,
    pub denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Describes response structure for **RegisterInterchainQuery** msg.
pub struct MsgRegisterInterchainQueryResponse {
    /// **id** is an identifier of newly registered interchain query.
    pub id: u64,
}