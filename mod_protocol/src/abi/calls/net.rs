use crate::Identifier;
use crate::abi::{Handle, Target};
use crate::handshake::ChannelDirection;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum NetRequest
{
    RegisterChannel
    {
        name: String, version: String, direction: ChannelDirection, handler: Handle
    },

    Send
    {
        channel: Identifier, target: Target, data: Vec<u8>
    },

    CanSend
    {
        player: i32, channel: Identifier
    },

    /// api 1.54: a configuration task named <modid>:<name> every joining client with this mod runs, in load order,
    /// before play starts, registration only
    RegisterConfigTask
    {
        name: String, handler: Handle
    },
}
