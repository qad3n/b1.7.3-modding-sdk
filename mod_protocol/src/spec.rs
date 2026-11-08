use crate::Identifier;
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
    // one value is a fixed count, two values are an inclusive range
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

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Cuboid
{
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub texture: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct BlockModelSpec
{
    // a vanilla render type id such as 1 for crossed sprites or 2 for torches
    pub render_type: Option<i32>,
    pub cuboids: Vec<Cuboid>,
    pub translucent: bool,
}

// the hot path block methods a code mod can route into wasm, every other method keeps the template body
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
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[serde(rename_all = "snake_case")]
pub enum TileMethod
{
    Tick,
    Removed,
}

// behavior binds a spec to a guest object, set by the sdk and absent for data content
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, Default)]
pub struct Behavior<M>
{
    pub handle: u32,
    pub methods: Vec<M>,
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
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct TagSpec
{
    pub replace: bool,
    pub values: Vec<Identifier>,
}

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
        let spec: BlockSpec = BlockSpec {
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
