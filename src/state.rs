use cosmwasm_schema::cw_serde;
use cosmwasm_std::{IbcEndpoint, Uint128};
use cw_storage_plus::Map;

/// static info on one channel that doesn't change
pub const CHANNEL_INFO: Map<&str, ChannelInfo> = Map::new("channel_info");

pub const ICQ_CHANNEL_INFO: Map<&str, String> = Map::new("icq_channel_info");

pub const KV_CALLBACK_STATS: Map<u64, u64> = Map::new("kv_callback_stats");

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