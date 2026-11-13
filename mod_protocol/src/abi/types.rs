use crate::Identifier;
use serde::{Deserialize, Serialize};

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

/// every block callback carries where it runs and whether this side owns the world
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug, Default)]
pub struct BlockCtx
{
    pub world: i32,
    pub pos: BlockPos,
    pub block: i32,
    pub meta: i32,
    pub authority: bool,
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

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Sender
{
    pub player: Option<i32>,
    pub name: String,
    pub world: i32,
    pub op: bool,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ChunkInfo
{
    pub world: i32,
    pub x: i32,
    pub z: i32,
    pub seed: i64,
}

/// api 1.33: an amount of a registered fluid in thousandths of a bucket, minecraft:water and minecraft:lava included
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct FluidStack
{
    pub fluid: Identifier,
    pub amount: i32,
}

/// api 1.33: a fluid block where it stands, level 0 is a source and 8 or more falls from above
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct FluidBlock
{
    pub fluid: Identifier,
    pub level: i32,
    pub source: bool,
}

/// api 1.33: an item's numbers: its stack size, durability, whether damage picks a subtype and whether it is a
/// block drawn in 3d as a dropped item
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ItemInfo
{
    pub max_stack: i32,
    pub max_damage: i32,
    pub subtypes: bool,
    pub block: bool,
    pub cube: bool,
}

/// api 2.0: what a mod attachment sits on; a player is an entity
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Holder
{
    Tile
    {
        world: i32, pos: BlockPos
    },

    Chunk
    {
        world: i32, x: i32, z: i32
    },

    Entity
    {
        world: i32, id: i32
    },

    /// one value per save, shared by every dimension
    World,
}

/// api 2.0: one attachment value as it now stands on its holder, none when it was cleared
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AttachmentChange
{
    pub holder: Holder,
    pub name: crate::Identifier,
    pub data: Option<Vec<u8>>,
}

/// api 2.0: the custom event the owner of a synced attachment hears when its value changes on a side that draws
/// the world, a multiplayer client or single player; the event data is an AttachmentChange
pub fn attachmentEvent(name: &crate::Identifier) -> crate::Identifier
{
    crate::Identifier::new("loader", &format!("attached/{}/{}", name.namespace(), name.path()))
        .expect("a valid identifier stays valid under a prefix")
}

/// where a stack lives, so a component change applies to that stack and not to a copy
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

/// a block and the values of its named properties, vanilla blocks report their metadata as meta
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct StateData
{
    pub block: Identifier,
    pub properties: std::collections::BTreeMap<String, String>,
}

impl StateData
{
    /// a block with every property at its default
    pub fn of(block: Identifier) -> StateData { StateData { block, properties: std::collections::BTreeMap::new() } }

    pub fn with(mut self, property: &str, value: impl ToString) -> StateData
    {
        self.properties.insert(property.to_owned(), value.to_string());
        self
    }

    pub fn get(&self, property: &str) -> Option<&str> { self.properties.get(property).map(String::as_str) }

    /// a true or false property, false when absent
    pub fn flag(&self, property: &str) -> bool { self.get(property) == Some("true") }
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
    /// isOpaqueCube, full cubes that hide their neighbours' faces
    pub opaque: bool,
    /// the material blocks movement
    pub solid: bool,
    pub liquid: bool,
    pub replaceable: bool,
}
