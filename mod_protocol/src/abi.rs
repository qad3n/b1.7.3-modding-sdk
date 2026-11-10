use crate::Identifier;
use crate::handshake::ChannelDirection;
use crate::spec::
{
    BlockModifySpec, BlockSpec, BlockStatesSpec, CreatureKind, EntitySpec, ItemSpec, LootSpec, RecipeSpec,
    SpawnModifySpec, TagEntry, TileEntitySpec, WorldgenSpec,
};
use serde::{Deserialize, Serialize};

// guest exports b173_alloc, b173_free, b173_dispatch and b173_abi; host imports live in module b173
pub const ABI_VERSION: u32 = 3;
// variants only append, so a host also runs guests built for any version from this one up
pub const ABI_MIN_VERSION: u32 = 2;
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
    // a whole texture of any size stretched over a rectangle, blended by its alpha
    Image
    {
        texture: String, x: i32, y: i32, w: i32, h: i32
    },
    Button
    {
        id: i32, x: i32, y: i32, w: i32, h: i32, label: String, enabled: bool
    },
    Particle
    {
        name: String, pos: [f64; 3], motion: [f64; 3]
    },
    // a u v region of a texture of any size stretched over a rectangle, as Gui.drawScaledCustomSizeModalRect
    Region
    {
        texture: String, x: i32, y: i32, w: i32, h: i32, uv: [f32; 4], size: [f32; 2]
    },
    // api 1.3, Image with its color and alpha multiplied by an argb tint, as vanilla fades the portal overlay
    TintedImage
    {
        texture: String, x: i32, y: i32, w: i32, h: i32, color: u32
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
    // a container screen syncs these tile entity slots plus the player inventory through vanilla slot packets,
    // opened without a block they live only while the screen is open and drop at the player when it closes
    pub slots: Vec<SlotSpec>,
    pub player_inventory: Option<[i32; 2]>,
    // a plain screen drawn over the world without the dark backdrop, as dialogue screens are
    pub transparent: bool,
}

// a piece of a player model a mod draws over a player, as armor layers and worn accessories do
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum BipedPart
{
    Head,
    Body,
    RightArm,
    LeftArm,
    RightLeg,
    LeftLeg,
    // the cape, drawn swinging like a vanilla cloak
    Cloak,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct PlayerLayer
{
    pub texture: String,
    pub color: u32,
    pub parts: Vec<BipedPart>,
    // how far the layer's boxes grow past the player's, armor uses 1.0 and 0.5
    pub inflate: f32,
    // drawn at full brightness, as glowing armor
    pub glow: bool,
    // scales the layer about the model origin, slightly smaller layers sit inside the skin
    pub scale: f32,
    // blended by the texture's alpha, as a translucent shield
    pub blend: bool,
    // extra geometry in the player model's space after the parts, such as wings, quads and transforms only
    pub ops: Vec<ModelOp>,
}

// a call to an overlay on a vanilla container screen, positions relative to the container's corner
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum OverlayOp
{
    Draw
    {
        size: [i32; 2],
        mouse: [i32; 2],
    },
    // answered with whether the overlay took the click away from the screen
    Mouse
    {
        x: i32,
        y: i32,
        button: i32,
    },
    // api 1.2: a key typed on the screen, hovered is the stack in the vanilla slot under the mouse, answered with whether
    // the overlay took the key away from the screen
    Key
    {
        character: u32,
        key: i32,
        hovered: Option<StackData>,
    },
    // api 1.2: the mouse wheel over the screen, delta positive when rolled away from the player, answered as Mouse
    Scroll
    {
        x: i32,
        y: i32,
        delta: i32,
    },
    // api 1.5: asked each frame before the screen draws, answered with whether the overlay covers the point so the
    // vanilla slot under it shows no highlight or tooltip
    Cover
    {
        x: i32,
        y: i32,
    },
    // api 1.6: asked each frame before the screen draws, answered with whether the overlay fills the container, while
    // one does the screen draws and asks only that overlay
    Exclusive,
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
    // a mouse button let go, for sliders dragged since its click
    Release
    {
        x: i32,
        y: i32,
        button: i32,
    },
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
    // api 1.12: the sprite slot of one face of a block where it stands, -1 keeps its metadata texture
    FaceTexture
    {
        ctx: BlockCtx, face: i32
    },
    // api 1.13: how much of the block one tick of a player's digging breaks, as Forge's
    // getPlayerRelativeBlockHardness, a negative Float keeps beta's blockStrength
    Strength
    {
        ctx: BlockCtx, player: i32
    },
    // api 1.13: a sprite drawn over one face of the block after it, answered with Overlay
    Overlay
    {
        ctx: BlockCtx, face: i32
    },
    // api 1.14: quarter turns of each face's sprite where the block stands, as RenderBlocks' uvRotate fields, answered
    // with Bytes of six values by face index, anything else keeps every face unturned
    FaceRotation
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
    // api 1.20: Item.getMaxItemUseDuration, answered with Int, 0 for an item that is not held in use
    UseDuration
    {
        stack: StackData
    },
    // api 1.20: Item.getItemUseAction, answered with Int as a UseAction
    UseAction
    {
        stack: StackData
    },
    // api 1.20: Item.onUsingTick each tick the player holds it in use, left ticks to go
    UsingTick
    {
        world: i32, player: i32, stack: StackData, left: i32
    },
    // api 1.20: Item.onPlayerStoppedUsing when the player lets go early, answered with Use
    StoppedUsing
    {
        world: i32, player: i32, stack: StackData, left: i32
    },
    // api 1.20: Item.onEaten when the use runs its full duration, answered with Use
    UseFinish
    {
        world: i32, player: i32, stack: StackData
    },
    // api 1.23: the item sheet slot to draw while the stack is held in use for used ticks, as Item.getIcon with the
    // using item, answered with Int and -1 for its usual icon
    UseIcon
    {
        stack: StackData, used: i32
    },
}

// api 1.20: EnumAction, how a player holds an item in use, a bow also narrows the view as it draws
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum UseAction
{
    #[default]
    None,
    Eat,
    Drink,
    Block,
    Bow,
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
    // client only, once per frame per visible entity, the reply is a Pose
    Pose
    {
        input: PoseInput
    },
    Ai
    {
        entity: EntityInfo
    },
}

// what RenderLiving knows when it draws a living entity
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct PoseInput
{
    pub entity: EntityInfo,
    pub walk: f32,
    pub walk_speed: f32,
    pub age: f32,
    pub head_yaw: f32,
    pub head_pitch: f32,
    pub partial: f32,
    pub on_ground: bool,
    pub riding: bool,
    pub ridden_by_local: bool,
    pub riding_local: bool,
    // the rider's width plus height, zero without one
    pub rider_size: f32,
    // EntityLiving.hurtTime, counting down from 10 after a hit
    pub hurt_time: i32,
    pub synced: Vec<u8>,
}

// one step of a model draw program, GL transforms in block units after the model's -1 scale flip
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum ModelOp
{
    Push,
    Pop,
    Translate([f32; 3]),
    // degrees around an axis
    Rotate(f32, [f32; 3]),
    Scale([f32; 3]),
    Color([f32; 4]),
    // binds a texture by identifier, as Render.loadTexture
    Texture(String),
    Blend(bool),
    // draws a part and its children with this rotation and pivot
    Part
    {
        part: u16,
        rotation: [f32; 3],
        pivot: [f32; 3],
    },
    // draws a part as the spec placed it
    Rest(u16),
    // turns to face the camera like RenderSnowball, for sprites
    FaceCamera,
    // quads of x, y, z, u, v vertices in GL units and 0 to 1 texture coordinates, four per quad
    Quads(Vec<[f32; 5]>),
    // glNormal3f for the quads that follow
    Normal([f32; 3]),
    // GL_LIGHTING off or back on, for glowing sprites
    Lighting(bool),
    // api 1.8: a stack held in the hand at the end of a part, drawn as RenderBiped draws a held item
    HeldItem
    {
        part: u16,
        stack: StackData,
    },
}

// the draw program for one frame: pre runs where preRenderCallback does, model replaces ModelBase.render
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct Pose
{
    pub pre: Vec<ModelOp>,
    pub model: Vec<ModelOp>,
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
    // api 1.20: one frame of a tile entity drawn with the model its type registered, answered with Pose whose ops
    // run in block space with the origin at the block's lower corner
    Pose
    {
        ctx: BlockCtx, partial: f32, ticks: i32
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
    // the player changed this option in the mod menu, the file is already saved
    ConfigChanged(String),
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
    Overlay
    {
        handler: Handle,
        op: OverlayOp,
    },
    PlayerLayers
    {
        handler: Handle,
        player: EntityInfo,
        synced: Vec<u8>,
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
    // fills a fresh chunk of a dimension whose terrain the mod generates, the reply is a Chunk
    Terrain
    {
        handler: Handle,
        chunk: ChunkInfo,
    }, // api 1.4: the lines a tooltip handler adds for a stack, the reply is Lines
    Tooltip
    {
        handler: Handle,
        stack: StackData,
    },
    // api 1.7: the biome index and grass and leaf colors of every column of a chunk, the reply is Biomes
    BiomeSource
    {
        handler: Handle,
        chunk: ChunkInfo,
    },
    // api 1.8: one frame of a world render handler, ops in block units from the camera, the reply is Pose with model
    WorldRender
    {
        handler: Handle,
        partial: f32,
        camera: [f64; 3],
    },
    // api 1.9: the creatures of one kind that may spawn at a spot of a mod dimension, as Forge's getPossibleCreatures;
    // the reply is Spawns
    Spawns
    {
        handler: Handle,
        world: i32,
        kind: CreatureKind,
        pos: BlockPos,
    },
    // api 1.10: whether a structure kind starts in a chunk of a world with this seed, the reply is Bool
    StructureFinder
    {
        handler: Handle,
        seed: i64,
        chunk_x: i32,
        chunk_z: i32,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum GuestReply
{
    Layers(Vec<PlayerLayer>),
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
    // every block of a chunk in the index order x << 11 | z << 7 | y, with one metadata nibble per byte
    Chunk
    {
        blocks: Vec<u16>,
        meta: Vec<u8>,
    },
    Pose(Pose),
    Lines(Vec<String>),
    // api 1.7: one entry per column of a chunk, x fastest, an index into the source's biome list
    Biomes
    {
        biomes: Vec<u8>,
    },
    // api 1.9: the weighted creatures a spawn list handler offers, None keeps the biome's list, empty spawns nothing
    Spawns
    {
        entries: Option<Vec<SpawnEntry>>,
    },
    // api 1.13: the sprite a block draws over one face, None draws nothing there
    Overlay(Option<FaceOverlay>),
}

// api 1.13: a sprite laid over a block face, as BlockTFCastleMagic's runes
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub struct FaceOverlay
{
    // a terrain atlas slot, as terrainSlot answers
    pub sprite: i32,
    // multiplies the sprite, 0xFFFFFF keeps its colors
    pub color: u32,
    // drawn at full brightness instead of the light in front of the face
    pub glow: bool,
    // pixels the sprite floats out from the face
    pub lift: f32,
}

// api 1.9: one weighted creature of a spawn list, vanilla names such as minecraft:zombie or mod identifiers
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct SpawnEntry
{
    pub entity: Identifier,
    pub weight: i32,
}

// api 1.10: a kind of structure, every start of it lies in one dimension
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct StructureSpec
{
    pub name: String,
    pub dimension: Identifier,
}

// api 1.10: one part of a built structure with inclusive corners, the label is the owning mod's name for the part
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct StructurePiece
{
    pub min: [i32; 3],
    pub max: [i32; 3],
    pub label: String,
}

// api 1.10: one built structure, the origin tells two starts of a kind apart and the box holds every piece
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct StructureStart
{
    pub structure: Identifier,
    pub origin: BlockPos,
    pub min: [i32; 3],
    pub max: [i32; 3],
    pub pieces: Vec<StructurePiece>,
}

// api 1.10: where locate found a structure, generated is false when only the kind's finder says one starts there
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Located
{
    pub x: i32,
    pub z: i32,
    pub generated: bool,
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

// a typed option, key is name or category.name and a category becomes a table in config/<modid>.toml
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ConfigOption
{
    pub key: String,
    // shown in the mod menu, empty uses the name part of the key
    pub label: String,
    pub comment: String,
    pub scope: ConfigScope,
    pub kind: ConfigKind,
    // the mod reads the value once, so a change shows after a restart
    pub restart: bool,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum ConfigKind
{
    Bool
    {
        default: bool
    },
    Int
    {
        default: i64, min: i64, max: i64
    },
    Float
    {
        default: f64, min: f64, max: f64
    },
    Text
    {
        default: String
    },
    Choice
    {
        default: String, choices: Vec<String>
    },
}

impl ConfigKind
{
    pub fn defaultText(&self) -> String
    {
        match self
        {
            ConfigKind::Bool { default } => default.to_string(),
            ConfigKind::Int { default, .. } => default.to_string(),
            ConfigKind::Float { default, .. } => default.to_string(),
            ConfigKind::Text { default } | ConfigKind::Choice { default, .. } => default.clone(),
        }
    }

    // the canonical text of a value, or why it does not fit this option
    pub fn check(&self, text: &str) -> Result<String, String>
    {
        let trimmed: &str = text.trim();
        match self
        {
            ConfigKind::Bool { .. } => match trimmed.to_ascii_lowercase().as_str()
            {
                "true" => Ok("true".to_owned()),
                "false" => Ok("false".to_owned()),
                _ => Err(format!("'{trimmed}' is not true or false")),
            },
            ConfigKind::Int { min, max, .. } =>
            {
                let value: i64 = trimmed.parse().map_err(|_| format!("'{trimmed}' is not a whole number"))?;
                inRange(value, *min, *max).map(|value| value.to_string())
            }
            ConfigKind::Float { min, max, .. } =>
            {
                let value: f64 = trimmed
                    .parse()
                    .ok()
                    .filter(|value: &f64| value.is_finite())
                    .ok_or_else(|| format!("'{trimmed}' is not a number"))?;
                inRange(value, *min, *max).map(|value| value.to_string())
            }
            ConfigKind::Text { .. } => Ok(text.to_owned()),
            ConfigKind::Choice { choices, .. } => choices
                .iter()
                .find(|choice| choice.as_str() == trimmed)
                .cloned()
                .ok_or_else(|| format!("'{trimmed}' is not one of {}", choices.join(", "))),
        }
    }
}

fn inRange<T: PartialOrd + std::fmt::Display>(value: T, min: T, max: T) -> Result<T, String>
{
    if value < min || value > max
    {
        return Err(format!("{value} is outside {min} to {max}"));
    }
    Ok(value)
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
    // beds work here, otherwise they explode as in the nether
    pub beds: bool,
    // plays here instead of the vanilla tracks
    pub music: Option<MusicSpec>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct MusicSpec
{
    // a sound pool name, sounds/music/track1.ogg to track4.ogg all answer <modid>.music.track
    pub sound: String,
    // ticks of quiet between tracks, drawn between the two
    pub delay: [i32; 2],
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
    // draws over a vanilla screen by its class name, GuiInventory for the player inventory, and may take its clicks
    RegisterOverlay
    {
        target: String,
        handler: Handle,
    },
    // layers drawn over every player, asked for each player each frame
    RegisterPlayerLayer
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
    // a lightning bolt as weather strikes, seen by every player in range
    Lightning
    {
        world: i32,
        pos: [f64; 3],
    },
    // a player's inventory slot, 0 to 8 the hotbar and 9 to 35 the rest
    InventorySlot
    {
        player: i32,
        slot: i32,
    },
    SetInventorySlot
    {
        player: i32,
        slot: i32,
        stack: Option<StackData>,
    },
    // the stack a player carries on the mouse in an open inventory
    CursorStack
    {
        player: i32,
    },
    SetCursorStack
    {
        player: i32,
        stack: Option<StackData>,
    },
    // the width in pixels the client font draws a string at, zero on a server
    TextWidth
    {
        text: String,
    },
    // the vanilla block break effect, cracking particles and the dig sound, seen by every nearby player
    BlockEffect
    {
        world: i32,
        pos: BlockPos,
        block: i32,
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
    RegisterTerrain
    {
        dimension: Identifier,
        handler: Handle,
    },
    // y of the first block above the highest light blocking block in the column
    Height
    {
        world: i32,
        x: i32,
        z: i32,
    },
    // numeric fields answer Float, flags answer Bool
    EntityField
    {
        world: i32,
        id: i32,
        field: EntityField,
    },
    // World.rayTraceBlocks between two points, answers Hit
    RayTrace
    {
        world: i32,
        from: [f64; 3],
        to: [f64; 3],
        liquids: bool,
    },
    // entities whose box, grown by radius, the segment crosses, nearest first, as projectiles find what they hit
    EntitiesAlong
    {
        world: i32,
        from: [f64; 3],
        to: [f64; 3],
        radius: f64,
        exclude: Option<i32>,
    },
    // EntityLiving.mountEntity, None leaves the current vehicle
    Mount
    {
        world: i32,
        id: i32,
        vehicle: Option<i32>,
    },
    // static properties of a block id, answers BlockInfo
    BlockInfo
    {
        block: i32,
    },
    // bytes the server keeps per entity and mirrors to every client tracking it
    SyncedData
    {
        world: i32,
        id: i32,
    },
    SetSyncedData
    {
        world: i32,
        id: i32,
        data: Vec<u8>,
    },
    // flags take any value above zero as true
    SetEntityField
    {
        world: i32,
        id: i32,
        field: EntityField,
        value: f64,
    },
    // typed options shown in the mod menu, defining a known key again replaces it
    DefineConfigOptions
    {
        options: Vec<ConfigOption>,
    },
    // checked against the option kind and saved, the mod gets no ConfigChanged for its own change
    SetConfigValue
    {
        key: String,
        value: String,
    },
    // the time of every loaded world, as the time command sets it
    SetWorldTime
    {
        time: i64,
    },
    // the time a mod dimension's sun, sky and daylight show on this side, None follows the world time
    SetSkyTime
    {
        world: i32,
        time: Option<i64>,
    },
    // an operator on a server, always true in single player
    IsOp
    {
        player: i32,
    },
    // a slot of the container the player has open, as its client sees it
    OpenSlot
    {
        player: i32,
        slot: i32,
    },
    // the translated name an item shows, vanilla items included
    DisplayName
    {
        stack: StackData,
    },
    // Block.dropBlockAsItem: a block's usual drops at a spot, as a harvest callback dropping them twice needs
    DropBlockItems
    {
        world: i32,
        pos: BlockPos,
        block: i32,
        meta: i32,
    }, // abi 3: changes properties of any block, vanilla or another mod's, applied in load order after freeze
    ModifyBlock
    {
        spec: BlockModifySpec,
    },
    // abi 3: adds or removes biome spawn entries for any entity, applied in load order after vanilla spawns
    ModifySpawns
    {
        spec: SpawnModifySpec,
    },
    // abi 3: this mod's own data on any tile entity, vanilla ones included, saved with it and never synced
    TileAttachment
    {
        world: i32,
        pos: BlockPos,
    },
    SetTileAttachment
    {
        world: i32,
        pos: BlockPos,
        data: Vec<u8>,
    },
    // abi 3: this mod's own data on a chunk, saved with it and never synced
    ChunkAttachment
    {
        world: i32,
        chunk_x: i32,
        chunk_z: i32,
    },
    SetChunkAttachment
    {
        world: i32,
        chunk_x: i32,
        chunk_z: i32,
        data: Vec<u8>,
    },
    // abi 3: named states for a block this mod registered
    RegisterBlockStates
    {
        block: Identifier,
        spec: BlockStatesSpec,
    },
    // abi 3: the block and its named state at a spot, answered with HostReply::State
    BlockState
    {
        world: i32,
        pos: BlockPos,
    },
    SetBlockState
    {
        world: i32,
        pos: BlockPos,
        state: StateData,
        notify: bool,
    },
    // abi 3: a data component on a stack in place, any mod reads any component, a mod writes only its own namespace
    StackComponent
    {
        at: StackRef,
        component: Identifier,
    },
    SetStackComponent
    {
        at: StackRef,
        component: Identifier,
        data: Vec<u8>,
    },
    // api 1.2: every stack a creative item list shows, subtypes included, in numeric id order, each a full stack
    ItemCatalog,
    // api 1.2: the crafting and smelting recipes the game knows, mod recipes included
    RecipeList,
    // api 1.2: changes whenever the loader rebuilds ids or recipe tables, answered with HostReply::Int
    ContentGeneration,
    // api 1.4: lines a mod adds under the name of every item tooltip, asked for the hovered stack each frame
    RegisterTooltip
    {
        handler: Handle,
    },
    // api 1.4: the display name of a loaded mod, Minecraft for the vanilla namespace and empty for an unknown id
    ModName
    {
        modid: String,
    }, // api 1.7: wears the held stack down as using it would, a stack worn past its last use breaks
    DamageHeld
    {
        player: i32,
        amount: i32,
    }, // api 1.7: the biomes of a mod dimension come from the mod, listed in the index order its handler answers with
    RegisterBiomeSource
    {
        dimension: Identifier,
        biomes: Vec<Identifier>,
        handler: Handle,
    },
    // api 1.7: the seed a world was made with, a multiplayer client has it from the login
    WorldSeed
    {
        world: i32,
    },
    // api 1.7: the vanilla grass or foliage colorizer at a temperature and rainfall, white where no colormap is loaded
    Colorizer
    {
        foliage: bool,
        temperature: f64,
        rainfall: f64,
    },
    // api 1.7: makes the block a mob spawner that spawns this entity, vanilla names or mod identifiers
    SetSpawner
    {
        world: i32,
        pos: BlockPos,
        entity: String,
    },
    // api 1.7: runs the block's random tick now, as a world tick would when it picks that block
    RandomTick
    {
        world: i32,
        pos: BlockPos,
    },
    // api 1.7: bone meal on the block, a sapling grows, crops ripen and grass sprouts plants
    Fertilize
    {
        world: i32,
        pos: BlockPos,
    },
    // api 1.7: the uses a damageable item has, zero for anything that does not wear
    MaxDamage
    {
        item: i32,
    },
    // api 1.8: the pixels of a texture a mod draws at runtime, argb rows from the top left, bound by its name from then
    // on wherever a texture is named; a no-op on a dedicated server
    SetDynamicTexture
    {
        name: Identifier,
        width: u32,
        height: u32,
        pixels: Vec<u32>,
    },
    // api 1.8: the rgb a vanilla map draws a block in, its material's map color, answered with Int and 0 for air
    BlockMapColor
    {
        block: i32,
    },
    // api 1.8: the stack a dropped item entity carries, answered with Stack and None for any other entity
    ItemEntityStack
    {
        world: i32,
        id: i32,
    },
    // api 1.8: draws in the world after it and before the hand each frame, the handler answers ops around the camera
    RegisterWorldRender
    {
        handler: Handle,
    },
    // api 1.9: a mod dimension taller than vanilla's 128 blocks, a multiple of 16 up to 256, set by the mod that
    // registered it; world generation fills the full height while players place blocks only below the build limit
    DimensionHeight
    {
        dimension: Identifier,
        height: i32,
        build_limit: i32,
    },
    // api 1.9: the spawn list handler of a mod dimension, asked for every natural spawn attempt there
    RegisterSpawns
    {
        dimension: Identifier,
        handler: Handle,
    },
    // api 1.10: a kind of structure this mod builds, the finder answers whether one starts in a chunk of a seed
    RegisterStructure
    {
        name: String,
        spec: StructureSpec,
        finder: Option<Handle>,
    },
    // api 1.10: records a structure the mod built so every mod can find it, once per kind and origin; only the side
    // that owns the world records, a client gets an error
    AddStructure
    {
        world: i32,
        start: StructureStart,
    },
    // api 1.10: the recorded structures crossing a box, each with only the pieces that cross it, answered with Starts
    FindStructures
    {
        world: i32,
        min: [i32; 3],
        max: [i32; 3],
        structure: Option<Identifier>,
    },
    // api 1.10: the nearest structure of a kind within radius chunks, recorded ones first and then the kind's finder,
    // answered with Located
    LocateStructure
    {
        world: i32,
        structure: Identifier,
        from: BlockPos,
        radius: i32,
    },
    // api 1.11: replaces the block and keeps its tile entity with inventory and open screens, as beta's furnace swaps
    // its idle and lit blocks; only the side that owns the world swaps, answered with whether the block changed
    SwapBlock
    {
        world: i32,
        pos: BlockPos,
        block: i32,
        meta: i32,
    },
    // api 1.12: the terrain atlas slot of a block texture for the texture hooks, answered with Int, -1 when unknown; a
    // call during registration also reserves the texture a block spec does not name
    TerrainSlot
    {
        texture: String,
    },
    // api 1.12: the seed a mod dimension generates from instead of the save's, as Forge's WorldProvider.getSeed; only
    // the registering mod sets it, before any world of the dimension exists
    DimensionSeed
    {
        dimension: Identifier,
        seed: i64,
    },
    // api 1.12: starts rain, and thunder with it when asked, for a number of ticks in a world, or clears both
    Weather
    {
        world: i32,
        raining: bool,
        thundering: bool,
        ticks: i32,
    },
    // api 1.13: cuts a terrain texture into a grid of tiles, each blown up to a whole sprite of its own, as Twilight
    // Forest's giant blocks show a quarter of a quarter of the vanilla face; call while registering, then ask
    // terrainSlot for each tile by name
    TerrainTiles
    {
        texture: String,
        columns: i32,
        rows: i32,
    },
    // api 1.14: how fast a player digs a block with what it holds, EntityPlayer.getCurrentPlayerStrVsBlock, answered
    // with Float
    DigSpeed
    {
        player: i32,
        block: i32,
    },
    // api 1.15: tag entries that may name other tags with #ns:path or be optional, replace drops earlier members
    // including the vanilla ones
    AddTagEntries
    {
        registry: RegistryKind,
        tag: Identifier,
        entries: Vec<TagEntry>,
        replace: bool,
    },
    // api 1.16: turns one column of a mod dimension to a biome of its biome source, as a chunk's biome array edited
    // in place; only the side that owns the world sets it, the loader saves it with the world and tells every client
    SetBiome
    {
        world: i32,
        x: i32,
        z: i32,
        biome: Identifier,
    },
    // api 1.17: the damage type of the hit being handled, such as minecraft:fall or minecraft:arrow, answered with Id
    // and None outside an attack, hurt or onHurt call
    DamageType,
    // api 1.17: DamageEntity with a vanilla damage type, answered with Bool
    DamageEntityAs
    {
        world: i32,
        id: i32,
        source: Option<i32>,
        amount: i32,
        kind: Identifier,
    },
    // api 1.18: adds or replaces the modifier with this id on a living entity's attribute, such as
    // minecraft:movement_speed; saved with the entity and sent to a player's own client
    SetModifier
    {
        world: i32,
        id: i32,
        attribute: Identifier,
        modifier: Identifier,
        amount: f64,
        operation: ModifierOperation,
    },
    // api 1.18
    RemoveModifier
    {
        world: i32,
        id: i32,
        attribute: Identifier,
        modifier: Identifier,
    },
    // api 1.19: loot for a block or entity of any mod or of vanilla, merged in load order with data/<modid>/loot files
    SetLoot
    {
        registry: RegistryKind,
        target: Identifier,
        spec: LootSpec,
    },
    // api 1.20: holds the player's current item in use for ticks, as EntityPlayer.setItemInUse, 0 lets it go
    SetItemInUse
    {
        player: i32,
        ticks: i32,
    },
    // api 1.20: the player lets go of the item in use, its item hears onStoppedUsing
    StopUsingItem
    {
        player: i32,
    },
    // api 1.20: the model a tile entity type draws with, posed every frame by its behavior's pose, before any world
    TileModel
    {
        tile: Identifier,
        model: crate::spec::EntityModelSpec,
    },
    // api 1.21: a fluid named <modid>:<name>, register it before the blocks that use it as their material
    RegisterFluid
    {
        name: String,
        spec: crate::spec::FluidSpec,
    },
    // api 1.22: a status effect named <modid>:<name>
    RegisterEffect
    {
        name: String,
        spec: crate::spec::EffectSpec,
    },
    // api 1.22: a stronger or longer effect replaces the one already on the entity, level 0 is the first level
    AddEffect
    {
        world: i32,
        id: i32,
        effect: Identifier,
        level: i32,
        ticks: i32,
    },
    // api 1.22
    RemoveEffect
    {
        world: i32,
        id: i32,
        effect: Identifier,
    },
    // api 1.22: the level of an effect on an entity, answered with Int and -1 when it has none
    EffectLevel
    {
        world: i32,
        id: i32,
        effect: Identifier,
    },
    // api 1.23: the item sheet slot of a texture for the item icon hooks, answered with Int and -1 when unknown; a
    // call during registration also reserves the texture no item spec names
    ItemSlot
    {
        texture: String,
    },
}

// how a modifier combines with an attribute: adds first, then base multipliers summed, then total multipliers one by one
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ModifierOperation
{
    Add,
    MultiplyBase,
    MultiplyTotal,
}

// where a stack lives, so a component change applies to that stack and not to a copy
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum StackRef
{
    Held
    {
        player: i32
    },
    Inventory
    {
        player: i32, slot: i32
    },
    Tile
    {
        world: i32, pos: BlockPos, slot: i32
    },
}

// a block and the values of its named properties, vanilla blocks report their metadata as meta
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct StateData
{
    pub block: Identifier,
    pub properties: std::collections::BTreeMap<String, String>,
}

// entity state beyond EntityInfo that ported mods read or change, new fields go at the end
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum EntityField
{
    FallDistance,
    OnGround,
    Sneaking,
    InWater,
    Riding,
    FireTicks,
    Air,
    TicksExisted,
    // entity id of the vehicle or the rider, -1 for none, read only
    RidingId,
    RiderId,
    // EntityLiving steering input, a riding player's keys reach its vehicle through these
    MoveForward,
    MoveStrafe,
    Jumping,
    Yaw,
    Pitch,
    // a steered mount's air jumps before it lands, overriding SteeringSpec.air_jumps for this entity
    AirJumps,
    // a steered mount's speed with a rider, overriding SteeringSpec.speed for this entity
    MountSpeed,
    // EntityCreature.playerToAttack, -1 for none
    TargetId,
    // EntityMob.attackStrength, the melee damage of a monster
    AttackStrength,
    // Entity.setSize, writing either keeps the other
    Width,
    Height,
    // EntityLiving.swingProgress, above zero while a player swings its arm, read only
    SwingProgress,
    // the block id a moving_block entity draws
    Block,
    // Entity.isCollided, the last move ran into a block on any side, read only
    Collided,
    // EntityLiving.health, writing heals or hurts without a hit
    Health,
    // the bottom of the bounding box, a player's feet rather than its eyes, read only
    Bottom,
    // EntityLiving.moveSpeed, how fast its own AI walks it
    MoveSpeed,
    // Entity.stepHeight, how tall a block it walks up without jumping
    StepHeight,
    // the health it heals up to and the hearts a player sees, 20 unless a mod raised it
    MaxHealth,
    // drawn without its model or shadow on this side, as an invisibility potion in later versions
    Invisible,
    // who fired an arrow, snowball, egg, fireball or mod projectile, -1 for nobody and -2 for anything else
    Shooter,
    // api 1.12: EntityLiving.deathTime, ticks since its health ran out, it is removed when this reaches 20
    DeathTime,
    // api 1.12: the metadata a moving_block entity draws its block with
    BlockMeta,
    // api 1.20: ticks left on a player's item in use, 0 when it holds nothing in use, read only
    ItemInUse,
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
    BlockInfo(Option<BlockInfo>),
    Hit(Option<RayHit>),
    // abi 3
    State(Option<StateData>),
    // api 1.2
    Stacks(Vec<StackData>),
    Recipes(Vec<RecipeView>),
    // api 1.10
    Starts(Vec<StructureStart>),
    Located(Option<Located>),
}

// api 1.2: a recipe as a viewer shows it, an ingredient with damage -1 takes any damage
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct RecipeView
{
    // the recipe registry name, minecraft:crafting/<output> for vanilla or the mod's own, none if added outside the loader
    pub id: Option<Identifier>,
    pub output: StackData,
    pub shape: RecipeShape,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum RecipeShape
{
    // ingredients row by row, width per row
    Shaped
    {
        width: i32, inputs: Vec<Option<StackData>>
    },
    Shapeless
    {
        inputs: Vec<StackData>
    },
    Smelting
    {
        input: StackData
    },
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub struct RayHit
{
    pub pos: BlockPos,
    pub face: i32,
    pub point: [f64; 3],
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug, Default)]
pub struct BlockInfo
{
    pub opacity: i32,
    pub light: i32,
    pub hardness: f32,
    // isOpaqueCube, full cubes that hide their neighbours' faces
    pub opaque: bool,
    // the material blocks movement
    pub solid: bool,
    pub liquid: bool,
    pub replaceable: bool,
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
