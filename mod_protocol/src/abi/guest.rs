use super::
{
    BlockCtx, BlockPos, ChunkInfo, DrawList, EnergyDemand, EntityInfo, EventEnvelope, FaceOverlay, FluidStack, Handle,
    OverlayOp, PlayerLayer, Pose, PoseInput, ScreenOp, Sender, SpawnEntry, StackData, StateData,
};
use crate::Identifier;
use crate::spec::CreatureKind;
use serde::{Deserialize, Serialize};

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

    /// api 1.12: the sprite slot of one face of a block where it stands, -1 keeps its metadata texture
    FaceTexture
    {
        ctx: BlockCtx, face: i32
    },

    /// api 1.13: how much of the block one tick of a player's digging breaks, as Forge's
    /// getPlayerRelativeBlockHardness, a negative Float keeps beta's blockStrength
    Strength
    {
        ctx: BlockCtx, player: i32
    },

    /// api 1.13: a sprite drawn over one face of the block after it, answered with Overlay
    Overlay
    {
        ctx: BlockCtx, face: i32
    },

    /// api 1.14: quarter turns of each face's sprite where the block stands, as RenderBlocks' uvRotate fields, answered
    /// with Bytes of six values by face index, anything else keeps every face unturned
    FaceRotation
    {
        ctx: BlockCtx
    },

    /// api 1.33: which faces a connecting block joins where it stands, a bit per face index, answered with Int; the
    /// loader keeps the answer for the rest of the tick for its model, collision and selection
    Connections
    {
        ctx: BlockCtx
    },

    /// api 1.33: how bright the block draws where it stands given the light there, as Block.getBlockBrightness,
    /// answered with Float, a negative one keeps the light
    Brightness
    {
        ctx: BlockCtx, base: f32
    },

    /// api 1.45: the sprite slot of a face for a named state, answered with Int, a negative one keeps the texture
    StateTexture
    {
        face: i32, state: StateData
    },

    /// api 1.53: an event sent with World::blockEvent reached the block on this side
    BlockEvent
    {
        ctx: BlockCtx, kind: i32, data: i32
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

    /// api 1.20: Item.getMaxItemUseDuration, answered with Int, 0 for an item that is not held in use
    UseDuration
    {
        stack: StackData
    },

    /// api 1.20: Item.getItemUseAction, answered with Int as a UseAction
    UseAction
    {
        stack: StackData
    },

    /// api 1.20: Item.onUsingTick each tick the player holds it in use, left ticks to go
    UsingTick
    {
        world: i32, player: i32, stack: StackData, left: i32
    },

    /// api 1.20: Item.onPlayerStoppedUsing when the player lets go early, answered with Use
    StoppedUsing
    {
        world: i32, player: i32, stack: StackData, left: i32
    },

    /// api 1.20: Item.onEaten when the use runs its full duration, answered with Use
    UseFinish
    {
        world: i32, player: i32, stack: StackData
    },

    /// api 1.23: the item sheet slot to draw while the stack is held in use for used ticks, as Item.getIcon with the
    /// using item, answered with Int and -1 for its usual icon
    UseIcon
    {
        stack: StackData, used: i32
    },

    /// api 1.33: the name one stack shows, as Item.getItemNameIS with its own localization, answered with Lines of
    /// one name or Unit to keep the item's name
    Name
    {
        stack: StackData
    },
}

/// api 1.20: EnumAction, how a player holds an item in use, a bow also narrows the view as it draws
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

    /// client only, once per frame per visible entity, the reply is a Pose
    Pose
    {
        input: PoseInput
    },

    Ai
    {
        entity: EntityInfo
    },

    /// api 1.28: the reply is Bool, false lets the template move the entity
    Travel
    {
        entity: EntityInfo, strafe: f32, forward: f32
    },
}

/// api 1.29: added and removed run where the effect is added and removed, tick on both sides and on a client only for
/// its own player
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum EffectCall
{
    /// the effect started or a stronger or longer one replaced it, level 0 is the first level
    Added
    {
        entity: EntityInfo, effect: Identifier, level: i32
    },

    /// every tick after the effect counted down, ticks left before it ends
    Tick
    {
        entity: EntityInfo, effect: Identifier, level: i32, ticks: i32
    },

    /// the effect ran out or was removed
    Removed
    {
        entity: EntityInfo, effect: Identifier
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

    /// api 1.20: one frame of a tile entity drawn with the model its type registered, answered with Pose whose ops
    /// run in block space with the origin at the block's lower corner
    Pose
    {
        ctx: BlockCtx, partial: f32, ticks: i32
    },

    /// api 1.33: a stack offered to the tile from a face by a pipe, machine or World::insertItem, answered with Int,
    /// the count it leaves over; simulate changes nothing
    InsertItem
    {
        ctx: BlockCtx, face: i32, stack: StackData, simulate: bool
    },

    /// api 1.33: up to max items the tile gives out of a face, answered with Stack
    ExtractItem
    {
        ctx: BlockCtx, face: i32, max: i32, simulate: bool
    },

    /// api 1.33: the energy kind the tile takes from a face and the smallest and largest packet it takes now, answered
    /// with Demand
    EnergyDemand
    {
        ctx: BlockCtx, face: i32
    },

    /// api 1.33: energy pushed into the tile from a face in the kind its demand named, the loader converts other kinds
    /// first; answered with Long, the amount it took
    ReceiveEnergy
    {
        ctx: BlockCtx, face: i32, kind: Identifier, amount: i64, simulate: bool
    },

    /// api 1.33: fluid poured into the tile from a face, answered with Int, the amount it took
    FillFluid
    {
        ctx: BlockCtx, face: i32, stack: FluidStack, simulate: bool
    },

    /// api 1.33: up to max of the fluid the tile gives out of a face, answered with Fluid
    DrainFluid
    {
        ctx: BlockCtx, face: i32, max: i32, simulate: bool
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum Lifecycle
{
    /// carries the mod id so the guest knows its namespace
    Register(String),
    InterMod,
    /// numeric ids changed after a world load or a server join, cached ids must be resolved again
    Remapped,
    /// the player changed this option in the mod menu, the file is already saved
    ConfigChanged(String),
}

/// host to guest
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

    /// fills a fresh chunk of a dimension whose terrain the mod generates, the reply is a Chunk
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

    /// api 1.7: the biome index and grass and leaf colors of every column of a chunk, the reply is Biomes
    BiomeSource
    {
        handler: Handle,
        chunk: ChunkInfo,
    },

    /// api 1.8: one frame of a world render handler, ops in block units from the camera, the reply is Pose with model
    WorldRender
    {
        handler: Handle,
        partial: f32,
        camera: [f64; 3],
    },

    /// api 1.9: the creatures of one kind that may spawn at a spot of a mod dimension, as Forge's getPossibleCreatures;
    /// the reply is Spawns
    Spawns
    {
        handler: Handle,
        world: i32,
        kind: CreatureKind,
        pos: BlockPos,
    },

    /// api 1.10: whether a structure kind starts in a chunk of a world with this seed, the reply is Bool
    StructureFinder
    {
        handler: Handle,
        seed: i64,
        chunk_x: i32,
        chunk_z: i32,
    },

    /// api 1.29
    Effect
    {
        handle: Handle,
        call: EffectCall,
    },

    /// api 1.41: places a custom feature, the reply is Bool, whether it placed anything
    Feature
    {
        handler: Handle,
        call: FeatureCall,
    },

    /// api 1.42: whether a loot condition passes, the reply is Bool
    LootCondition
    {
        handler: Handle,
        context: LootContext,
    },

    /// api 1.42: the count a loot rule gives, the reply is Int
    LootCount
    {
        handler: Handle,
        context: LootContext,
    },

    /// api 1.54: on the server, what a configuration task sends a joining player, the reply is Bytes and empty skips
    /// the task for that player
    ConfigTask
    {
        handler: Handle,
        player: String,
    },

    /// api 1.54: on the client, what the server's configuration task sent, the reply is Unit to go on or Error to refuse
    /// the join with a reason
    ConfigTaskData
    {
        handler: Handle,
        data: Vec<u8>,
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
    /// every block of a chunk in the index order x << 11 | z << 7 | y, with one metadata nibble per byte
    Chunk
    {
        blocks: Vec<u16>,
        meta: Vec<u8>,
    },
    Pose(Pose),
    Lines(Vec<String>),
    /// api 1.7: one entry per column of a chunk, x fastest, an index into the source's biome list
    Biomes
    {
        biomes: Vec<u8>,
    },

    /// api 1.9: the weighted creatures a spawn list handler offers, None keeps the biome's list, empty spawns nothing
    Spawns
    {
        entries: Option<Vec<SpawnEntry>>,
    },
    /// api 1.13: the sprite a block draws over one face, None draws nothing there
    Overlay(Option<FaceOverlay>),
    /// api 1.33
    Long(i64),
    /// api 1.33
    Demand(Option<EnergyDemand>),
    /// api 1.33
    Fluid(Option<FluidStack>),
}

/// api 1.42: what a loot condition or count rule knows about one roll, the dying entity and its killer still in the
/// world, the broken block at pos, tool the killer's held stack, seed for the rule's own random
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct LootContext
{
    pub world: i32,
    pub pos: Option<BlockPos>,
    pub state: Option<StateData>,
    pub entity: Option<i32>,
    pub killer: Option<i32>,
    pub tool: Option<StackData>,
    pub seed: i64,
}

/// api 1.41: one placement of a custom feature, origin the chunk's corner when chunk is set or the spot it is placed
/// at, the random seeded from seed
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct FeatureCall
{
    pub world: i32,
    pub origin: BlockPos,
    pub seed: i64,
    pub chunk: bool,
}
