use cosmwasm_schema::cw_serde;
use cosmwasm_std::{IbcEndpoint};
use cw_storage_plus::{Item};

/// static info on one channel that doesn't change
pub const CHANNEL_INFO: Item<ChannelInfo> = Item::new("channel_info");
pub const ICQ_CHANNEL_INFO: Item<String> = Item::new("icq_connection_id");

#[cw_serde]
pub struct ChannelInfo {
    /// id of this channel
    pub id: String,
    /// the remote channel/port we connect to
    pub counterparty_endpoint: IbcEndpoint,
    /// the connection this exists on (you can use to query client/consensus info)
    pub connection_id: String,
}