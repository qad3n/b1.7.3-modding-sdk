use crate::Identifier;
use crate::identifier::IdentifierError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ToolClass
{
    Pickaxe,
    Axe,
    Shovel,
    Sword,
    Hoe,
    Shears,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct ToolSpec
{
    pub class: ToolClass,
    #[serde(default)]
    pub level: i32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct StackSpec
{
    pub item: Identifier,
    #[serde(default = "one")]
    pub count: i32,
    #[serde(default)]
    pub meta: i32,
}

fn one() -> i32 { 1 }

fn oneF() -> f32 { 1.0 }

impl StackSpec
{
    pub fn of(item: Identifier, count: i32) -> StackSpec { StackSpec { item, count, meta: 0 } }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct DropSpec
{
    pub item: Identifier,
    /// one value is a fixed count, two values are an inclusive range
    #[serde(default = "oneCount")]
    pub count: Vec<i32>,
    #[serde(default)]
    pub meta: i32,
    #[serde(default = "oneF")]
    pub chance: f32,
}

fn oneCount() -> Vec<i32> { vec![1] }

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct FaceTextures
{
    pub top: Option<String>,
    pub bottom: Option<String>,
    pub side: Option<String>,
    pub front: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Flammability
{
    pub encouragement: i32,
    pub flammability: i32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct Cuboid
{
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub texture: Option<String>,
    /// the metadata values this cuboid shows for, every value when empty, as stair facings and open gates
    #[serde(default)]
    pub metas: Vec<i32>,
    /// a face 2 to 5 whose neighbor must connect for the cuboid to show, as wall and fence arms
    #[serde(default)]
    pub connect: Option<i32>,
}

/// the six bed sprites, a bed of the bed template picks them by facing and half as BlockBed picks vanilla's
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct BedFaces
{
    pub top_head: String,
    pub top_foot: String,
    pub side_head: String,
    pub side_foot: String,
    pub end_head: String,
    pub end_foot: String,
    pub bottom: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct BlockModelSpec
{
    /// a vanilla render type id such as 1 for crossed sprites or 2 for torches
    pub render_type: Option<i32>,
    pub cuboids: Vec<Cuboid>,
    pub translucent: bool,
    /// blocks a connecting cuboid also joins besides itself and opaque cubes
    pub connects_to: Vec<Identifier>,
}

/// the hot path block methods a code mod can route into wasm, every other method keeps the template body
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[serde(rename_all = "snake_case")]
pub enum BlockMethod
{
    Activated,
    Clicked,
    Added,
    Removed,
    Placed,
    PlacedBy,
    NeighborChanged,
    UpdateTick,
    RandomDisplayTick,
    EntityCollided,
    EntityWalking,
    DestroyedByPlayer,
    DestroyedByExplosion,
    Harvest,
    CanPlaceAt,
    CanStay,
    PoweringTo,
    IndirectlyPoweringTo,
    CanProvidePower,
    Texture,
    ColorMultiplier,
    /// api 1.12
    FaceTexture,
    /// api 1.13
    Strength,
    Overlay,
    /// api 1.14
    FaceRotation,
    /// api 1.33
    Connections,
    /// api 1.33
    Brightness,
    /// api 1.45
    StateTexture,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ItemMethod
{
    RightClick,
    UseOnBlock,
    HitEntity,
    InteractEntity,
    BlockDestroyed,
    Icon,
    StrVsBlock,
    CanHarvest,
    Crafted,
    /// api 1.20
    UseDuration,
    UseAction,
    UsingTick,
    StoppedUsing,
    UseFinish,
    /// api 1.23
    UseIcon,
    /// api 1.33
    Name,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[serde(rename_all = "snake_case")]
pub enum EntityMethod
{
    Tick,
    LivingUpdate,
    Interact,
    Hurt,
    Death,
    CanSpawn,
    Pose,
    /// replaces the template's own AI tick, updatePlayerActionState, the template moves and attacks no more
    Ai,
    /// api 1.28: EntityLiving.moveEntityWithHeading, the mod moves the entity itself or leaves it to the template
    Travel,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[serde(rename_all = "snake_case")]
pub enum TileMethod
{
    Tick,
    Removed,
    /// api 1.20
    Pose,
    /// api 1.33
    InsertItem,
    /// api 1.33
    ExtractItem,
    /// api 1.33: both energy calls
    Energy,
    /// api 1.33: both fluid calls
    Fluid,
}

/// api 1.29
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[serde(rename_all = "snake_case")]
pub enum EffectMethod
{
    Added,
    Tick,
    Removed,
}

/// behavior binds a spec to a guest object, set by the sdk and absent for data content
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, Default)]
pub struct Behavior<M>
{
    pub handle: u32,
    pub methods: Vec<M>,
}

/// one named property of a block state and its values, the first value listed is not special
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(deny_unknown_fields)]
pub struct PropertySpec
{
    pub name: String,
    pub values: Vec<String>,
}

impl PropertySpec
{
    pub fn new(name: &str, values: &[&str]) -> PropertySpec
    {
        PropertySpec { name: name.to_owned(), values: values.iter().map(|value| (*value).to_owned()).collect() }
    }
}

/// textures for the states whose properties match every entry of when
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct StateVariantSpec
{
    pub when: BTreeMap<String, String>,
    pub texture: Option<String>,
    pub textures: Option<FaceTextures>,
}

/// the named states of a block, saved by name so a mod may add or reorder properties between versions
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct BlockStatesSpec
{
    pub properties: Vec<PropertySpec>,
    /// a property absent here defaults to its first value
    pub default: BTreeMap<String, String>,
    pub variants: Vec<StateVariantSpec>,
}

/// a change to a block another mod or vanilla owns, every absent field keeps the current value
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct BlockModifySpec
{
    pub target: Identifier,
    #[serde(default)]
    pub hardness: Option<f32>,
    #[serde(default)]
    pub resistance: Option<f32>,
    #[serde(default)]
    pub light: Option<f32>,
    #[serde(default)]
    pub opacity: Option<i32>,
    #[serde(default)]
    pub slipperiness: Option<f32>,
    #[serde(default)]
    pub flammability: Option<Flammability>,
    #[serde(default)]
    pub tool: Option<ToolSpec>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct BlockSpec
{
    pub template: Option<Identifier>,
    pub material: Option<String>,
    pub hardness: Option<f32>,
    pub resistance: Option<f32>,
    pub light: Option<f32>,
    pub opacity: Option<i32>,
    pub texture: Option<String>,
    pub textures: Option<FaceTextures>,
    pub color: Option<u32>,
    pub sound: Option<String>,
    pub name: Option<String>,
    pub tool: Option<ToolSpec>,
    pub drops: Option<Vec<DropSpec>>,
    pub tick_randomly: Option<bool>,
    pub tile_entity: Option<Identifier>,
    pub fuel: Option<i32>,
    pub flammability: Option<Flammability>,
    pub replaceable: Option<bool>,
    pub ladder: Option<bool>,
    pub redstone_connect: Option<bool>,
    pub solid_sides: Option<bool>,
    pub model: Option<BlockModelSpec>,
    pub vanilla_fallback: Option<Identifier>,
    pub item: Option<ItemSpec>,
    pub tags: Vec<Identifier>,
    pub behavior: Option<Behavior<BlockMethod>>,
    /// friction factor entities slide with, vanilla is 0.6 and ice 0.98
    pub slipperiness: Option<f32>,
    /// boxes entities collide with in block units 0 to 16, empty lets entities pass, absent keeps the template's
    pub collision: Option<Vec<Cuboid>>,
    pub bed: Option<BedFaces>,
    /// decays like vanilla leaves, defaults to true for the leaves template
    pub leaves: Option<bool>,
    /// keeps nearby leaves from decaying like a vanilla log
    pub sustains_leaves: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct FoodSpec
{
    pub heal: i32,
    #[serde(default)]
    pub wolf_food: bool,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ArmorSlot
{
    Head,
    Chest,
    Legs,
    Feet,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct ArmorSpec
{
    pub slot: ArmorSlot,
    pub protection: i32,
    pub durability: i32,
    pub texture: String,
    /// multiplies the armor sheet, so one sheet serves several materials
    #[serde(default)]
    pub color: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct ItemToolSpec
{
    pub class: ToolClass,
    pub level: i32,
    #[serde(default = "oneF")]
    pub speed: f32,
    #[serde(default)]
    pub damage: i32,
    #[serde(default)]
    pub durability: i32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct ItemSpec
{
    pub template: Option<Identifier>,
    pub texture: Option<String>,
    pub name: Option<String>,
    pub max_stack: Option<i32>,
    pub max_damage: Option<i32>,
    pub full3d: Option<bool>,
    pub food: Option<FoodSpec>,
    pub tool: Option<ItemToolSpec>,
    pub armor: Option<ArmorSpec>,
    pub container_item: Option<Identifier>,
    pub fuel: Option<i32>,
    pub places: Option<Identifier>,
    pub vanilla_fallback: Option<Identifier>,
    pub tags: Vec<Identifier>,
    pub behavior: Option<Behavior<ItemMethod>>,
    /// tints the icon, Item.getColorFromDamage
    pub color: Option<u32>,
    /// a music disc the jukebox plays
    pub record: Option<RecordSpec>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct RecordSpec
{
    /// a streaming sound, a mod's sounds/record/<name>.ogg plays as <modid>.record.<name>
    pub sound: String,
    /// what the now playing line shows, vanilla discs show C418 - <name>
    pub title: String,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum CreatureKind
{
    Monster,
    Creature,
    Water,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct SpawnSpec
{
    pub kind: CreatureKind,
    pub weight: i32,
    #[serde(default)]
    pub biomes: Vec<Identifier>,
}

/// adds an entity, vanilla or another mod's, to biome spawn lists, a weight of 0 removes it from them instead
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct SpawnModifySpec
{
    pub entity: Identifier,
    pub spawn: SpawnSpec,
}

/// one cuboid of a model part in model units, the uv overrides the part's texture offset for this box
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct ModelBox
{
    pub origin: [f32; 3],
    pub size: [i32; 3],
    pub inflate: f32,
    pub uv: Option<[i32; 2]>,
}

/// a ModelRenderer: boxes sharing one pivot and rotation, children render relative to it
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct ModelPart
{
    pub name: String,
    pub parent: Option<String>,
    pub uv: [i32; 2],
    pub mirror: bool,
    pub pivot: [f32; 3],
    pub rotation: [f32; 3],
    pub boxes: Vec<ModelBox>,
    /// sheet size this part's uvs refer to, absent means the model's
    pub texture_size: Option<[i32; 2]>,
}

/// a ModelBase as data, drawn by the client with the entity's pose program
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct EntityModelSpec
{
    pub texture_size: [i32; 2],
    pub parts: Vec<ModelPart>,
    pub shadow: f32,
}

impl Default for EntityModelSpec
{
    fn default() -> EntityModelSpec { EntityModelSpec { texture_size: [64, 32], parts: Vec::new(), shadow: 0.5 } }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct EntitySpec
{
    pub template: Option<Identifier>,
    pub texture: Option<String>,
    pub size: Option<[f32; 2]>,
    pub health: Option<i32>,
    pub speed: Option<f32>,
    pub tracking_range: Option<i32>,
    pub update_frequency: Option<i32>,
    pub send_velocity: Option<bool>,
    pub spawn: Option<SpawnSpec>,
    pub drops: Option<Vec<DropSpec>>,
    pub name: Option<String>,
    pub vanilla_fallback: Option<Identifier>,
    pub behavior: Option<Behavior<EntityMethod>>,
    pub model: Option<EntityModelSpec>,
    pub sounds: Option<EntitySounds>,
    /// removed in api 1.28, the loader refuses an entity that sets it; steer with EntityBehavior::onTravel
    pub steering: Option<SteeringSpec>,
    /// Entity.isImmuneToFire: fire and lava never hurt it
    pub fire_immune: bool,
    /// never despawns away from players, as a boss
    pub persistent: bool,
    /// nothing pushes it or adds to its motion, only its own AI moves it
    pub immovable: bool,
}

/// removed in api 1.28 and kept only so older specs still decode, see EntitySpec.steering
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct SteeringSpec
{
    /// the mount's walking speed with a rider, vanilla walking is 0.1
    pub speed: f32,
    /// upward speed a jump gives, vanilla is 0.42
    pub jump: f32,
    /// jumps allowed before landing again
    pub air_jumps: i32,
    /// falling speed is multiplied by this every tick while the rider is not sneaking, 1 keeps vanilla falling
    pub glide: f32,
    pub step_height: f32,
    /// a sneaking rider gets off once the mount stands on ground
    pub sneak_dismount: bool,
    /// upward speed of a jump in the air, zero uses jump
    pub air_jump: f32,
    /// air jumps only start while the mount already falls
    pub air_jump_falling: bool,
}

impl Default for SteeringSpec
{
    fn default() -> SteeringSpec
    {
        SteeringSpec
        {
            speed: 0.15,
            jump: 0.42,
            air_jumps: 0,
            glide: 1.0,
            step_height: 1.0,
            sneak_dismount: true,
            air_jump: 0.0,
            air_jump_falling: false,
        }
    }
}

/// sound pool names, an absent one keeps the template's and an empty one is silent
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct EntitySounds
{
    pub living: Option<String>,
    pub hurt: Option<String>,
    pub death: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct TileEntitySpec
{
    pub inventory: i32,
    pub ticks: bool,
    pub sync: bool,
    pub behavior: Option<Behavior<TileMethod>>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct ShapedSpec
{
    pub pattern: Vec<String>,
    pub key: BTreeMap<String, StackSpec>,
    pub result: StackSpec,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct ShapelessSpec
{
    pub ingredients: Vec<StackSpec>,
    pub result: StackSpec,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct SmeltingSpec
{
    pub input: Identifier,
    pub result: StackSpec,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct FuelSpec
{
    pub item: Identifier,
    pub ticks: i32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum RecipeSpec
{
    Shaped(ShapedSpec),
    Shapeless(ShapelessSpec),
    Smelting(SmeltingSpec),
    Fuel(FuelSpec),
    ShapedTagged(ShapedTaggedSpec),
    ShapelessTagged(ShapelessTaggedSpec),
}

/// one recipe slot, an item or any member of an item tag
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct IngredientSpec
{
    pub item: Option<Identifier>,
    pub tag: Option<Identifier>,
    pub meta: i32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct ShapedTaggedSpec
{
    pub pattern: Vec<String>,
    pub key: BTreeMap<String, IngredientSpec>,
    pub result: StackSpec,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct ShapelessTaggedSpec
{
    pub ingredients: Vec<IngredientSpec>,
    pub result: StackSpec,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct TagSpec
{
    pub replace: bool,
    pub values: Vec<TagValue>,
}

/// a tag member, either one id or every member of another tag
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum TagRef
{
    Id(Identifier),
    Tag(Identifier),
}

impl TagRef
{
    /// a leading # names another tag
    pub fn parse(text: &str) -> Result<TagRef, IdentifierError>
    {
        match text.strip_prefix('#')
        {
            Some(tag) => Identifier::parse(tag).map(TagRef::Tag),
            None => Identifier::parse(text).map(TagRef::Id),
        }
    }
}

/// an optional entry is skipped quietly when its id or tag does not exist
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct TagEntry
{
    pub target: TagRef,
    pub required: bool,
}

/// a tag file value, "ns:id", "#ns:tag" or { id = "...", required = false }
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(untagged)]
pub enum TagValue
{
    Plain(String),
    Entry
    {
        id: String,
        #[serde(default = "required")]
        required: bool,
    },
}

impl TagValue
{
    pub fn entry(&self) -> Result<TagEntry, IdentifierError>
    {
        match self
        {
            TagValue::Plain(text) => Ok(TagEntry { target: TagRef::parse(text)?, required: true }),
            TagValue::Entry { id, required } => Ok(TagEntry { target: TagRef::parse(id)?, required: *required }),
        }
    }
}

fn required() -> bool { true }

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum WorldgenKind
{
    Ore,
    Plant,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct WorldgenSpec
{
    #[serde(rename = "type")]
    pub kind: WorldgenKind,
    pub block: Identifier,
    #[serde(default = "overworld")]
    pub dimension: Identifier,
    #[serde(default = "defaultVein")]
    pub vein_size: i32,
    #[serde(default = "one")]
    pub per_chunk: i32,
    #[serde(default = "defaultHeight")]
    pub y: [i32; 2],
    #[serde(default)]
    pub replaces: Option<Identifier>,
    #[serde(default)]
    pub biomes: Vec<Identifier>,
}

fn overworld() -> Identifier { Identifier::minecraft("overworld") }

fn defaultVein() -> i32 { 8 }

fn defaultHeight() -> [i32; 2] { [0, 64] }

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn blockSpecSurvivesPostcard()
    {
        let spec: BlockSpec = BlockSpec
        {
            template: Some(Identifier::minecraft("stone")),
            hardness: Some(3.0),
            tool: Some(ToolSpec { class: ToolClass::Pickaxe, level: 2 }),
            behavior: Some(Behavior { handle: 4, methods: vec![BlockMethod::Activated] }),
            ..BlockSpec::default()
        };
        let bytes: Vec<u8> = crate::wire::encode(&spec);
        let back: BlockSpec = crate::wire::decode(&bytes).expect("decodes");
        assert_eq!(spec, back);
    }
}

/// drops a mod adds to a block or entity, or puts in place of its beta drops with replace
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct LootSpec
{
    pub replace: bool,
    pub entries: Vec<LootEntrySpec>,
}

/// one drop rolled on its own: chance from 0 to 1, a count from count to max, player only for entities killed by a
/// player, block_meta only for a block broken with that metadata
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct LootEntrySpec
{
    pub item: Identifier,
    #[serde(default)]
    pub meta: i32,
    #[serde(default = "one")]
    pub count: i32,
    #[serde(default)]
    pub max: Option<i32>,
    #[serde(default = "always")]
    pub chance: f32,
    #[serde(default)]
    pub player: bool,
    #[serde(default)]
    pub block_meta: Option<i32>,
}

fn always() -> f32 { 1.0 }

/// which beta liquid a mod fluid behaves like for sounds, particles, render pass, fire, slow spreading and entities
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum FluidStyle
{
    Water,
    Lava,
}

/// a mod fluid: its flowing and still blocks use the flowing_fluid and still_fluid templates with this fluid's
/// identifier as their material, and its bucket the fluid_bucket template; unset numbers follow the style
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct FluidSpec
{
    pub style: FluidStyle,
    pub flowing: Identifier,
    pub still: Identifier,
    #[serde(default)]
    pub bucket: Option<Identifier>,
    #[serde(default)]
    pub tick_rate: Option<i32>,
    #[serde(default)]
    pub decay: Option<i32>,
    #[serde(default)]
    pub nether_decay: Option<i32>,
    #[serde(default)]
    pub infinite: Option<bool>,
}

/// api 1.33: an energy kind and what one unit is worth in the loader's shared base unit, so machines of different
/// mods power each other; kinds with no worth only move between blocks of the same kind
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct EnergySpec
{
    pub worth: i64,
    pub label: String,
}

/// a status effect: a hud colour and optional icon texture, attribute modifiers added once per level, and every
/// interval ticks a heal for a positive amount or harm for a negative one, also once per level; api 1.29 lets a mod
/// run its own code when the effect starts, ticks and ends
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct EffectSpec
{
    pub color: u32,
    pub beneficial: bool,
    pub icon: Option<String>,
    pub modifiers: Vec<EffectModifierSpec>,
    pub interval: i32,
    pub amount: i32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct EffectModifierSpec
{
    pub attribute: Identifier,
    pub amount: f64,
    pub operation: crate::abi::ModifierOperation,
}

/// a mod particle on top of a beta one named as spawnParticle takes it, such as smoke, flame or reddust; colour is
/// 0xRRGGBB, scale multiplies the beta size, lifetime is in ticks, gravity only moves particles that fall
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct ParticleSpec
{
    pub base: String,
    #[serde(default)]
    pub color: Option<u32>,
    #[serde(default)]
    pub scale: Option<f32>,
    #[serde(default)]
    pub lifetime: Option<i32>,
    #[serde(default)]
    pub gravity: Option<f32>,
}

/// a mod achievement: lang keys or plain text for its title and description, an item icon and an optional parent
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct AchievementSpec
{
    pub title: String,
    pub description: String,
    pub icon: Identifier,
    #[serde(default)]
    pub parent: Option<Identifier>,
}

/// api 1.27: a number every living entity carries, modifiers change it and the value stays within min and max
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct AttributeSpec
{
    pub default: f64,
    pub min: f64,
    pub max: f64,
}

impl Default for AttributeSpec
{
    fn default() -> AttributeSpec { AttributeSpec { default: 0.0, min: f64::MIN, max: f64::MAX } }
}

/// api 1.32: where /gamerule lists a rule
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum GameRuleCategory
{
    Player,
    Mobs,
    Spawning,
    Drops,
    Updates,
    Chat,
    #[default]
    Misc,
}

/// api 1.32: a world setting saved with the save and changed with /gamerule, a whole number rule stays within min
/// and max and a true or false rule ignores them
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct GameRuleSpec
{
    pub default: crate::abi::GameRuleValue,
    pub category: GameRuleCategory,
    pub min: i32,
    pub max: i32,
}

impl Default for GameRuleSpec
{
    fn default() -> GameRuleSpec
    {
        GameRuleSpec
        {
            default: crate::abi::GameRuleValue::Bool(false),
            category: GameRuleCategory::Misc,
            min: i32::MIN,
            max: i32::MAX,
        }
    }
}

/// api 1.35: a machine recipe viewers show as its own category, its workstations are the items that run it and the
/// first is the category icon, the title falls back to that item's name
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct MachineSpec
{
    pub title: Option<String>,
    pub workstations: Vec<Identifier>,
}

/// api 1.35: one job of a machine as viewers show it, inputs to outputs with notes such as energy, time or heat
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct MachineRecipeSpec
{
    pub inputs: Vec<MachineSlotSpec>,
    pub outputs: Vec<MachineSlotSpec>,
    pub notes: Vec<String>,
}

/// api 1.35: an item or every item of a tag, meta -1 takes any damage, chance from 0 to 1 for an output
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct MachineSlotSpec
{
    #[serde(default)]
    pub item: Option<Identifier>,
    #[serde(default)]
    pub tag: Option<Identifier>,
    #[serde(default)]
    pub meta: i32,
    #[serde(default = "one")]
    pub count: i32,
    #[serde(default = "always")]
    pub chance: f32,
}

fn anyDamage() -> i32 { -1 }

fn consumed() -> bool { true }

/// api 1.40: a loot pool rolled rolls times, or a count from rolls to max_rolls, each roll picking exactly one entry
/// by weight as modern loot pools do, for block, entity and chest loot alike
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct LootPoolSpec
{
    #[serde(default = "one")]
    pub rolls: i32,
    #[serde(default)]
    pub max_rolls: Option<i32>,
    pub entries: Vec<WeightedLootSpec>,
}

/// api 1.40: an entry a pool picks with its weight out of the pool's total, no item makes the pick drop nothing;
/// an entry whose condition fails, player for an entity no player killed or block_meta for a block of other
/// metadata, leaves the pick for that roll, the same item may be listed twice
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct WeightedLootSpec
{
    #[serde(default)]
    pub item: Option<Identifier>,
    #[serde(default)]
    pub meta: i32,
    #[serde(default = "one")]
    pub count: i32,
    #[serde(default)]
    pub max: Option<i32>,
    #[serde(default = "one")]
    pub weight: i32,
    #[serde(default)]
    pub player: bool,
    #[serde(default)]
    pub block_meta: Option<i32>,
}

/// api 1.40: a job a machine runs, which the machine looks up by its inputs instead of keeping its own table, so any
/// mod adds jobs to any machine; duration in ticks, or with duration_from_fuel the machine fuel value of the stack
/// filling the first input; energy the job draws and energy it gives, both optional
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct MachineJobSpec
{
    pub inputs: Vec<JobInputSpec>,
    pub outputs: Vec<JobOutputSpec>,
    pub fluid_inputs: Vec<JobFluidSpec>,
    pub fluid_outputs: Vec<JobFluidSpec>,
    pub duration: i32,
    pub duration_from_fuel: bool,
    pub energy_in: Option<JobEnergySpec>,
    pub energy_out: Option<JobEnergySpec>,
    pub notes: Vec<String>,
}

/// api 1.40: an item or any item of a tag, meta -1 takes any damage; an input not consumed stays where it is and
/// gains wear damage each job, as a tool or catalyst
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct JobInputSpec
{
    #[serde(default)]
    pub item: Option<Identifier>,
    #[serde(default)]
    pub tag: Option<Identifier>,
    #[serde(default = "anyDamage")]
    pub meta: i32,
    #[serde(default = "one")]
    pub count: i32,
    #[serde(default = "consumed")]
    pub consumed: bool,
    #[serde(default)]
    pub wear: i32,
}

/// api 1.40: an output item with a chance from 0 to 1, or with from_input the stack that matched that input, its
/// damage set to damage when given, so a repair hands back the same item mended
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct JobOutputSpec
{
    #[serde(default)]
    pub item: Option<Identifier>,
    #[serde(default)]
    pub meta: i32,
    #[serde(default = "one")]
    pub count: i32,
    #[serde(default = "always")]
    pub chance: f32,
    #[serde(default)]
    pub from_input: Option<u32>,
    #[serde(default)]
    pub damage: Option<i32>,
}

/// api 1.40: millibuckets of a fluid or, for an input, of any fluid of a fluid tag
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct JobFluidSpec
{
    #[serde(default)]
    pub fluid: Option<Identifier>,
    #[serde(default)]
    pub tag: Option<Identifier>,
    pub amount: i32,
}

/// api 1.40: energy of a kind a job draws or gives, each tick or in total over its duration
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct JobEnergySpec
{
    pub kind: Identifier,
    pub amount: i64,
    #[serde(default)]
    pub per_tick: bool,
}

/// api 1.40: what an item or any item of a tag is worth to one machine, in the unit that machine defines (burn
/// ticks, amplifier, fuel quality); for that machine it stands over the furnace fuel registry
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct MachineFuelSpec
{
    #[serde(default)]
    pub item: Option<Identifier>,
    #[serde(default)]
    pub tag: Option<Identifier>,
    #[serde(default = "anyDamage")]
    pub meta: i32,
    pub value: i32,
}

/// api 1.40: a slot every player wears one item in beside the vanilla inventory, taking items of tag, saved with the
/// player and synced to every client; icon is the sprite an empty slot shows
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct AccessorySlotSpec
{
    pub tag: Identifier,
    #[serde(default)]
    pub icon: Option<String>,
}

fn ores() -> FeatureStep { FeatureStep::UndergroundOres }

fn baseStone() -> Identifier { Identifier::minecraft("base_stone_overworld") }

fn dirt() -> Identifier { Identifier::minecraft("dirt") }

fn treeHeight() -> i32 { 4 }

fn treeSpread() -> i32 { 3 }

/// api 1.41: the modern generation steps, features of an earlier step run first in every populated chunk and within a
/// step in the order of their stable keys, so load order never changes a chunk
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[serde(rename_all = "snake_case")]
pub enum FeatureStep
{
    RawGeneration,
    Lakes,
    LocalModifications,
    UndergroundStructures,
    SurfaceStructures,
    Strongholds,
    UndergroundOres,
    UndergroundDecoration,
    FluidSprings,
    VegetalDecoration,
    TopLayerModification,
}

/// api 1.41: a named worldgen feature the loader places, every populated chunk of its dimension whose biome is in the
/// biomes tag, or with placed only where a sapling, a structure or a decorator places it by name
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct FeatureSpec
{
    #[serde(rename = "type")]
    pub kind: FeatureKind,
    #[serde(default = "ores")]
    pub step: FeatureStep,
    #[serde(default = "overworld")]
    pub dimension: Identifier,
    #[serde(default)]
    pub biomes: Option<Identifier>,
    #[serde(default)]
    pub placed: bool,
}

/// api 1.41: what a feature places, custom ones call the mod's own handler with a seeded random
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum FeatureKind
{
    Ore(OreFeatureSpec),
    Tree(TreeFeatureSpec),
    Custom,
}

/// api 1.41: veins of a block state shaped as beta's ores, replacing only blocks of the replaces tag
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct OreFeatureSpec
{
    pub block: Identifier,
    #[serde(default)]
    pub properties: BTreeMap<String, String>,
    #[serde(default = "defaultVein")]
    pub vein_size: i32,
    #[serde(default = "one")]
    pub per_chunk: i32,
    pub height: HeightSpec,
    #[serde(default = "baseStone")]
    pub replaces: Identifier,
}

/// api 1.41: how a feature draws its height, uniform in min..max or the sum of one draw below each bound plus offset
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum HeightSpec
{
    Uniform
    {
        min: i32, max: i32
    },
    Sum
    {
        draws: Vec<i32>,
        #[serde(default)]
        offset: i32,
    },
}

/// api 1.41: a tree shaped as beta's small oak, height from min_height plus a draw below spread, on soil of the soil
/// tag, per_chunk tries in one of chance chunks
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct TreeFeatureSpec
{
    pub log: Identifier,
    #[serde(default)]
    pub log_properties: BTreeMap<String, String>,
    pub leaves: Identifier,
    #[serde(default)]
    pub leaves_properties: BTreeMap<String, String>,
    #[serde(default = "treeHeight")]
    pub min_height: i32,
    #[serde(default = "treeSpread")]
    pub spread: i32,
    #[serde(default = "dirt")]
    pub soil: Identifier,
    #[serde(default = "one")]
    pub per_chunk: i32,
    #[serde(default = "one")]
    pub chance: i32,
}

/// api 1.41: features a data file or loader.toml turns off by full name
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct FeatureModifySpec
{
    pub disable: Vec<Identifier>,
}

/// api 1.42: a weighted pool whose entries may name loot conditions and a count rule any mod registered
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct ConditionalPoolSpec
{
    #[serde(default = "one")]
    pub rolls: i32,
    #[serde(default)]
    pub max_rolls: Option<i32>,
    pub entries: Vec<ConditionalLootSpec>,
}

/// api 1.42: a pool entry that takes part in a roll only when every when condition passes and no unless condition
/// does, its count from the count_by rule when named; conditions and rules go by their registered names
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct ConditionalLootSpec
{
    #[serde(default)]
    pub item: Option<Identifier>,
    #[serde(default)]
    pub meta: i32,
    #[serde(default = "one")]
    pub count: i32,
    #[serde(default)]
    pub max: Option<i32>,
    #[serde(default = "one")]
    pub weight: i32,
    #[serde(default)]
    pub player: bool,
    #[serde(default)]
    pub block_meta: Option<i32>,
    #[serde(default)]
    pub when: Vec<Identifier>,
    #[serde(default)]
    pub unless: Vec<Identifier>,
    #[serde(default)]
    pub count_by: Option<Identifier>,
}

impl From<LootPoolSpec> for ConditionalPoolSpec
{
    fn from(pool: LootPoolSpec) -> Self
    {
        let entries: Vec<ConditionalLootSpec> = pool
            .entries
            .into_iter()
            .map(|entry| ConditionalLootSpec
            {
                item: entry.item,
                meta: entry.meta,
                count: entry.count,
                max: entry.max,
                weight: entry.weight,
                player: entry.player,
                block_meta: entry.block_meta,
                when: Vec::new(),
                unless: Vec::new(),
                count_by: None,
            })
            .collect();
        Self { rolls: pool.rolls, max_rolls: pool.max_rolls, entries }
    }
}

impl ConditionalPoolSpec
{
    /// the api 1.40 pool when no entry names a condition or count rule, so older loaders take it
    pub fn plain(&self) -> Option<LootPoolSpec>
    {
        let entries: Option<Vec<WeightedLootSpec>> = self
            .entries
            .iter()
            .map(|entry|
            {
                let ruled: bool = !entry.when.is_empty() || !entry.unless.is_empty() || entry.count_by.is_some();
                (!ruled).then(|| WeightedLootSpec
                {
                    item: entry.item.clone(),
                    meta: entry.meta,
                    count: entry.count,
                    max: entry.max,
                    weight: entry.weight,
                    player: entry.player,
                    block_meta: entry.block_meta,
                })
            })
            .collect();
        Some(LootPoolSpec { rolls: self.rolls, max_rolls: self.max_rolls, entries: entries? })
    }
}

fn bucketful() -> i32 { 1000 }

/// api 1.44: an item holding amount millibuckets of a fluid, full_meta -1 for any damage; draining it leaves the empty
/// item, none when the container is used up, and filling the empty item gives the full one
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct FluidContainerSpec
{
    pub full: Identifier,
    #[serde(default)]
    pub full_meta: i32,
    #[serde(default)]
    pub empty: Option<Identifier>,
    pub fluid: Identifier,
    #[serde(default = "bucketful")]
    pub amount: i32,
}

/// api 1.44: which stacks a block gives out, any stack, an item with meta -1 for any damage, any item of a tag, any
/// item except the members of a tag, a full container of a fluid or of any fluid, an empty container
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ItemFilter
{
    Any,
    Item
    {
        item: Identifier,
        meta: i32,
    },
    Tag(Identifier),
    Except(Identifier),
    Holding(Option<Identifier>),
    Empty,
}

/// api 1.44: the job input at index takes any item that is no member of the except tag, as a recycler takes
/// everything but its blacklist
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct JobInputRule
{
    pub input: u32,
    #[serde(default)]
    pub except: Option<Identifier>,
}

/// api 1.45: a model cuboid shown for the states whose properties match every entry of when, every state when empty
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct StateCuboid
{
    pub cuboid: Cuboid,
    pub when: BTreeMap<String, String>,
}

/// api 1.45: a block model whose cuboids follow named states instead of metadata, its cuboids' metas go unused
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct StateModelSpec
{
    pub cuboids: Vec<StateCuboid>,
}
