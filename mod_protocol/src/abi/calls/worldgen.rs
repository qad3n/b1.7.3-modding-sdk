use crate::Identifier;
use crate::abi::{BiomeSpec, BlockPos, DimensionSpec, Handle, StructureSpec, StructureStart};
use crate::spec::{CreatureKind, WorldgenSpec};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum WorldgenRequest
{
    RegisterWorldgen
    {
        name: String,
        #[serde(with = "crate::wire::named")]
        spec: WorldgenSpec,
    },

    RegisterGenerator
    {
        dimension: Identifier, handler: Handle
    },

    RegisterBiome
    {
        name: String,
        #[serde(with = "crate::wire::named")]
        spec: BiomeSpec,
    },

    RegisterDimension
    {
        name: String,
        #[serde(with = "crate::wire::named")]
        spec: DimensionSpec,
    },

    RegisterTerrain
    {
        dimension: Identifier, handler: Handle
    },

    RegisterBiomeSource
    {
        dimension: Identifier, biomes: Vec<Identifier>, handler: Handle
    },

    /// api 1.7: the seed a world was made with, a multiplayer client has it from the login
    WorldSeed
    {
        world: i32
    },

    /// api 1.7: the vanilla grass or foliage colorizer at a temperature and rainfall, white where no colormap is loaded
    Colorizer
    {
        foliage: bool, temperature: f64, rainfall: f64
    },

    /// api 1.9: a mod dimension taller than vanilla's 128 blocks, a multiple of 16 up to 256, set by the mod that
    /// registered it; world generation fills the full height while players place blocks only below the build limit
    DimensionHeight
    {
        dimension: Identifier, height: i32, build_limit: i32
    },

    /// api 1.9: the spawn list handler of a mod dimension, asked for every natural spawn attempt there
    RegisterSpawns
    {
        dimension: Identifier, handler: Handle
    },

    /// api 1.10: a kind of structure this mod builds, the finder answers whether one starts in a chunk of a seed
    RegisterStructure
    {
        name: String,
        #[serde(with = "crate::wire::named")]
        spec: StructureSpec,
        finder: Option<Handle>,
    },

    /// api 1.10: records a structure the mod built so every mod can find it, once per kind and origin; only the side
    /// that owns the world records, a client gets an error
    AddStructure
    {
        world: i32, start: StructureStart
    },

    /// api 1.10: the recorded structures crossing a box, each with only the pieces that cross it, answered with Starts
    FindStructures
    {
        world: i32, min: [i32; 3], max: [i32; 3], structure: Option<Identifier>
    },

    /// api 1.10: the nearest structure of a kind within radius chunks, recorded ones first and then the kind's finder,
    /// answered with Located
    LocateStructure
    {
        world: i32, structure: Identifier, from: BlockPos, radius: i32
    },

    /// api 1.12: the seed a mod dimension generates from instead of the save's, as Forge's WorldProvider.getSeed; only
    /// the registering mod sets it, before any world of the dimension exists
    DimensionSeed
    {
        dimension: Identifier, seed: i64
    },

    /// api 1.16: turns one column of a mod dimension to a biome of its biome source, as a chunk's biome array edited
    /// in place; only the side that owns the world sets it, the loader saves it with the world and tells every client
    SetBiome
    {
        world: i32, x: i32, z: i32, biome: Identifier
    },

    /// api 1.33: the biome a column of a world holds, answered with Id
    BiomeAt
    {
        world: i32, x: i32, z: i32
    },

    /// api 1.41: a named worldgen feature, a custom one with the handler that places it, registration only
    RegisterFeature
    {
        name: String,
        #[serde(with = "crate::wire::named")]
        spec: crate::spec::FeatureSpec,
        handler: Option<Handle>,
    },

    /// api 1.41: places a feature of any mod once at a position with a random seeded from seed, nothing when the
    /// feature is turned off, authority only, answered with Bool
    PlaceFeature
    {
        feature: Identifier, world: i32, pos: BlockPos, seed: i64
    },

    /// api 1.41: every registered feature in run order, answered with Features
    Features,

    /// api 1.43: what may spawn of a kind in a vanilla or mod biome, the list the game spawns from with every mod's
    /// spawn entries and spawn edits applied, answered with SpawnEntries
    BiomeSpawns
    {
        biome: Identifier, kind: CreatureKind
    },

    /// api 1.55: a structure template named <modid>:<name>, registration only
    RegisterTemplate
    {
        name: String,
        #[serde(with = "crate::wire::named")]
        spec: crate::spec::StructureTemplateSpec,
    },

    /// api 1.55: places a template of any mod with its first layer's first corner at origin, turned a quarter turn
    /// clockwise rotation times, chests filled with a seed, authority only, answered with Bool
    PlaceTemplate
    {
        world: i32, template: Identifier, origin: BlockPos, rotation: i32, seed: i64
    },
}
