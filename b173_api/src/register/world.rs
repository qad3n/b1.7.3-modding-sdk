use crate::rt::call;
use crate::terrain::ChunkBlocks;
use crate::{Result, dispatch, expectUnit, ownIdentifier};
use mod_protocol::Identifier;
use mod_protocol::abi::
{
    BiomeSpec, BlockPos, ChunkInfo, DimensionSpec, HostCall, RegistryKind, SpawnEntry, StructureSpec,
};
use mod_protocol::spec::{CreatureKind, WorldgenKind, WorldgenSpec};
use std::rc::Rc;

pub struct Worldgen;

impl Worldgen
{
    pub fn ore(&self, name: &str, block: &str, veinSize: i32, perChunk: i32, y: [i32; 2]) -> Result<()>
    {
        let spec: WorldgenSpec = WorldgenSpec
        {
            kind: WorldgenKind::Ore,
            block: ownIdentifier(block)?,
            dimension: Identifier::minecraft("overworld"),
            vein_size: veinSize,
            per_chunk: perChunk,
            y,
            replaces: None,
            biomes: Vec::new(),
        };
        expectUnit(call(HostCall::RegisterWorldgen { name: name.to_owned(), spec }))
    }

    pub fn feature(&self, name: &str, spec: WorldgenSpec) -> Result<()>
    {
        expectUnit(call(HostCall::RegisterWorldgen { name: name.to_owned(), spec }))
    }

    /// needs api 1.41, a named ore or tree feature the loader places, listed and turned off by name
    pub fn add(&self, name: &str, feature: crate::FeatureBuilder) -> Result<()>
    {
        crate::features::register(name, feature, None)
    }

    /// needs api 1.41, a named feature the handler places, per chunk with the chunk's corner as origin or wherever
    /// World::placeFeature puts it, returning whether it placed anything
    pub fn custom(
        &self,
        name: &str,
        feature: crate::FeatureBuilder,
        handler: impl Fn(&crate::FeatureCtx) -> bool + 'static,
    ) -> Result<()>
    {
        crate::features::register(name, feature, Some(Rc::new(handler)))
    }
}

pub struct Dimensions;

impl Dimensions
{
    /// the handle it returns sets everything else about the dimension, so handlers never name the wrong one
    pub fn register(&self, name: &str, spec: DimensionSpec) -> Result<Dimension>
    {
        expectUnit(call(HostCall::RegisterDimension { name: name.to_owned(), spec }))?;
        Ok(Dimension { m_id: ownIdentifier(name)? })
    }

    pub fn biome(&self, name: &str, spec: BiomeSpec) -> Result<()>
    {
        expectUnit(call(HostCall::RegisterBiome { name: name.to_owned(), spec }))
    }

    /// the generator runs once per populated chunk of the dimension, vanilla dimensions take generators too
    pub fn generator(&self, dimension: &str, handler: impl Fn(ChunkInfo) + 'static) -> Result<()>
    {
        addGenerator(ownIdentifier(dimension)?, handler)
    }

    /// needs api 1.7, the mod names the biome of every column, vanilla spawning snow and grass colors follow it
    pub fn biomeSource(
        &self,
        dimension: &str,
        biomes: &[&str],
        handler: impl Fn(&ChunkInfo) -> crate::ChunkBiomes + 'static,
    ) -> Result<()>
    {
        addBiomeSource(ownIdentifier(dimension)?, biomes, handler)
    }

    /// the terrain generator fills every new chunk of a mod dimension, the generator above then populates it
    pub fn terrain(&self, dimension: &str, handler: impl Fn(&ChunkInfo, &mut ChunkBlocks) + 'static) -> Result<()>
    {
        addTerrain(ownIdentifier(dimension)?, handler)
    }
}

/// one dimension this mod registered, each call sets one part of it and hands the dimension back for the next
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Dimension
{
    m_id: Identifier,
}

impl Dimension
{
    pub fn id(&self) -> &Identifier { &self.m_id }

    /// the numeric id worlds of it carry, known once registration is over
    pub fn numeric(&self) -> Option<i32> { crate::resolve(RegistryKind::Dimension, &self.m_id).ok() }

    /// fills every new chunk, 128 high, before the generators populate it
    pub fn terrain(&self, handler: impl Fn(&ChunkInfo, &mut ChunkBlocks) + 'static) -> Result<&Self>
    {
        addTerrain(self.m_id.clone(), handler)?;
        Ok(self)
    }

    /// runs once per populated chunk, structures and decoration go here
    pub fn generator(&self, handler: impl Fn(ChunkInfo) + 'static) -> Result<&Self>
    {
        addGenerator(self.m_id.clone(), handler)?;
        Ok(self)
    }

    /// needs api 1.7, the biome of every column, an index into biomes
    pub fn biomeSource(
        &self,
        biomes: &[&str],
        handler: impl Fn(&ChunkInfo) -> crate::ChunkBiomes + 'static,
    ) -> Result<&Self>
    {
        addBiomeSource(self.m_id.clone(), biomes, handler)?;
        Ok(self)
    }

    /// needs api 1.9, a world as tall as height, a multiple of 16 up to 256; generators and structures fill all of it
    /// while players place blocks only below the build limit, terrain handlers still fill the bottom 128
    pub fn height(&self, height: i32, buildLimit: i32) -> Result<&Self>
    {
        expectUnit(call(HostCall::DimensionHeight { dimension: self.m_id.clone(), height, build_limit: buildLimit }))?;
        Ok(self)
    }

    /// needs api 1.12, the seed the dimension generates from instead of the save's
    pub fn seed(&self, seed: i64) -> Result<&Self>
    {
        expectUnit(call(HostCall::DimensionSeed { dimension: self.m_id.clone(), seed }))?;
        Ok(self)
    }

    /// needs api 1.9, the creatures that may spawn at each natural spawn attempt, None keeps the biome's own list and
    /// an empty list spawns nothing there; with this the biome spawn lists of the dimension go unused
    pub fn spawns(&self, handler: impl Fn(&SpawnQuery) -> Option<Vec<SpawnEntry>> + 'static) -> Result<&Self>
    {
        let handle: u32 = dispatch::addSpawnList(Rc::new(handler));
        expectUnit(call(HostCall::RegisterSpawns { dimension: self.m_id.clone(), handler: handle }))?;
        Ok(self)
    }
}

pub struct Structures;

impl Structures
{
    /// needs api 1.10, a kind of structure this mod builds; the dimension is a bare name for this mod's own or a full
    /// identifier such as minecraft:overworld
    /// needs api 1.55, a structure template named <modid>:<name> World::placeTemplate places; data files do the same
    /// under data/<modid>/structure_templates/
    pub fn template(&self, name: &str, spec: mod_protocol::spec::StructureTemplateSpec) -> Result<()>
    {
        crate::expectUnit(crate::rt::call(mod_protocol::abi::HostCall::RegisterTemplate
        {
            name: crate::ownIdentifier(name)?.to_string(),
            spec,
        }))
    }

    pub fn register(&self, name: &str, label: &str, dimension: &str) -> Result<Structure>
    {
        addStructure(name, label, dimension, None)
    }

    /// needs api 1.10, as register with a finder that answers whether one starts in a chunk of a seed, so locate also
    /// finds the ones no chunk has built yet
    pub fn withFinder(
        &self,
        name: &str,
        label: &str,
        dimension: &str,
        finder: impl Fn(i64, i32, i32) -> bool + 'static,
    ) -> Result<Structure>
    {
        addStructure(name, label, dimension, Some(dispatch::addFinder(Rc::new(finder))))
    }
}

fn addStructure(name: &str, label: &str, dimension: &str, finder: Option<u32>) -> Result<Structure>
{
    let spec: StructureSpec = StructureSpec { name: label.to_owned(), dimension: ownIdentifier(dimension)? };
    expectUnit(call(HostCall::RegisterStructure { name: name.to_owned(), spec, finder }))?;
    Ok(Structure { m_id: ownIdentifier(name)? })
}

/// one structure kind this mod registered
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Structure
{
    m_id: Identifier,
}

impl Structure
{
    pub fn id(&self) -> &Identifier { &self.m_id }
}

/// one natural spawn attempt a spawn list handler answers
#[derive(Clone, Copy, Debug)]
pub struct SpawnQuery
{
    pub world: crate::world::World,
    pub kind: CreatureKind,
    pub pos: BlockPos,
}

/// a weighted spawn list entry, vanilla names such as minecraft:zombie or identifiers, bare names in this mod
pub fn spawnEntry(entity: &str, weight: i32) -> Result<SpawnEntry>
{
    Ok(SpawnEntry { entity: ownIdentifier(entity)?, weight })
}

fn addGenerator(dimension: Identifier, handler: impl Fn(ChunkInfo) + 'static) -> Result<()>
{
    let handle: u32 = dispatch::addGenerator(Rc::new(handler));
    expectUnit(call(HostCall::RegisterGenerator { dimension, handler: handle }))
}

fn addTerrain(dimension: Identifier, handler: impl Fn(&ChunkInfo, &mut ChunkBlocks) + 'static) -> Result<()>
{
    let handle: u32 = dispatch::addTerrain(Rc::new(handler));
    expectUnit(call(HostCall::RegisterTerrain { dimension, handler: handle }))
}

fn addBiomeSource(
    dimension: Identifier,
    biomes: &[&str],
    handler: impl Fn(&ChunkInfo) -> crate::ChunkBiomes + 'static,
) -> Result<()>
{
    let biomes: Vec<Identifier> = biomes.iter().map(|name| ownIdentifier(name)).collect::<Result<_>>()?;
    let handle: u32 = dispatch::addBiomeSource(Rc::new(handler));
    expectUnit(call(HostCall::RegisterBiomeSource { dimension, biomes, handler: handle }))
}
