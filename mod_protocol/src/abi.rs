use crate::Identifier;
use crate::handshake::ChannelDirection;
use crate::spec::{BlockSpec, EntitySpec, ItemSpec, RecipeSpec, TileEntitySpec, WorldgenSpec};
use serde::{Deserialize, Serialize};

// guest exports b173_alloc, b173_free, b173_dispatch and b173_abi; host imports live in module b173
pub const ABI_VERSION: u32 = 1;
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

// a returned buffer is packed as pointer in the high word and length in the low word
pub fn packBuffer(pointer: u32, length: u32) -> u64 { ((pointer as u64) << 32) | length as u64 }

pub fn unpackBuffer(packed: u64) -> (u32, u32) { ((packed >> 32) as u32, packed as u32) }

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct BlockPos
{
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos
{
    pub const fn new(x: i32, y: i32, z: i32) -> BlockPos { BlockPos { x, y, z } }

    pub fn offset(self, dx: i32, dy: i32, dz: i32) -> BlockPos { BlockPos::new(self.x + dx, self.y + dy, self.z + dz) }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct StackData
{
    pub id: i32,
    pub count: i32,
    pub damage: i32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct EntityInfo
{
    pub id: i32,
    pub world: i32,
    pub kind: Option<Identifier>,
    pub pos: [f64; 3],
    pub motion: [f64; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub health: Option<i32>,
    pub player: Option<String>,
    pub alive: bool,
}

// every block callback carries where it runs and whether this side owns the world
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug, Default)]
pub struct BlockCtx
{
    pub world: i32,
    pub pos: BlockPos,
    pub block: i32,
    pub meta: i32,
    pub authority: bool,
}

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

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Permission
{
    Anyone,
    Op,
    Console,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum Target
{
    Server,
    Player(i32),
    All,
    Dimension(i32),
    Tracking(i32, BlockPos),
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum LogLevel
{
    Info,
    Warn,
    Error,
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
}

// catalog name of each event kind, the hooks.toml rows use the same names
pub const EVENT_NAMES: [(EventKind, &str); 55] = [
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

// listeners may cancel or set a result, monitor listeners only read
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct EventEnvelope
{
    pub kind: EventKind,
    pub canceled: bool,
    pub result: Option<i32>,
    pub data: EventData,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum DrawOp
{
    Rect
    {
        x: i32, y: i32, w: i32, h: i32, color: u32
    },
    Gradient
    {
        x: i32, y: i32, w: i32, h: i32, top: u32, bottom: u32
    },
    Text
    {
        x: i32, y: i32, text: String, color: u32, shadow: bool, centered: bool
    },
    Texture
    {
        texture: String, x: i32, y: i32, u: i32, v: i32, w: i32, h: i32
    },
    Item
    {
        x: i32, y: i32, stack: StackData
    },
    Button
    {
        id: i32, x: i32, y: i32, w: i32, h: i32, label: String, enabled: bool
    },
    Particle
    {
        name: String, pos: [f64; 3], motion: [f64; 3]
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct DrawList
{
    pub ops: Vec<DrawOp>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct SlotSpec
{
    pub index: i32,
    pub x: i32,
    pub y: i32,
    pub output: bool,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct ScreenSpec
{
    pub width: i32,
    pub height: i32,
    pub background: Option<String>,
    pub title: String,
    // a container screen syncs these tile entity slots plus the player inventory through vanilla slot packets
    pub slots: Vec<SlotSpec>,
    pub player_inventory: Option<[i32; 2]>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum ScreenOp
{
    Init
    {
        width: i32,
        height: i32,
        pos: Option<BlockPos>,
        data: Vec<u8>,
    },
    Draw
    {
        mouse: [i32; 2],
        partial: f32,
    },
    Button
    {
        id: i32,
    },
    Mouse
    {
        x: i32,
        y: i32,
        button: i32,
    },
    Key
    {
        character: u32,
        key: i32,
    },
    Tick,
    Close,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum BlockCall
{
    Activated
    {
        ctx: BlockCtx, player: i32
    },
    Clicked
    {
        ctx: BlockCtx, player: i32
    },
    Added
    {
        ctx: BlockCtx
    },
    Removed
    {
        ctx: BlockCtx
    },
    Placed
    {
        ctx: BlockCtx, face: i32
    },
    PlacedBy
    {
        ctx: BlockCtx, entity: i32
    },
    NeighborChanged
    {
        ctx: BlockCtx, neighbor: i32
    },
    UpdateTick
    {
        ctx: BlockCtx
    },
    RandomDisplayTick
    {
        ctx: BlockCtx
    },
    EntityCollided
    {
        ctx: BlockCtx, entity: i32
    },
    EntityWalking
    {
        ctx: BlockCtx, entity: i32
    },
    DestroyedByPlayer
    {
        ctx: BlockCtx
    },
    DestroyedByExplosion
    {
        ctx: BlockCtx
    },
    Harvest
    {
        ctx: BlockCtx, player: i32
    },
    CanPlaceAt
    {
        ctx: BlockCtx
    },
    CanStay
    {
        ctx: BlockCtx
    },
    PoweringTo
    {
        ctx: BlockCtx, direction: i32
    },
    IndirectlyPoweringTo
    {
        ctx: BlockCtx, direction: i32
    },
    CanProvidePower
    {
        block: i32
    },
    Texture
    {
        face: i32, meta: i32
    },
    ColorMultiplier
    {
        ctx: BlockCtx
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum ItemCall
{
    RightClick
    {
        world: i32, player: i32, stack: StackData
    },
    UseOnBlock
    {
        world: i32, player: i32, stack: StackData, pos: BlockPos, face: i32
    },
    HitEntity
    {
        stack: StackData, target: i32, attacker: i32
    },
    InteractEntity
    {
        stack: StackData, target: i32, player: i32
    },
    BlockDestroyed
    {
        stack: StackData, block: i32, pos: BlockPos, player: i32
    },
    Icon
    {
        damage: i32
    },
    StrVsBlock
    {
        stack: StackData, block: i32
    },
    CanHarvest
    {
        block: i32
    },
    Crafted
    {
        stack: StackData, player: i32
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum EntityCall
{
    Tick
    {
        entity: EntityInfo
    },
    LivingUpdate
    {
        entity: EntityInfo
    },
    Interact
    {
        entity: EntityInfo, player: i32
    },
    Hurt
    {
        entity: EntityInfo, source: Option<i32>, amount: i32
    },
    Death
    {
        entity: EntityInfo, source: Option<i32>
    },
    CanSpawn
    {
        entity: EntityInfo
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum TileCall
{
    Tick
    {
        ctx: BlockCtx
    },
    Removed
    {
        ctx: BlockCtx
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Sender
{
    pub player: Option<i32>,
    pub name: String,
    pub world: i32,
    pub op: bool,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum Lifecycle
{
    // carries the mod id so the guest knows its namespace
    Register(String),
    InterMod,
    // numeric ids changed after a world load or a server join, cached ids must be resolved again
    Remapped,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ChunkInfo
{
    pub world: i32,
    pub x: i32,
    pub z: i32,
    pub seed: i64,
}

// host to guest
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum GuestCall
{
    Lifecycle(Lifecycle),
    Block
    {
        handle: Handle,
        call: BlockCall,
    },
    Item
    {
        handle: Handle,
        call: ItemCall,
    },
    Entity
    {
        handle: Handle,
        call: EntityCall,
    },
    Tile
    {
        handle: Handle,
        call: TileCall,
    },
    Event
    {
        handler: Handle,
        event: EventEnvelope,
    },
    Command
    {
        handler: Handle,
        sender: Sender,
        args: Vec<String>,
    },
    Key
    {
        handler: Handle,
        pressed: bool,
    },
    Channel
    {
        handler: Handle,
        player: i32,
        data: Vec<u8>,
    },
    Screen
    {
        handler: Handle,
        op: ScreenOp,
    },
    Hud
    {
        handler: Handle,
        partial: f32,
        width: i32,
        height: i32,
    },
    Service
    {
        handler: Handle,
        method: String,
        args: Vec<u8>,
    },
    Lookup
    {
        handler: Handle,
        world: i32,
        pos: BlockPos,
        side: i32,
        method: String,
        args: Vec<u8>,
    },
    InterMod
    {
        from: String,
        kind: String,
        data: Vec<u8>,
    },
    Generate
    {
        handler: Handle,
        chunk: ChunkInfo,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum GuestReply
{
    Unit,
    Bool(bool),
    Int(i32),
    Float(f32),
    Bytes(Vec<u8>),
    Stack(Option<StackData>),
    Use
    {
        handled: bool,
        stack: Option<StackData>,
    },
    Event(EventEnvelope),
    Draw(DrawList),
    Error(String),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ConfigEntry
{
    pub key: String,
    pub default: String,
    pub comment: String,
    // client values stay local, common values load on both sides, server values sync to clients
    pub scope: ConfigScope,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConfigScope
{
    Client,
    Common,
    Server,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum RegistryKind
{
    Block,
    Item,
    Entity,
    TileEntity,
    Recipe,
    Biome,
    Dimension,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct BiomeSpec
{
    pub name: String,
    pub color: u32,
    pub temperature: f32,
    pub rainfall: f32,
    pub top_block: Option<Identifier>,
    pub filler_block: Option<Identifier>,
    pub weight: i32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct DimensionSpec
{
    pub name: String,
    pub sky_color: u32,
    pub fog_color: u32,
    pub has_sky: bool,
    pub fill_block: Option<Identifier>,
    pub ground_height: i32,
    pub portal_block: Option<Identifier>,
    pub portal_frame: Option<Identifier>,
    pub biome: Option<Identifier>,
    // a generator handle routes chunk filling into the mod, absent means flat terrain from the spec
    pub generator: Option<Handle>,
}

// guest to host
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum HostCall
{
    RegisterBlock
    {
        name: String,
        spec: BlockSpec,
    },
    RegisterItem
    {
        name: String,
        spec: ItemSpec,
    },
    RegisterEntity
    {
        name: String,
        spec: EntitySpec,
    },
    RegisterTileEntity
    {
        name: String,
        spec: TileEntitySpec,
    },
    RegisterRecipe
    {
        name: String,
        spec: RecipeSpec,
    },
    RemoveRecipe
    {
        id: Identifier,
    },
    ReplaceBlock
    {
        id: Identifier,
        spec: BlockSpec,
    },
    AddTag
    {
        registry: RegistryKind,
        tag: Identifier,
        values: Vec<Identifier>,
    },
    TagValues
    {
        registry: RegistryKind,
        tag: Identifier,
    },
    RegisterWorldgen
    {
        name: String,
        spec: WorldgenSpec,
    },
    RegisterGenerator
    {
        dimension: Identifier,
        handler: Handle,
    },
    RegisterBiome
    {
        name: String,
        spec: BiomeSpec,
    },
    RegisterDimension
    {
        name: String,
        spec: DimensionSpec,
    },
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
    RegisterKeybind
    {
        name: String,
        key: i32,
        handler: Handle,
    },
    RegisterChannel
    {
        name: String,
        version: String,
        direction: ChannelDirection,
        handler: Handle,
    },
    RegisterScreen
    {
        name: String,
        spec: ScreenSpec,
        handler: Handle,
    },
    RegisterHud
    {
        handler: Handle,
    },
    ProvideService
    {
        name: Identifier,
        version: u32,
        handler: Handle,
    },
    CallService
    {
        name: Identifier,
        version: u32,
        method: String,
        args: Vec<u8>,
    },
    HasService
    {
        name: Identifier,
        version: u32,
    },
    RegisterLookup
    {
        api: Identifier,
        block: Identifier,
        handler: Handle,
    },
    CallLookup
    {
        api: Identifier,
        world: i32,
        pos: BlockPos,
        side: i32,
        method: String,
        args: Vec<u8>,
    },
    SendInterMod
    {
        to: String,
        kind: String,
        data: Vec<u8>,
    },
    DefineConfig
    {
        entries: Vec<ConfigEntry>,
    },
    ConfigValue
    {
        key: String,
    },
    Translate
    {
        key: String,
    },
    Resolve
    {
        registry: RegistryKind,
        id: Identifier,
    },
    NameOf
    {
        registry: RegistryKind,
        numeric: i32,
    },
    IsAuthority
    {
        world: i32,
    },
    IsClient,
    IsLoaded
    {
        modid: String,
    },
    GetBlock
    {
        world: i32,
        pos: BlockPos,
    },
    GetMeta
    {
        world: i32,
        pos: BlockPos,
    },
    SetBlock
    {
        world: i32,
        pos: BlockPos,
        block: i32,
        meta: i32,
        notify: bool,
    },
    SetMeta
    {
        world: i32,
        pos: BlockPos,
        meta: i32,
    },
    NotifyNeighbors
    {
        world: i32,
        pos: BlockPos,
        block: i32,
    },
    ScheduleTick
    {
        world: i32,
        pos: BlockPos,
        block: i32,
        delay: i32,
    },
    MarkForRender
    {
        world: i32,
        pos: BlockPos,
    },
    Light
    {
        world: i32,
        pos: BlockPos,
    },
    IsPowered
    {
        world: i32,
        pos: BlockPos,
    },
    WorldTime
    {
        world: i32,
    },
    WorldRandom
    {
        world: i32,
        bound: i32,
    },
    TileData
    {
        world: i32,
        pos: BlockPos,
    },
    SetTileData
    {
        world: i32,
        pos: BlockPos,
        data: Vec<u8>,
    },
    TileSlot
    {
        world: i32,
        pos: BlockPos,
        slot: i32,
    },
    SetTileSlot
    {
        world: i32,
        pos: BlockPos,
        slot: i32,
        stack: Option<StackData>,
    },
    SpawnEntity
    {
        world: i32,
        kind: Identifier,
        pos: [f64; 3],
        data: Vec<u8>,
    },
    SpawnItem
    {
        world: i32,
        pos: [f64; 3],
        stack: StackData,
    },
    Entity
    {
        world: i32,
        id: i32,
    },
    EntitiesNear
    {
        world: i32,
        pos: [f64; 3],
        radius: f64,
    },
    EntityData
    {
        world: i32,
        id: i32,
    },
    SetEntityData
    {
        world: i32,
        id: i32,
        data: Vec<u8>,
    },
    MoveEntity
    {
        world: i32,
        id: i32,
        pos: Option<[f64; 3]>,
        motion: Option<[f64; 3]>,
        rotation: Option<[f32; 2]>,
    },
    DamageEntity
    {
        world: i32,
        id: i32,
        source: Option<i32>,
        amount: i32,
    },
    KillEntity
    {
        world: i32,
        id: i32,
    },
    MoveTowards
    {
        world: i32,
        id: i32,
        pos: BlockPos,
        speed: f32,
    },
    Players
    {
        world: Option<i32>,
    },
    Chat
    {
        player: Option<i32>,
        message: String,
    },
    Give
    {
        player: i32,
        stack: StackData,
    },
    HeldStack
    {
        player: i32,
    },
    SetHeldStack
    {
        player: i32,
        stack: Option<StackData>,
    },
    Teleport
    {
        player: i32,
        dimension: i32,
        pos: [f64; 3],
    },
    PlaySound
    {
        world: i32,
        pos: [f64; 3],
        sound: String,
        volume: f32,
        pitch: f32,
    },
    Particle
    {
        world: i32,
        name: String,
        pos: [f64; 3],
        motion: [f64; 3],
    },
    Explode
    {
        world: i32,
        pos: [f64; 3],
        power: f32,
    },
    Send
    {
        channel: Identifier,
        target: Target,
        data: Vec<u8>,
    },
    CanSend
    {
        player: i32,
        channel: Identifier,
    },
    OpenScreen
    {
        player: i32,
        screen: Identifier,
        pos: Option<BlockPos>,
        data: Vec<u8>,
    },
    CloseScreen
    {
        player: i32,
    },
    WorldData
    {
        key: String,
    },
    SetWorldData
    {
        key: String,
        data: Vec<u8>,
    },
    PlayerData
    {
        player: i32,
        key: String,
    },
    SetPlayerData
    {
        player: i32,
        key: String,
        data: Vec<u8>,
    },
    LocalPlayer,
    ScreenSize,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum HostReply
{
    Unit,
    Bool(bool),
    Int(i32),
    Long(i64),
    Float(f32),
    Text(String),
    Bytes(Vec<u8>),
    Id(Option<Identifier>),
    Ids(Vec<Identifier>),
    Stack(Option<StackData>),
    Entity(Option<EntityInfo>),
    Entities(Vec<EntityInfo>),
    Size(i32, i32),
    Error(String),
}

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
