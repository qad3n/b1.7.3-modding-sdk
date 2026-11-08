use crate::Identifier;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// spells b173MOD in the mapSeed of the client Packet1Login, vanilla servers ignore the field
pub const LOADER_MAGIC: i64 = 0x6231_3733_4D4F_4400;
pub const LOADER_PROTOCOL: i8 = 1;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ModSide
{
    Both,
    Client,
    Server,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChannelDirection
{
    ToClient,
    ToServer,
    Both,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct ModEntry
{
    pub id: String,
    pub version: String,
    pub side: ModSide,
    pub network_version: String,
    pub required: bool,
    pub sha256: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct ChannelEntry
{
    pub id: Identifier,
    pub version: String,
    pub direction: ChannelDirection,
    pub required: bool,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Hello
{
    pub loader_version: String,
    pub mods: Vec<ModEntry>,
    pub channels: Vec<ChannelEntry>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, Default)]
pub struct RegistrySnapshot
{
    // registry name to identifier to numeric id
    pub registries: BTreeMap<String, BTreeMap<Identifier, i32>>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct ConfigSync
{
    // mod id to the toml text of its server scope values
    pub values: BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum DataSync
{
    Hash(String),
    Full(Vec<u8>),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Control
{
    Hello(Hello),
    HelloReply(Hello),
    Registry(RegistrySnapshot),
    Config(ConfigSync),
    Data(DataSync),
    // the data hash differed, the client asks for the full table
    DataRequest,
    Task(String, Vec<u8>),
    Ack(String),
    // the client could not apply the registry, each line names one unknown required entry
    Refused(Vec<String>),
    // channel indices in the order of the channel list, both sides map them before play
    Channels(Vec<Identifier>),
    Done,
}

pub mod acks
{
    pub const HELLO: &str = "hello";
    pub const REGISTRY: &str = "registry";
    pub const CONFIG: &str = "config";
    pub const DATA: &str = "data";
}
