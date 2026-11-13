mod content;
mod entity;
mod inventory;
mod media;
mod meta;
mod net;
mod player;
mod registry;
mod screen;
mod shared;
mod stored;
mod viewer;
mod world;
mod worldgen;

pub use content::ContentRequest;
pub use entity::EntityRequest;
pub use inventory::InventoryRequest;
pub use media::{MediaInfo, MediaLimit, MediaRequest, MediaStatus, Song};
pub use meta::MetaRequest;
pub use net::NetRequest;
pub use player::PlayerRequest;
pub use registry::RegistryRequest;
pub use screen::ScreenRequest;
use serde::{Deserialize, Serialize};
pub use shared::SharedRequest;
pub use stored::StoredRequest;
pub use viewer::ViewerRequest;
pub use world::WorldRequest;
pub use worldgen::WorldgenRequest;

// one row per group: the enum of every request a mod can make, and the conversion that lets a call site name only
// its group
macro_rules! host_calls
{
    ($($group:ident($call:ident),)*) =>
    {
        #[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
        pub enum HostCall
        {
            $($group($call),)*
        }

        $(impl From<$call> for HostCall
        {
            fn from(call: $call) -> HostCall { HostCall::$group(call) }
        })*
    };
}

host_calls!
{
    Content(ContentRequest),
    Registry(RegistryRequest),
    Worldgen(WorldgenRequest),
    World(WorldRequest),
    Entity(EntityRequest),
    Player(PlayerRequest),
    Inventory(InventoryRequest),
    Stored(StoredRequest),
    Shared(SharedRequest),
    Net(NetRequest),
    Screen(ScreenRequest),
    Viewer(ViewerRequest),
    Meta(MetaRequest),
    Media(MediaRequest),
}
