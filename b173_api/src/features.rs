use crate::rt::call;
use crate::terrain::Random;
use crate::world::World;
use crate::{BlockPos, Result, dispatch, epoch, expectBool, expectUnit, ownIdentifier};
use mod_protocol::Identifier;
use mod_protocol::abi::{FeatureInfo, HostReply, WorldgenRequest};
use mod_protocol::spec::{FeatureKind, FeatureSpec, FeatureStep, HeightSpec, OreFeatureSpec, TreeFeatureSpec};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

pub(crate) type FeatureHandler = Rc<dyn Fn(&FeatureCtx) -> bool>;

thread_local!
{
    // this mod's custom features by full name, placed in process so a decorator's random runs on unbroken
    static OWN: RefCell<HashMap<Identifier, u32>> = RefCell::new(HashMap::new());
    static LIST: RefCell<Option<(u32, Rc<Vec<FeatureInfo>>)>> = const { RefCell::new(None) };
}

/// one placement of a custom feature, origin the chunk's corner when chunk is set or the spot it is placed at
pub struct FeatureCtx<'a>
{
    pub world: World,
    pub origin: BlockPos,
    pub random: &'a Random,
    pub chunk: bool,
}

/// needs api 1.41, a named worldgen feature built up step by step
#[derive(Clone, Debug)]
pub struct FeatureBuilder
{
    pub m_spec: FeatureSpec,
}

impl FeatureBuilder
{
    /// per_chunk veins of block, vein_size blocks each, at heights the height spec draws
    pub fn ore(block: &str, veinSize: i32, perChunk: i32, height: HeightSpec) -> Result<Self>
    {
        let ore: OreFeatureSpec = OreFeatureSpec
        {
            block: ownIdentifier(block)?,
            properties: BTreeMap::new(),
            vein_size: veinSize,
            per_chunk: perChunk,
            height,
            replaces: Identifier::minecraft("base_stone_overworld"),
        };
        Ok(Self::build(FeatureKind::Ore(ore), FeatureStep::UndergroundOres))
    }

    /// a small beta oak shaped tree of log and leaves on any block of minecraft:dirt
    pub fn tree(log: &str, leaves: &str) -> Result<Self>
    {
        let tree: TreeFeatureSpec = TreeFeatureSpec
        {
            log: ownIdentifier(log)?,
            log_properties: BTreeMap::new(),
            leaves: ownIdentifier(leaves)?,
            leaves_properties: BTreeMap::new(),
            min_height: 4,
            spread: 3,
            soil: Identifier::minecraft("dirt"),
            per_chunk: 1,
            chance: 1,
        };
        Ok(Self::build(FeatureKind::Tree(tree), FeatureStep::VegetalDecoration))
    }

    /// a feature the mod's own handler places
    pub fn custom(step: FeatureStep) -> Self { Self::build(FeatureKind::Custom, step) }

    fn build(kind: FeatureKind, step: FeatureStep) -> Self
    {
        Self
        {
            m_spec: FeatureSpec
            {
                kind,
                step,
                dimension: Identifier::minecraft("overworld"),
                biomes: None,
                placed: false,
            },
        }
    }

    pub fn step(mut self, step: FeatureStep) -> Self
    {
        self.m_spec.step = step;
        self
    }

    /// a bare name for this mod's own dimension or a full identifier such as minecraft:the_nether
    pub fn dimension(mut self, dimension: &str) -> Result<Self>
    {
        self.m_spec.dimension = ownIdentifier(dimension)?;
        Ok(self)
    }

    /// only chunks whose biome is in the biome tag
    pub fn biomes(mut self, tag: &str) -> Result<Self>
    {
        self.m_spec.biomes = Some(ownIdentifier(tag.trim_start_matches('#'))?);
        Ok(self)
    }

    /// never per chunk, only where a sapling, structure or decorator places it by name
    pub fn placed(mut self) -> Self
    {
        self.m_spec.placed = true;
        self
    }

    /// a property of the ore block or the tree's log
    pub fn with(mut self, property: &str, value: &str) -> Self
    {
        match &mut self.m_spec.kind
        {
            FeatureKind::Ore(ore) => ore.properties.insert(property.to_owned(), value.to_owned()),
            FeatureKind::Tree(tree) => tree.log_properties.insert(property.to_owned(), value.to_owned()),
            FeatureKind::Custom => None,
        };
        self
    }

    pub fn leavesWith(mut self, property: &str, value: &str) -> Self
    {
        if let FeatureKind::Tree(tree) = &mut self.m_spec.kind
        {
            tree.leaves_properties.insert(property.to_owned(), value.to_owned());
        }
        self
    }

    /// the block tag ore veins replace, minecraft:base_stone_overworld unless set
    pub fn replaces(mut self, tag: &str) -> Result<Self>
    {
        if let FeatureKind::Ore(ore) = &mut self.m_spec.kind
        {
            ore.replaces = ownIdentifier(tag.trim_start_matches('#'))?;
        }
        Ok(self)
    }

    /// trunk height from min plus a draw below spread
    pub fn height(mut self, min: i32, spread: i32) -> Self
    {
        if let FeatureKind::Tree(tree) = &mut self.m_spec.kind
        {
            tree.min_height = min;
            tree.spread = spread;
        }
        self
    }

    /// the block tag a tree grows on, minecraft:dirt unless set
    pub fn soil(mut self, tag: &str) -> Result<Self>
    {
        if let FeatureKind::Tree(tree) = &mut self.m_spec.kind
        {
            tree.soil = ownIdentifier(tag.trim_start_matches('#'))?;
        }
        Ok(self)
    }

    /// tries per chunk in one of chance chunks
    pub fn perChunk(mut self, tries: i32, chance: i32) -> Self
    {
        if let FeatureKind::Tree(tree) = &mut self.m_spec.kind
        {
            tree.per_chunk = tries;
            tree.chance = chance;
        }
        self
    }
}

pub(crate) fn register(name: &str, feature: FeatureBuilder, handler: Option<FeatureHandler>) -> Result<()>
{
    let id: Identifier = ownIdentifier(name)?;
    let handle: Option<u32> = handler.map(dispatch::addFeature);
    expectUnit(call(WorldgenRequest::RegisterFeature
    {
        name: name.to_owned(),
        spec: feature.m_spec,
        handler: handle,
    }))?;

    if let Some(handle) = handle
    {
        OWN.with(|own| own.borrow_mut().insert(id, handle));
    }

    Ok(())
}

/// needs api 1.41, every registered feature in run order, read once per content epoch
pub fn featureList() -> Rc<Vec<FeatureInfo>>
{
    let epoch: u32 = epoch();
    if let Some(list) =
        LIST.with(|cache| cache.borrow().as_ref().filter(|(at, _)| *at == epoch).map(|(_, list)| list.clone()))
    {
        return list;
    }

    let list: Rc<Vec<FeatureInfo>> = Rc::new(match call(WorldgenRequest::Features)
    {
        HostReply::Features(features) => features,
        _ => Vec::new(),
    });
    LIST.with(|cache| *cache.borrow_mut() = Some((epoch, list.clone())));
    list
}

fn enabled(id: &Identifier) -> bool { featureList().iter().any(|feature| feature.id == *id && feature.enabled) }

impl World
{
    /// needs api 1.41, places a feature of any mod once at pos, as a sapling grows its tree; this mod's own custom
    /// features run in place on the given random, others on a seed drawn from it; false when nothing was placed or
    /// the feature is turned off
    pub fn placeFeature(self, name: &str, pos: BlockPos, random: &Random) -> bool
    {
        let Ok(id) = ownIdentifier(name)
        else
        {
            return false;
        };

        if let Some(handle) = OWN.with(|own| own.borrow().get(&id).copied())
        {
            let ctx: FeatureCtx = FeatureCtx { world: self, origin: pos, random, chunk: false };
            return enabled(&id) && dispatch::feature(handle).is_some_and(|handler| handler(&ctx));
        }

        let seed: i64 = random.nextLong();
        expectBool(call(WorldgenRequest::PlaceFeature { feature: id, world: self.dimension, pos, seed }))
            .unwrap_or(false)
    }
}
