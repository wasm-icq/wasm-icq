use cosmwasm_std::{Binary, Deps, DepsMut, entry_point, Env, IbcMsg, MessageInfo, Response, StdError, StdResult, to_json_binary};
use cw2::set_contract_version;
use neutron_sdk::bindings::msg::NeutronMsg;
use neutron_sdk::bindings::query::NeutronQuery;
use neutron_sdk::interchain_queries::{check_query_type, get_registered_query, query_kv_result};
use neutron_sdk::interchain_queries::types::QueryType;
use neutron_sdk::interchain_queries::v047::queries::{BalanceResponse, query_balance};
use neutron_sdk::interchain_queries::v047::types::Balances;
use neutron_sdk::NeutronResult;
use neutron_sdk::sudo::msg::SudoMsg;
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{CHANNEL_INFO, ICQ_CHANNEL_INFO};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:wasm-icq";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    deps.api.debug("WASMDEBUG: instantiate");
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<NeutronMsg>, ContractError> {
    match msg {
        ExecuteMsg::RegisterInterchainQueryChannel { connection_id } => {
            deps.api.debug("WASMDEBUG: RegisterInterchainQueryChannel");
            ICQ_CHANNEL_INFO.save(deps.storage, &connection_id)?;
            Ok(Response::new())
        },
        ExecuteMsg::RemoveInterchainQuery { query_id } => {
            let remove_msg = NeutronMsg::remove_interchain_query(query_id);
            Ok(Response::new().add_message(remove_msg))
        },
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: QueryMsg) -> NeutronResult<Binary> {
    match msg {
        QueryMsg::Balance { query_id } => Ok(to_json_binary(&query_balance(deps, env, query_id)?)?),
    }
}

#[cfg(test)]
mod tests {}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}

#[entry_point]
pub fn sudo(deps: DepsMut<NeutronQuery>, env: Env, msg: SudoMsg) -> NeutronResult<Response> {
    match msg {
        SudoMsg::KVQueryResult { query_id } => sudo_kv_query_result(deps, env, query_id),
        _ => Ok(Response::default()),
    }
}

/// sudo_kv_query_result is the contract's callback for KV query results. Note that only the query
/// id is provided, so you need to read the query result from the state.
pub fn sudo_kv_query_result(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    query_id: u64,
) -> NeutronResult<Response> {
    deps.api.debug(
        format!(
            "WASMDEBUG: sudo_kv_query_result received; query_id: {:?}",
            query_id,
        )
            .as_str(),
    );

    let channel_id: String = get_channel_id(deps.as_ref())?;

    let balance_response: BalanceResponse = get_balances_result(deps.as_ref(), query_id)?;

    // timeout is in nanoseconds
    let timeout = env.block.time.plus_seconds(120);

    // TODO review adding message to list of called messages
    // let remove_msg = NeutronMsg::remove_interchain_query(query_id);

    // prepare ibc message
    let ibc_msg = IbcMsg::SendPacket {
        channel_id: channel_id.to_string(),
        data: to_json_binary(&balance_response)?,
        timeout: timeout.into(),
    };

    Ok(Response::new()
        .add_attribute("method", "send_query_balance_response")
        .add_attribute("channel", channel_id.to_string())
        .add_message(ibc_msg))
}

fn get_channel_id(deps: Deps<NeutronQuery>,) -> StdResult<String> {
    match CHANNEL_INFO.may_load(deps.storage)? {
        Some(channel_info) => Ok(channel_info.id), // Return the item if it's loaded
        None => Err(StdError::generic_err("Channel to ICQ module is not setup")),
    }
}

pub fn get_balances_result(
    deps: Deps<NeutronQuery>,
    registered_query_id: u64,
) -> NeutronResult<BalanceResponse> {
    let registered_query = get_registered_query(deps, registered_query_id)?;

    check_query_type(registered_query.registered_query.query_type, QueryType::KV)?;

    let balances: Balances = query_kv_result(deps, registered_query_id)?;

    Ok(BalanceResponse {
        last_submitted_local_height: registered_query
            .registered_query
            .last_submitted_result_local_height,
        balances,
    })
}
