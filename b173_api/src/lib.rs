mod behavior;
mod config;
mod dispatch;
pub mod events;
mod register;
pub mod rt;
pub mod screen;
pub mod services;
pub mod terrain;
pub mod vanilla;
pub mod world;

pub use b173_api_macros::{block_behavior, entity_behavior, item_behavior, mod_main, tile_behavior};
pub use behavior::{BlockBehavior, EntityBehavior, ItemBehavior, TileBehavior, Use};
pub use config::{Config, Setting, configValue};
pub use mod_protocol::Identifier;
pub use mod_protocol::abi::{
    BiomeSpec, BipedPart, BlockInfo, BlockPos, ChunkInfo, ConfigScope, DimensionSpec, EntityField, EntityInfo,
    EventKind, ModelOp, MusicSpec, Permission, PlayerLayer, Pose, PoseInput, Priority, RayHit, RecipeShape, RecipeView,
    RegistryKind, StackData, StackRef, StateData, Target,
};
use mod_protocol::abi::{EntityInfo as RawEntity, HostCall, HostReply};
pub use mod_protocol::handshake::ChannelDirection as Direction;
pub use mod_protocol::spec::{
    ArmorSlot, ArmorSpec, BedFaces, BlockModelSpec, BlockModifySpec, BlockSpec, BlockStatesSpec, CreatureKind, Cuboid,
    DropSpec, EntityModelSpec, EntitySounds, EntitySpec, FaceTextures, Flammability, FoodSpec, ItemSpec, ItemToolSpec,
    ModelBox, ModelPart, PropertySpec, SpawnModifySpec, SpawnSpec, StateVariantSpec, SteeringSpec, TileEntitySpec,
    ToolClass, ToolSpec,
};
pub use register::*;
pub use serde;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fmt;
pub use world::{BlockCtx, Entity, Player, World};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error(pub String);

impl fmt::Display for Error
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result { formatter.write_str(&self.0) }
}

impl std::error::Error for Error {}

impl From<String> for Error
{
    fn from(text: String) -> Error { Error(text) }
}

impl From<&str> for Error
{
    fn from(text: &str) -> Error { Error(text.to_owned()) }
}

pub type Result<T> = std::result::Result<T, Error>;

fn unexpected(reply: HostReply) -> Error
{
    match reply
    {
        HostReply::Error(reason) => Error(reason),
        other => Error(format!("unexpected host reply {other:?}")),
    }
}

pub(crate) fn expectUnit(reply: HostReply) -> Result<()>
{
    match reply
    {
        HostReply::Unit => Ok(()),
        other => Err(unexpected(other)),
    }
}

pub(crate) fn expectBool(reply: HostReply) -> Result<bool>
{
    match reply
    {
        HostReply::Bool(value) => Ok(value),
        other => Err(unexpected(other)),
    }
}

pub(crate) fn expectInt(reply: HostReply) -> Result<i32>
{
    match reply
    {
        HostReply::Int(value) => Ok(value),
        other => Err(unexpected(other)),
    }
}

pub(crate) fn expectBytes(reply: HostReply) -> Result<Vec<u8>>
{
    match reply
    {
        HostReply::Bytes(bytes) => Ok(bytes),
        HostReply::Unit => Ok(Vec::new()),
        other => Err(unexpected(other)),
    }
}

pub(crate) fn expectState(reply: HostReply) -> Result<Option<StateData>>
{
    match reply
    {
        HostReply::State(state) => Ok(state),
        other => Err(unexpected(other)),
    }
}

pub(crate) fn expectStack(reply: HostReply) -> Result<Option<StackData>>
{
    match reply
    {
        HostReply::Stack(stack) => Ok(stack),
        other => Err(unexpected(other)),
    }
}

pub(crate) fn expectEntity(reply: HostReply) -> Option<RawEntity>
{
    match reply
    {
        HostReply::Entity(entity) => entity,
        _ => None,
    }
}

pub(crate) fn expectEntities(reply: HostReply) -> Vec<RawEntity>
{
    match reply
    {
        HostReply::Entities(entities) => entities,
        _ => Vec::new(),
    }
}

thread_local! {
    static MOD_ID: RefCell<String> = const { RefCell::new(String::new()) };
    // numeric ids change when a world or server remaps, the epoch invalidates cached lookups
    static EPOCH: Cell<u32> = const { Cell::new(0) };
    static RESOLVED: RefCell<HashMap<(RegistryKind, Identifier), (u32, i32)>> = RefCell::new(HashMap::new());
}

pub fn modId() -> String { MOD_ID.with(|id| id.borrow().clone()) }

pub(crate) fn setModId(id: &str) { MOD_ID.with(|slot| *slot.borrow_mut() = id.to_owned()); }

pub(crate) fn bumpEpoch() { EPOCH.with(|epoch| epoch.set(epoch.get() + 1)); }

pub(crate) fn ownId(name: &str) -> Result<Identifier>
{
    Identifier::parseIn(name, &modId()).map_err(|error| Error(error.to_string()))
}

pub fn resolve(registry: RegistryKind, id: &Identifier) -> Result<i32>
{
    let epoch: u32 = EPOCH.with(Cell::get);
    if let Some((cachedEpoch, numeric)) = RESOLVED.with(|cache| cache.borrow().get(&(registry, id.clone())).copied())
        && cachedEpoch == epoch
    {
        return Ok(numeric);
    }
    let numeric: i32 = expectInt(rt::call(HostCall::Resolve { registry, id: id.clone() }))?;
    RESOLVED.with(|cache| cache.borrow_mut().insert((registry, id.clone()), (epoch, numeric)));
    Ok(numeric)
}

pub fn nameOf(registry: RegistryKind, numeric: i32) -> Option<Identifier>
{
    match rt::call(HostCall::NameOf { registry, numeric })
    {
        HostReply::Id(id) => id,
        _ => None,
    }
}

// an item named by identifier, resolved to its numeric id when it crosses into the game
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ItemRef
{
    Named(Identifier),
    Numeric(i32),
}

impl ItemRef
{
    pub fn id(&self) -> Result<i32>
    {
        match self
        {
            ItemRef::Named(id) => resolve(RegistryKind::Item, id),
            ItemRef::Numeric(numeric) => Ok(*numeric),
        }
    }

    pub fn name(&self) -> Option<Identifier>
    {
        match self
        {
            ItemRef::Named(id) => Some(id.clone()),
            ItemRef::Numeric(numeric) => nameOf(RegistryKind::Item, *numeric),
        }
    }

    pub fn is(&self, other: &str) -> bool
    {
        let Ok(other) = Identifier::parseIn(other, &modId())
        else
        {
            return false;
        };
        match self
        {
            ItemRef::Named(id) => *id == other,
            ItemRef::Numeric(numeric) => resolve(RegistryKind::Item, &other).is_ok_and(|value| value == *numeric),
        }
    }
}

impl From<&str> for ItemRef
{
    fn from(name: &str) -> ItemRef
    {
        ItemRef::Named(Identifier::parseIn(name, &modId()).unwrap_or_else(|_| Identifier::minecraft("air")))
    }
}

impl From<Identifier> for ItemRef
{
    fn from(id: Identifier) -> ItemRef { ItemRef::Named(id) }
}

impl From<&ItemRef> for ItemRef
{
    fn from(item: &ItemRef) -> ItemRef { item.clone() }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ItemStack
{
    pub item: ItemRef,
    pub count: i32,
    pub damage: i32,
}

impl ItemStack
{
    pub fn of(item: impl Into<ItemRef>, count: i32) -> ItemStack { ItemStack { item: item.into(), count, damage: 0 } }

    pub fn withDamage(mut self, damage: i32) -> ItemStack
    {
        self.damage = damage;
        self
    }

    pub(crate) fn raw(&self) -> Result<StackData>
    {
        Ok(StackData { id: self.item.id()?, count: self.count, damage: self.damage })
    }

    pub fn fromRaw(raw: StackData) -> ItemStack
    {
        ItemStack { item: ItemRef::Numeric(raw.id), count: raw.count, damage: raw.damage }
    }

    pub fn toRaw(&self) -> Result<StackData> { self.raw() }
}

pub fn log(message: &str) { rt::log(mod_protocol::abi::LogLevel::Info, message); }

pub fn warn(message: &str) { rt::log(mod_protocol::abi::LogLevel::Warn, message); }

pub mod prelude
{
    pub use crate::events::*;
    pub use crate::screen::{Draw, Overlay, Screen};
    pub use crate::services::{Lookup, Service};
    pub use crate::world::{
        allPlayers, broadcast, contentGeneration, displayName, isClient, isLoaded, itemCatalog, localPlayer, modName,
        recipeList, screenSize, send, setStackComponent, setWorldData, setWorldTime, stackComponent, textWidth,
        translate, worldData,
    };
    pub use crate::{vanilla, *};
    pub use serde::{Deserialize, Serialize};
}
