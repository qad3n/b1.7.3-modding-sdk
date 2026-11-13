mod calls;
mod config;
mod events;
mod guest;
mod host;
mod registry;
mod screen;
mod shared;
mod types;
mod views;
mod world;

pub use calls::
{
    ContentRequest, EntityRequest, HostCall, InventoryRequest, MetaRequest, NetRequest, PlayerRequest, RegistryRequest,
    ScreenRequest, SharedRequest, StoredRequest, ViewerRequest, WorldRequest, WorldgenRequest,
};
pub use config::{ConfigKind, ConfigOption, ConfigScope};
pub use events::{EVENT_NAMES, EventData, EventEnvelope, EventKind, Priority};
pub use guest::
{
    BlockCall, EffectCall, EntityCall, FeatureCall, GuestCall, GuestReply, ItemCall, Lifecycle, LootContext, TileCall,
    UseAction,
};
pub use host::HostReply;
pub use registry::{KindRow, MapTarget, RegistryKind};
pub use screen::
{
    BipedPart, DrawList, DrawOp, FaceOverlay, ItemModelSpec, ModelOp, OverlayOp, PlayerLayer, Pose, PoseInput,
    ScreenOp, ScreenSpec, SlotSpec,
};
pub use shared::
{
    ContainerSwap, EnergyDemand, EnergyLevel, EntityField, GameRuleValue, JobMatch, JobOutput, LootTarget,
    ModifierOperation, TankLevel, Worn, capability,
};
pub use types::
{
    AttachmentChange, BlockCtx, BlockInfo, BlockPos, ChunkInfo, EntityInfo, FluidBlock, FluidStack, Holder, ItemInfo,
    LogLevel, Permission, RayHit, Sender, StackData, StackRef, StateData, Target, attachmentEvent,
};
pub use views::{LootDrop, LootSource, LootView, MachineRecipeView, MachineSlot, MachineView, RecipeShape, RecipeView};
pub use world::
{
    BiomeSpec, DimensionSpec, FeatureInfo, Located, MusicSpec, SpawnEntry, StructurePiece, StructureSpec,
    StructureStart,
};

/// guest exports b173_alloc, b173_free, b173_dispatch and b173_abi; host imports live in module b173
pub const ABI_VERSION: u32 = 4;

/// abi 4 moved declarations and stored data to the named form and dropped doubled calls, so older guests do not load;
/// from here variants only append and named structs only gain defaulted fields
pub const ABI_MIN_VERSION: u32 = 4;

pub const IMPORT_MODULE: &str = "b173";

pub mod exports
{
    pub const MEMORY: &str = "memory";
    pub const ALLOC: &str = "b173_alloc";
    pub const FREE: &str = "b173_free";
    pub const DISPATCH: &str = "b173_dispatch";
    pub const ABI: &str = "b173_abi";
}

pub mod imports
{
    pub const CALL: &str = "host_call";
    pub const LOG: &str = "host_log";
}

pub type Handle = u32;

/// a returned buffer is packed as pointer in the high word and length in the low word
pub fn packBuffer(pointer: u32, length: u32) -> u64 { ((pointer as u64) << 32) | length as u64 }

pub fn unpackBuffer(packed: u64) -> (u32, u32) { ((packed >> 32) as u32, packed as u32) }

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn everyEventKindHasOneName()
    {
        for (kind, name) in EVENT_NAMES
        {
            assert_eq!(kind.name(), name);
        }

        let mut names: Vec<&str> = EVENT_NAMES.iter().map(|row| row.1).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), EVENT_NAMES.len());
    }

    #[test]
    fn callsRoundTrip()
    {
        let call: GuestCall =
            GuestCall::Block { handle: 2, call: BlockCall::NeighborChanged { ctx: BlockCtx::default(), neighbor: 4 } };
        let back: GuestCall = crate::wire::decode(&crate::wire::encode(&call)).expect("decodes");
        assert_eq!(call, back);
    }
}
