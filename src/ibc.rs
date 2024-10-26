use cosmwasm_schema::cw_serde;
use cosmwasm_std::{CosmosMsg, Deps, DepsMut, Env, from_json, IbcBasicResponse, IbcChannel, IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcChannelOpenResponse, IbcOrder, IbcPacket, IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse, Never, Reply, Response, StdError, StdResult, SubMsg, SubMsgResult};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use neutron_sdk::bindings::msg::NeutronMsg;
use neutron_sdk::interchain_queries::v045::{new_register_balances_query_msg};

use crate::ack::make_ack_fail;
use crate::error::ContractError;
use crate::state::{CHANNEL_INFO, ChannelInfo, ICQ_CHANNEL_INFO};

pub const IBC_VERSION: &str = "icq-1";
pub const ICQ_UPDATE_PERIOD: u64 = 1_000_000;

/// Handles the `OpenInit` and `OpenTry` parts of the IBC handshake.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> Result<IbcChannelOpenResponse, ContractError> {
    validate_order_and_version(msg.channel(), msg.counterparty_version())?;
    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_connect(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> Result<IbcBasicResponse, ContractError> {
    validate_order_and_version(msg.channel(), msg.counterparty_version())?;

    let channel: IbcChannel = msg.into();
    let info = ChannelInfo {
        id: channel.endpoint.channel_id,
        counterparty_endpoint: channel.counterparty_endpoint,
        connection_id: channel.connection_id,
    };
    CHANNEL_INFO.save(deps.storage, &info.id, &info)?;

    Ok(IbcBasicResponse::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_close(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelCloseMsg,
) -> Result<IbcBasicResponse, ContractError> {
    let channel = msg.channel().endpoint.channel_id.clone();
    // Reset the state for the channel.
    CHANNEL_INFO.remove(deps.storage, &channel);
    Ok(IbcBasicResponse::new()
        .add_attribute("method", "ibc_channel_close")
        .add_attribute("channel", channel))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_receive(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse<NeutronMsg>, Never> {
    let packet = msg.packet;

    do_ibc_packet_receive(deps, &packet).or_else(|err| {
        Ok(IbcReceiveResponse::new()
            .add_attribute("action", "receive")
            .add_attribute("success", "false")
            .add_attribute("error", err.to_string())
        )
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_ack(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, ContractError> {
    Ok(IbcBasicResponse::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketTimeoutMsg,
) -> Result<IbcBasicResponse, ContractError> {
    // As with ack above, nothing to do here. If we cared about
    // keeping track of state between the two chains then we'd want to
    // respond to this likely as it means that the packet in question
    // isn't going anywhere.
    Ok(IbcBasicResponse::new().add_attribute("method", "ibc_packet_timeout"))
}

fn validate_order_and_version(
    channel: &IbcChannel,
    counterparty_version: Option<&str>,
) -> Result<(), ContractError> {
    // We expect an unordered channel here. Ordered channels have the
    // property that if a message is lost the entire channel will stop
    // working until you start it again.
    if channel.order != IbcOrder::Unordered {
        return Err(ContractError::OnlyOrderedChannel {});
    }

    if channel.version != IBC_VERSION {
        return Err(ContractError::InvalidIbcVersion {
            actual: channel.version.to_string(),
            expected: IBC_VERSION.to_string(),
        });
    }

    // Make sure that we're talking with a counterparty who speaks the
    // same "protocol" as us.
    //
    // For a connection between chain A and chain B being established
    // by chain A, chain B knows counterparty information during
    // `OpenTry` and chain A knows counterparty information during
    // `OpenAck`. We verify it when we have it but when we don't it's
    // alright.
    if let Some(counterparty_version) = counterparty_version {
        if counterparty_version != IBC_VERSION {
            return Err(ContractError::InvalidIbcVersion {
                actual: counterparty_version.to_string(),
                expected: IBC_VERSION.to_string(),
            });
        }
    }

    Ok(())
}

fn do_ibc_packet_receive(
    deps: DepsMut,
    packet: &IbcPacket,
) -> Result<IbcReceiveResponse<NeutronMsg>, ContractError> {
    let query_data: IbcRegisterBalanceQuery = from_json(&packet.data)?;
    let connection_id: String = get_icq_channel_id(deps.as_ref(), query_data.chain_id)?;

    let register_balances_query_msg = new_register_balances_query_msg(
        connection_id,
        query_data.addr,
        vec![query_data.denom],
        ICQ_UPDATE_PERIOD,
    )?;

    Ok(IbcReceiveResponse::new()
        .add_submessage(SubMsg::reply_on_success(register_balances_query_msg, ICQ_CREATED_RECEIVE_ID))
        .add_attribute("method", "ibc_packet_ack")
        .add_attribute("sequence", packet.sequence.to_string())
    )
}

fn get_icq_channel_id(deps: Deps, chain_id: String) -> StdResult<String> {
    match ICQ_CHANNEL_INFO.may_load(deps.storage, &chain_id)? {
        Some(channel_id) => Ok(channel_id), // Return the item if it's loaded
        None => Err(StdError::generic_err("Channel to ICQ module is not setup")),
    }
}

#[cw_serde]
pub struct IbcRegisterBalanceQuery {
    chain_id: String,
    addr: String,
    denom: String,
}

const ICQ_CREATED_RECEIVE_ID: u64 = 1337;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        ICQ_CREATED_RECEIVE_ID => match reply.result {
            SubMsgResult::Ok(_) => {
                Ok(Response::new().add_attribute("method", "contract_paid"))
            },
            SubMsgResult::Err(err) => {
                Ok(Response::new().set_data(make_ack_fail(err)))
            }
        },
        _ => Err(ContractError::UnknownReplyId { id: reply.id }),
    }
}