use crate::Identifier;
use crate::abi::{ConfigOption, EventKind, Handle, Permission, Priority};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum MetaRequest
{
    Subscribe
    {
        kind: EventKind,
        custom: Option<Identifier>,
        priority: Priority,
        receive_canceled: bool,
        handler: Handle,
    },

    DefineEvent
    {
        id: Identifier,
    },

    PostEvent
    {
        id: Identifier,
        data: Vec<u8>,
    },

    RegisterCommand
    {
        name: String,
        permission: Permission,
        usage: String,
        handler: Handle,
    },

    ConfigValue
    {
        key: String,
    },

    Translate
    {
        key: String,
    },

    IsClient,

    IsLoaded
    {
        modid: String,
    },

    /// typed options shown in the mod menu, defining a known key again replaces it
    DefineConfigOptions
    {
        #[serde(with = "crate::wire::named")]
        options: Vec<ConfigOption>,
    },

    /// checked against the option kind and saved, the mod gets no ConfigChanged for its own change
    SetConfigValue
    {
        key: String,
        value: String,
    },

    /// api 1.4: the display name of a loaded mod, Minecraft for the vanilla namespace and empty for an unknown id
    ModName
    {
        modid: String,
    }, // api 1.7: wears the held stack down as using it would, a stack worn past its last use breaks
}
