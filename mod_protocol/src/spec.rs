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

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct Cuboid
{
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub texture: Option<String>,
    // the metadata values this cuboid shows for, every value when empty, as stair facings and open gates
    #[serde(default)]
    pub metas: Vec<i32>,
    // a face 2 to 5 whose neighbor must connect for the cuboid to show, as wall and fence arms
    #[serde(default)]
    pub connect: Option<i32>,
}

// the six bed sprites, a bed of the bed template picks them by facing and half as BlockBed picks vanilla's
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
    // a vanilla render type id such as 1 for crossed sprites or 2 for torches
    pub render_type: Option<i32>,
    pub cuboids: Vec<Cuboid>,
    pub translucent: bool,
    // blocks a connecting cuboid also joins besides itself and opaque cubes
    pub connects_to: Vec<Identifier>,
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
    Pose,
    // replaces the template's own AI tick, updatePlayerActionState, the template moves and attacks no more
    Ai,
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
    // friction factor entities slide with, vanilla is 0.6 and ice 0.98
    pub slipperiness: Option<f32>,
    // boxes entities collide with in block units 0 to 16, empty lets entities pass, absent keeps the template's
    pub collision: Option<Vec<Cuboid>>,
    pub bed: Option<BedFaces>,
    // decays like vanilla leaves, defaults to true for the leaves template
    pub leaves: Option<bool>,
    // keeps nearby leaves from decaying like a vanilla log
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
    // multiplies the armor sheet, so one sheet serves several materials
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
    // tints the icon, Item.getColorFromDamage
    pub color: Option<u32>,
    // a music disc the jukebox plays
    pub record: Option<RecordSpec>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct RecordSpec
{
    // a streaming sound, a mod's sounds/record/<name>.ogg plays as <modid>.record.<name>
    pub sound: String,
    // what the now playing line shows, vanilla discs show C418 - <name>
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

// one cuboid of a model part in model units, the uv overrides the part's texture offset for this box
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct ModelBox
{
    pub origin: [f32; 3],
    pub size: [i32; 3],
    pub inflate: f32,
    pub uv: Option<[i32; 2]>,
}

// a ModelRenderer: boxes sharing one pivot and rotation, children render relative to it
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
    // sheet size this part's uvs refer to, absent means the model's
    pub texture_size: Option<[i32; 2]>,
}

// a ModelBase as data, drawn by the client with the entity's pose program
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
    pub steering: Option<SteeringSpec>,
    // Entity.isImmuneToFire: fire and lava never hurt it
    pub fire_immune: bool,
    // never despawns away from players, as a boss
    pub persistent: bool,
    // nothing pushes it or adds to its motion, only its own AI moves it
    pub immovable: bool,
}

// a mount a riding player drives with movement keys and jump, as Aether's EntityMountable
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct SteeringSpec
{
    // the mount's walking speed with a rider, vanilla walking is 0.1
    pub speed: f32,
    // upward speed a jump gives, vanilla is 0.42
    pub jump: f32,
    // jumps allowed before landing again
    pub air_jumps: i32,
    // falling speed is multiplied by this every tick while the rider is not sneaking, 1 keeps vanilla falling
    pub glide: f32,
    pub step_height: f32,
    // a sneaking rider gets off once the mount stands on ground
    pub sneak_dismount: bool,
    // upward speed of a jump in the air, zero uses jump
    pub air_jump: f32,
    // air jumps only start while the mount already falls
    pub air_jump_falling: bool,
}

impl Default for SteeringSpec
{
    fn default() -> SteeringSpec
    {
        SteeringSpec {
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

// sound pool names, an absent one keeps the template's and an empty one is silent
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
