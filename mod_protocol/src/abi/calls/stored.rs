use crate::Identifier;
use crate::abi::{BlockPos, Holder, StackRef};
use crate::spec::AttachmentSpec;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum StoredRequest
{
    TileData
    {
        world: i32, pos: BlockPos
    },

    SetTileData
    {
        world: i32, pos: BlockPos, data: Vec<u8>
    },

    EntityData
    {
        world: i32, id: i32
    },

    SetEntityData
    {
        world: i32, id: i32, data: Vec<u8>
    },

    WorldData
    {
        key: String
    },

    SetWorldData
    {
        key: String, data: Vec<u8>
    },

    PlayerData
    {
        player: i32, key: String
    },

    SetPlayerData
    {
        player: i32, key: String, data: Vec<u8>
    },

    /// bytes the server keeps per entity and mirrors to every client tracking it
    SyncedData
    {
        world: i32, id: i32
    },

    SetSyncedData
    {
        world: i32, id: i32, data: Vec<u8>
    },

    /// abi 3: a data component on a stack in place, any mod reads any component, a mod writes only its own namespace
    StackComponent
    {
        at: StackRef, component: Identifier
    },

    SetStackComponent
    {
        at: StackRef, component: Identifier, data: Vec<u8>
    },

    /// api 1.33: bytes a mod keeps under a key outside any world, shared by every save of this game or server folder,
    /// answered with Bytes; keys are lowercase letters, digits and underscores
    SharedData
    {
        key: String
    },

    /// api 1.33: replaces a shared data key, empty bytes remove it
    SetSharedData
    {
        key: String, data: Vec<u8>
    },

    /// api 1.52: a named value this mod keeps on an entity, answered with Bytes, empty when unset; saved values go
    /// with the entity's save, transient ones end when it leaves its world
    EntityValue
    {
        world: i32, id: i32, key: String, saved: bool
    },

    /// api 1.52: sets or, with empty data, clears an entity value; a kept saved value of a player survives death
    SetEntityValue
    {
        world: i32, id: i32, key: String, saved: bool, kept: bool, data: Vec<u8>
    },

    /// api 2.0: names an attachment of this mod and says whether it is saved, synced and kept; while mods load
    Declare
    {
        name: String,
        #[serde(with = "crate::wire::named")]
        spec: AttachmentSpec,
    },

    /// api 2.0: the value of a declared attachment on a holder, answered with Stored, none when it was never set
    Fetch
    {
        holder: Holder, name: Identifier
    },

    /// api 2.0: sets a declared attachment on a holder, none clears it; a saved or synced one only where the world
    /// is owned
    Store
    {
        holder: Holder, name: Identifier, data: Option<Vec<u8>>
    },
}
