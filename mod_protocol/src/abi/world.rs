use super::{BlockPos, Handle};
use crate::Identifier;
use serde::{Deserialize, Serialize};

/// api 1.9: one weighted creature of a spawn list, vanilla names such as minecraft:zombie or mod identifiers
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct SpawnEntry
{
    pub entity: Identifier,
    pub weight: i32,
}

/// api 1.10: a kind of structure, every start of it lies in one dimension
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct StructureSpec
{
    pub name: String,
    pub dimension: Identifier,
}

/// api 1.10: one part of a built structure with inclusive corners, the label is the owning mod's name for the part
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct StructurePiece
{
    pub min: [i32; 3],
    pub max: [i32; 3],
    pub label: String,
}

/// api 1.10: one built structure, the origin tells two starts of a kind apart and the box holds every piece
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct StructureStart
{
    pub structure: Identifier,
    pub origin: BlockPos,
    pub min: [i32; 3],
    pub max: [i32; 3],
    pub pieces: Vec<StructurePiece>,
}

/// api 1.10: where locate found a structure, generated is false when only the kind's finder says one starts there
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Located
{
    pub x: i32,
    pub z: i32,
    pub generated: bool,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(deny_unknown_fields)]
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
    /// a generator handle routes chunk filling into the mod, absent means flat terrain from the spec
    pub generator: Option<Handle>,
    /// beds work here, otherwise they explode as in the nether
    pub beds: bool,
    /// plays here instead of the vanilla tracks
    pub music: Option<MusicSpec>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct MusicSpec
{
    /// a sound pool name, sounds/music/track1.ogg to track4.ogg all answer <modid>.music.track
    pub sound: String,
    /// ticks of quiet between tracks, drawn between the two
    pub delay: [i32; 2],
}

/// api 1.41: a registered feature as viewers and the console list it
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct FeatureInfo
{
    pub id: Identifier,
    pub owner: String,
    pub spec: crate::spec::FeatureSpec,
    pub enabled: bool,
}
