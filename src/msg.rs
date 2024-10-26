use cosmwasm_schema::cw_serde;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    RegisterInterchainQueryChannel {
        chain_id: String,
        connection_id: String,
    },
    UnregisterInterchainQueryChannel {
        chain_id: String,
    }
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
    pub chain_id: String,
    pub addr: String,
    pub denom: String,
}