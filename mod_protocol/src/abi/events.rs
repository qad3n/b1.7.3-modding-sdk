use super::{BlockPos, EntityInfo, StackData};
use crate::Identifier;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Priority
{
    Highest,
    High,
    Normal,
    Low,
    Lowest,
    Monitor,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EventKind
{
    Setup,
    ClientSetup,
    ServerSetup,
    LoadComplete,
    Shutdown,
    ServerStarting,
    ServerStarted,
    ServerStopping,
    TickPre,
    TickPost,
    WorldLoad,
    WorldSave,
    WorldUnload,
    ChunkLoad,
    ChunkUnload,
    ChunkGenerate,
    ChunkPopulate,
    BlockPlace,
    BlockBreak,
    BlockHarvestDrops,
    BlockActivate,
    NeighborChange,
    ExplosionAffect,
    ItemUse,
    ItemUseFirst,
    ItemUseOnBlock,
    ItemDestroyed,
    ItemPickup,
    ItemToss,
    EntityJoinWorld,
    EntityTick,
    LivingAttack,
    LivingHurt,
    LivingDeath,
    LivingDrops,
    LivingSpawnCheck,
    PlayerLogin,
    PlayerLogout,
    PlayerRespawn,
    PlayerChangeDimension,
    PlayerSleep,
    PlayerChat,
    PlayerInteract,
    PlayerBreakSpeed,
    PlayerCanHarvest,
    ItemCrafted,
    ItemSmelted,
    BucketFill,
    RenderWorldLast,
    RenderHud,
    RenderBlockHighlight,
    ScreenOpen,
    KeyInput,
    ClientTick,
    Custom,
    /// api 1.52: an entity left its world for good or for now, by death, despawn, chunk unload or travel
    EntityRemove,
    /// api 2.0: a lightning bolt is about to burn an entity, canceled the entity is spared
    LightningStrike,
}

/// catalog name of each event kind, the hooks.toml rows use the same names
pub const EVENT_NAMES: [(EventKind, &str); 57] = [
    (EventKind::Setup, "setup"),
    (EventKind::ClientSetup, "client_setup"),
    (EventKind::ServerSetup, "server_setup"),
    (EventKind::LoadComplete, "load_complete"),
    (EventKind::Shutdown, "shutdown"),
    (EventKind::ServerStarting, "server_starting"),
    (EventKind::ServerStarted, "server_started"),
    (EventKind::ServerStopping, "server_stopping"),
    (EventKind::TickPre, "tick_pre"),
    (EventKind::TickPost, "tick_post"),
    (EventKind::WorldLoad, "world_load"),
    (EventKind::WorldSave, "world_save"),
    (EventKind::WorldUnload, "world_unload"),
    (EventKind::ChunkLoad, "chunk_load"),
    (EventKind::ChunkUnload, "chunk_unload"),
    (EventKind::ChunkGenerate, "chunk_generate"),
    (EventKind::ChunkPopulate, "chunk_populate"),
    (EventKind::BlockPlace, "block_place"),
    (EventKind::BlockBreak, "block_break"),
    (EventKind::BlockHarvestDrops, "block_harvest_drops"),
    (EventKind::BlockActivate, "block_activate"),
    (EventKind::NeighborChange, "neighbor_change"),
    (EventKind::ExplosionAffect, "explosion_affect"),
    (EventKind::ItemUse, "item_use"),
    (EventKind::ItemUseFirst, "item_use_first"),
    (EventKind::ItemUseOnBlock, "item_use_on_block"),
    (EventKind::ItemDestroyed, "item_destroyed"),
    (EventKind::ItemPickup, "item_pickup"),
    (EventKind::ItemToss, "item_toss"),
    (EventKind::EntityJoinWorld, "entity_join_world"),
    (EventKind::EntityTick, "entity_tick"),
    (EventKind::LivingAttack, "living_attack"),
    (EventKind::LivingHurt, "living_hurt"),
    (EventKind::LivingDeath, "living_death"),
    (EventKind::LivingDrops, "living_drops"),
    (EventKind::LivingSpawnCheck, "living_spawn_check"),
    (EventKind::PlayerLogin, "player_login"),
    (EventKind::PlayerLogout, "player_logout"),
    (EventKind::PlayerRespawn, "player_respawn"),
    (EventKind::PlayerChangeDimension, "player_change_dimension"),
    (EventKind::PlayerSleep, "player_sleep"),
    (EventKind::PlayerChat, "player_chat"),
    (EventKind::PlayerInteract, "player_interact"),
    (EventKind::PlayerBreakSpeed, "player_break_speed"),
    (EventKind::PlayerCanHarvest, "player_can_harvest"),
    (EventKind::ItemCrafted, "item_crafted"),
    (EventKind::ItemSmelted, "item_smelted"),
    (EventKind::BucketFill, "bucket_fill"),
    (EventKind::RenderWorldLast, "render_world_last"),
    (EventKind::RenderHud, "render_hud"),
    (EventKind::RenderBlockHighlight, "render_block_highlight"),
    (EventKind::ScreenOpen, "screen_open"),
    (EventKind::KeyInput, "key_input"),
    (EventKind::ClientTick, "client_tick"),
    (EventKind::Custom, "custom"),
    (EventKind::EntityRemove, "entity_remove"),
    (EventKind::LightningStrike, "lightning_strike"),
];

impl EventKind
{
    pub fn name(self) -> &'static str
    {
        EVENT_NAMES.iter().find(|row| row.0 == self).map(|row| row.1).expect("every event kind has a name row")
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum EventData
{
    Empty,
    World
    {
        world: i32,
    },

    Chunk
    {
        world: i32,
        x: i32,
        z: i32,
    },

    Block
    {
        world: i32,
        pos: BlockPos,
        block: i32,
        meta: i32,
        player: Option<i32>,
    },

    Neighbor
    {
        world: i32,
        pos: BlockPos,
        block: i32,
        neighbor: i32,
    },

    Drops
    {
        world: i32,
        pos: BlockPos,
        block: i32,
        meta: i32,
        drops: Vec<StackData>,
    },

    ItemUse
    {
        world: i32,
        player: i32,
        stack: Option<StackData>,
        pos: Option<BlockPos>,
        face: i32,
    },

    Item
    {
        world: i32,
        player: i32,
        stack: StackData,
    },

    Entity
    {
        entity: EntityInfo,
    },

    Attack
    {
        entity: EntityInfo,
        source: Option<i32>,
        amount: i32,
    },

    Death
    {
        entity: EntityInfo,
        source: Option<i32>,
        drops: Vec<StackData>,
    },

    Spawn
    {
        entity: EntityInfo,
        pos: BlockPos,
    },

    Player
    {
        player: i32,
        world: i32,
        name: String,
    },

    Dimension
    {
        player: i32,
        from: i32,
        to: i32,
    },

    Chat
    {
        player: i32,
        message: String,
    },

    BreakSpeed
    {
        player: i32,
        world: i32,
        pos: BlockPos,
        block: i32,
        speed: f32,
    },

    Harvest
    {
        player: i32,
        block: i32,
        meta: i32,
    },

    Render
    {
        partial: f32,
        width: i32,
        height: i32,
    },

    Screen
    {
        name: String,
    },

    Key
    {
        key: i32,
        pressed: bool,
    },

    Custom
    {
        event: Identifier,
        data: Vec<u8>,
    },
}

/// listeners may cancel or set a result, monitor listeners only read
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct EventEnvelope
{
    pub kind: EventKind,
    pub canceled: bool,
    pub result: Option<i32>,
    pub data: EventData,
}
