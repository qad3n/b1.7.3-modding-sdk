#[macro_use]
mod macros;

mod accessories;
mod behavior;
mod config;
mod containers;
mod dispatch;
mod entity_state;
pub mod events;
mod features;
mod jobs;
mod machines;
pub mod math;
mod names;
mod refs;
mod register;
mod reply;
pub mod rt;
pub mod screen;
pub mod services;
mod states;
pub mod terrain;
pub mod vanilla;
pub mod world;

pub use accessories::{Accessories, WornStack};
pub use b173_api_macros::{block_behavior, effect_behavior, entity_behavior, item_behavior, mod_main, tile_behavior};
pub use behavior::{BlockBehavior, EffectBehavior, EntityBehavior, ItemBehavior, TileBehavior, Use};
pub use config::{Config, Setting, configValue};
pub use containers::{containedFluid, drainItem, fill, fluidContainers};
pub use entity_state::EntityState;
pub use features::{FeatureBuilder, FeatureCtx, featureList};
pub use jobs::{JobBuilder, findJob, machineFuel, matchJob};
pub use machines::{machineRecipe, machineRecipes};
pub use mod_protocol::Identifier;
use mod_protocol::abi::HostCall;
pub use mod_protocol::abi::
{
    AttachmentHolder, BiomeSpec, BipedPart, BlockInfo, BlockPos, ChunkInfo, ConfigScope, ContainerSwap, DimensionSpec,
    EnergyDemand, EntityField, EntityInfo, EventKind, FaceOverlay, FeatureInfo, FluidBlock, FluidStack, GameRuleValue,
    ItemInfo, ItemModelSpec, JobMatch, JobOutput, Located, LootContext, LootDrop, LootSource, LootTarget, LootView,
    MachineRecipeView, MachineSlot, MachineView, ModelOp, ModifierOperation, MusicSpec, Permission, PlayerLayer, Pose,
    PoseInput, Priority, RayHit, RecipeShape, RecipeView, RegistryKind, SpawnEntry, StackData, StackRef, StateData,
    StructurePiece, StructureSpec, StructureStart, Target, UseAction, Worn, capability,
};
pub use mod_protocol::handshake::ChannelDirection as Direction;
use mod_protocol::identifier::IdentifierError;
pub use mod_protocol::spec::
{
    AccessorySlotSpec, AchievementSpec, ArmorSlot, ArmorSpec, AttributeSpec, BedFaces, BlockModelSpec, BlockModifySpec,
    BlockSpec, BlockStatesSpec, ConditionalLootSpec, ConditionalPoolSpec, CreatureKind, Cuboid, DamageTypeSpec,
    DropSpec, EffectModifierSpec, EffectSpec, EnergySpec, EntityModelSpec, EntitySounds, EntitySpec, FaceTextures,
    FeatureKind, FeatureSpec, FeatureStep, Flammability, FluidContainerSpec, FluidSpec, FluidStyle, FoodSpec,
    GameRuleCategory, GameRuleSpec, HeightSpec, IngredientSpec, ItemFilter, ItemModifySpec, ItemSpec, ItemToolSpec,
    JobEnergySpec, JobFluidSpec, JobInputRule, JobInputSpec, JobOutputSpec, LoadConditions, LootEntrySpec,
    LootPoolSpec, LootSpec, MachineFuelSpec, MachineJobSpec, MachineModifySpec, MachineRecipeSpec, MachineSlotSpec,
    MachineSpec, ModelBox, ModelPart, ParticleSpec, PropertySpec, RecipeModifySpec, SpawnModifySpec, SpawnSpec,
    StatFormat, StatSpec, StateCuboid, StateModelSpec, StateVariantSpec, SteeringSpec, TagEntry, TagRef,
    TileEntitySpec, ToolClass, ToolSpec, WeightedLootSpec,
};
use mod_protocol::wire::WireError;
pub use names::{BlockRef, blockIs, blockTagged, nameOf, nameWith, named, tagMembers, tagged};
pub use refs::
{
    AchievementRef, AttributeRef, ComponentRef, DamageRef, EffectRef, EnergyRef, EntityRef, FeatureRef, FluidRef,
    GameRuleRef, IntoName, LootRuleRef, MachineRef, ParticleRef, Ref, ScreenRef, SlotRef, SoundRef, StatRef, TileRef,
    kind,
};
pub use register::*;
pub(crate) use reply::
{
    expectBool, expectBytes, expectDemand, expectDouble, expectEntities, expectEntity, expectFloat, expectFluid,
    expectFluidBlock, expectId, expectInt, expectItemInfo, expectLocated, expectLong, expectRule, expectStack,
    expectStarts, expectState, expectText, expectUnit, fetchData,
};
pub use serde;
pub use states::{StateHandle, stateOf};
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

// item info by item and damage, each with the epoch it was read in
type InfoCache = HashMap<(i32, i32), (u32, Option<ItemInfo>)>;

thread_local!
{
    static MOD_ID: RefCell<String> = const { RefCell::new(String::new()) };
    // numeric ids change when a world or server remaps, the epoch invalidates cached lookups
    static EPOCH: Cell<u32> = const { Cell::new(0) };
    // nested so a hit borrows the identifier instead of cloning it into a key
    static RESOLVED: RefCell<HashMap<RegistryKind, HashMap<Identifier, (u32, i32)>>> = RefCell::new(HashMap::new());
    static INFOS: RefCell<InfoCache> = RefCell::new(HashMap::new());
}

pub fn modId() -> String { MOD_ID.with(|id| id.borrow().clone()) }

pub(crate) fn setModId(id: &str) { MOD_ID.with(|slot| *slot.borrow_mut() = id.to_owned()); }

pub(crate) fn bumpEpoch() { EPOCH.with(|epoch| epoch.set(epoch.get() + 1)); }

pub(crate) fn epoch() -> u32 { EPOCH.with(Cell::get) }

pub fn resolve(registry: RegistryKind, id: &Identifier) -> Result<i32>
{
    let epoch: u32 = epoch();
    if let Some((cachedEpoch, numeric)) =
        RESOLVED.with(|cache| cache.borrow().get(&registry).and_then(|ids| ids.get(id)).copied())
        && cachedEpoch == epoch
    {
        return Ok(numeric);
    }

    let numeric: i32 = expectInt(rt::call(HostCall::Resolve { registry, id: id.clone() }))?;
    RESOLVED.with(|cache| cache.borrow_mut().entry(registry).or_default().insert(id.clone(), (epoch, numeric)));
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

    /// needs api 1.33, the item's stack size, durability, subtypes and block shape
    /// from api 1.47 cached per content epoch by item and damage
    pub fn info(&self) -> Option<ItemInfo>
    {
        let stack: StackData = self.raw().ok()?;
        let key: (i32, i32) = (stack.id, stack.damage);
        let epoch: u32 = epoch();
        if let Some((_, info)) = INFOS.with(|infos| infos.borrow().get(&key).filter(|(at, _)| *at == epoch).copied())
        {
            return info;
        }

        let info: Option<ItemInfo> = expectItemInfo(rt::call(HostCall::ItemInfo { stack }));
        INFOS.with(|infos| infos.borrow_mut().insert(key, (epoch, info)));
        info
    }
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
        allPlayers, broadcast, containerItem, contentGeneration, craftResult, displayName, foliageColor, fuelTicks,
        gameRule, grassColor, isClient, isLoaded, itemCatalog, localPlayer, lootList, machineList, mobList, modName,
        paintSprite, recipeList, rollLoot, screenSize, send, setDynamicTexture, setGameRule, setSharedData,
        setStackComponent, setWorldData, setWorldTime, sharedData, spritePixels, stackComponent, textWidth, translate,
        unified, worldData,
    };
    pub use crate::{vanilla, *};
    pub use serde::{Deserialize, Serialize};
}
