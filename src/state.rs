use cosmwasm_schema::cw_serde;
use cosmwasm_std::{IbcEndpoint, Uint128};
use cw_storage_plus::{Item, Map};

/// static info on one channel that doesn't change
pub const CHANNEL_INFO: Map<&str, ChannelInfo> = Map::new("channel_info");

pub const ICQ_CHANNEL_INFO: Map<&str, String> = Map::new("icq_channel_info");

pub const ICQ_QUERY_IBC_CHANNEL: Map<u64, String> = Map::new("channel_info");

pub const LAST_IBC_CHANNEL_USED: Item<String> = Item::new("last_ibc_channel_used");

#[cw_serde]
#[derive(Default)]
pub struct ChannelState {
    pub outstanding: Uint128,
    pub total_sent: Uint128,
}

#[cw_serde]
pub struct ChannelInfo {
    /// id of this channel
    pub id: String,
    /// the remote channel/port we connect to
    pub counterparty_endpoint: IbcEndpoint,
    /// the connection this exists on (you can use to query client/consensus info)
    pub connection_id: String,
}