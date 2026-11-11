mod behavior;
mod config;
mod dispatch;
pub mod events;
mod register;
mod reply;
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
use mod_protocol::abi::HostCall;
pub use mod_protocol::abi::
{
    BiomeSpec, BipedPart, BlockInfo, BlockPos, ChunkInfo, ConfigScope, DimensionSpec, EntityField, EntityInfo,
    EventKind, FaceOverlay, Located, ModelOp, ModifierOperation, MusicSpec, Permission, PlayerLayer, Pose, PoseInput,
    Priority, RayHit, RecipeShape, RecipeView, RegistryKind, SpawnEntry, StackData, StackRef, StateData,
    StructurePiece, StructureSpec, StructureStart, Target, UseAction,
};
pub use mod_protocol::handshake::ChannelDirection as Direction;
use mod_protocol::identifier::IdentifierError;
pub use mod_protocol::spec::
{
    AchievementSpec, ArmorSlot, ArmorSpec, BedFaces, BlockModelSpec, BlockModifySpec, BlockSpec, BlockStatesSpec,
    CreatureKind, Cuboid, DropSpec, EffectModifierSpec, EffectSpec, EntityModelSpec, EntitySounds, EntitySpec,
    FaceTextures, Flammability, FluidSpec, FluidStyle, FoodSpec, ItemSpec, ItemToolSpec, LootEntrySpec, LootSpec,
    ModelBox, ModelPart, ParticleSpec, PropertySpec, SpawnModifySpec, SpawnSpec, StateVariantSpec, SteeringSpec,
    TagEntry, TagRef, TileEntitySpec, ToolClass, ToolSpec,
};
use mod_protocol::wire::WireError;
pub use register::*;
pub(crate) use reply::
{
    expectBool, expectBytes, expectEntities, expectEntity, expectFloat, expectId, expectInt, expectLocated, expectLong,
    expectStack, expectStarts, expectState, expectText, expectUnit, fetchData,
};
pub use serde;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fmt;
pub use world::{BlockCtx, Entity, Player, World, damageType};

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

impl From<IdentifierError> for Error
{
    fn from(error: IdentifierError) -> Error { Error(error.to_string()) }
}

impl From<WireError> for Error
{
    fn from(error: WireError) -> Error { Error(error.to_string()) }
}

pub type Result<T> = std::result::Result<T, Error>;

thread_local!
{
    static MOD_ID: RefCell<String> = const { RefCell::new(String::new()) };
    // numeric ids change when a world or server remaps, the epoch invalidates cached lookups
    static EPOCH: Cell<u32> = const { Cell::new(0) };
    static RESOLVED: RefCell<HashMap<(RegistryKind, Identifier), (u32, i32)>> = RefCell::new(HashMap::new());
}

pub fn modId() -> String { MOD_ID.with(|id| id.borrow().clone()) }

pub(crate) fn setModId(id: &str) { MOD_ID.with(|slot| *slot.borrow_mut() = id.to_owned()); }

pub(crate) fn bumpEpoch() { EPOCH.with(|epoch| epoch.set(epoch.get() + 1)); }

pub(crate) fn ownId(name: &str) -> Result<Identifier> { Ok(Identifier::parseIn(name, &modId())?) }

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

/// needs api 1.7, how many uses the item has before it breaks, zero when it does not wear
pub fn maxDamage(item: &ItemRef) -> i32
{
    let Ok(numeric) = item.id()
    else
    {
        return 0;
    };

    expectInt(rt::call(HostCall::MaxDamage { item: numeric })).unwrap_or(0)
}

pub fn nameOf(registry: RegistryKind, numeric: i32) -> Option<Identifier>
{
    expectId(rt::call(HostCall::NameOf { registry, numeric }))
}

/// an item named by identifier, resolved to its numeric id when it crosses into the game
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
        match Identifier::parseIn(name, &modId())
        {
            Ok(id) => ItemRef::Named(id),

            Err(error) =>
            {
                warn(&format!("item name '{name}' is not an identifier, using air: {error}"));
                ItemRef::Named(Identifier::minecraft("air"))
            }
        }
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

/// a biome source's answer for one chunk, one biome index per column with x fastest; grass and leaves take the
/// vanilla colorizer's colour from the biome's temperature and rainfall
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct ChunkBiomes
{
    pub biomes: Vec<u8>,
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
    pub use crate::world::
    {
        allPlayers, broadcast, contentGeneration, displayName, foliageColor, grassColor, isClient, isLoaded,
        itemCatalog, localPlayer, modName, recipeList, screenSize, send, setDynamicTexture, setStackComponent,
        setWorldData, setWorldTime, stackComponent, textWidth, translate, worldData,
    };
    pub use crate::{vanilla, *};
    pub use serde::{Deserialize, Serialize};
}
