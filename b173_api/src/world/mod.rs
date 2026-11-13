use crate::rt::{call, fire};
use crate::terrain::structure::StructureBox;
use crate::
{
    Error, ItemStack, Result, expectBool, expectDemand, expectEntities, expectEntity, expectFluid, expectFluidBlock,
    expectId, expectInt, expectLocated, expectLong, expectStack, expectStarts, expectText, expectUnit, fetchData,
};
use mod_protocol::Identifier;
use mod_protocol::abi::
{
    BlockInfo, BlockPos, EnergyDemand, EnergyLevel, EntityInfo, EntityRequest, FluidBlock, FluidStack, HostReply,
    InventoryRequest, Located, LootView, MachineView, MetaRequest, PlayerRequest, RayHit, RecipeView, RegistryRequest,
    ScreenRequest, SharedRequest, StackData, StackRef, StateData, StoredRequest, StructureStart, TankLevel,
    ViewerRequest, WorldRequest, WorldgenRequest,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::rc::Rc;

mod blast;
mod block;
mod entity;
mod player;
mod rules;

pub use block::BlockCtx;
pub use entity::Entity;
pub use player::{Player, allPlayers, broadcast, displayName, send, setWorldTime};
pub use rules::{gameRule, setGameRule};

pub fn encode<T: Serialize>(value: &T) -> Vec<u8> { mod_protocol::wire::encode(value) }

pub fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> { Ok(mod_protocol::wire::decode(bytes)?) }

// data the host keeps for the mod, in the form that survives a change of the mod's struct
pub(crate) fn encodeStored<T: Serialize>(value: &T) -> Vec<u8> { mod_protocol::wire::encodeStored(value) }

/// a loaded world, named by its dimension id
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct World
{
    pub dimension: i32,
}

impl World
{
    pub const OVERWORLD: World = World { dimension: 0 };
    pub const NETHER: World = World { dimension: -1 };

    /// needs api 1.26, the count that did not fit; simulate only reports
    pub fn insertItem(self, pos: BlockPos, face: i32, stack: ItemStack, simulate: bool) -> Result<i32>
    {
        let stack: mod_protocol::abi::StackData = stack.raw()?;
        expectInt(call(InventoryRequest::InsertItem { world: self.dimension, pos, face, stack, simulate }))
    }

    /// needs api 1.26, simulate leaves the items where they are
    pub fn extractItem(self, pos: BlockPos, face: i32, max: i32, simulate: bool) -> Option<ItemStack>
    {
        expectStack(call(InventoryRequest::ExtractItem { world: self.dimension, pos, face, max, simulate }))
            .ok()
            .flatten()
            .map(ItemStack::fromRaw)
    }

    /// needs api 1.46, the leaf at pos starts checking whether a log still holds it, as a broken log tells its leaves
    pub fn checkLeafDecay(self, pos: BlockPos) -> bool
    {
        expectBool(call(WorldRequest::CheckLeafDecay { world: self.dimension, pos })).unwrap_or(false)
    }

    /// needs api 1.44, as extractItem, taking only a stack the filter accepts
    pub fn extractMatching(
        self,
        pos: BlockPos,
        face: i32,
        filter: crate::ItemFilter,
        max: i32,
        simulate: bool,
    ) -> Option<ItemStack>
    {
        let call: InventoryRequest =
            InventoryRequest::ExtractMatching { world: self.dimension, pos, face, filter, max, simulate };
        expectStack(crate::rt::call(call)).ok().flatten().map(ItemStack::fromRaw)
    }

    /// needs api 1.33, the capability bits a block offers from a face: items, fluids and energy
    pub fn capabilities(self, pos: BlockPos, face: i32) -> i32
    {
        expectInt(call(SharedRequest::Capabilities { world: self.dimension, pos, face })).unwrap_or(0)
    }

    /// needs api 1.33, the packet sizes a block takes of an energy kind from a face now, None when it takes none
    pub fn energyDemand(self, pos: BlockPos, face: i32, kind: &Identifier) -> Option<EnergyDemand>
    {
        expectDemand(call(SharedRequest::EnergyDemand { world: self.dimension, pos, face, kind: kind.clone() }))
    }

    /// needs api 1.33, pushes energy into a block from a face, the amount it took
    pub fn sendEnergy(self, pos: BlockPos, face: i32, kind: &Identifier, amount: i64, simulate: bool) -> i64
    {
        let request: SharedRequest =
            SharedRequest::SendEnergy { world: self.dimension, pos, face, kind: kind.clone(), amount, simulate };
        expectLong(call(request)).unwrap_or(0)
    }

    /// what a block holds of an energy kind and the most it can hold, in the kind asked for; none when the block
    /// keeps no energy the loader can read
    pub fn energyLevel(self, pos: BlockPos, face: i32, kind: &Identifier) -> Option<EnergyLevel>
    {
        match call(SharedRequest::EnergyLevel { world: self.dimension, pos, face, kind: kind.clone() })
        {
            HostReply::Energy(level) => level,
            _ => None,
        }
    }

    /// draws energy out of a block from a face, the amount it gave in the kind asked for
    pub fn takeEnergy(self, pos: BlockPos, face: i32, kind: &Identifier, amount: i64, simulate: bool) -> i64
    {
        let request: SharedRequest =
            SharedRequest::TakeEnergy { world: self.dimension, pos, face, kind: kind.clone(), amount, simulate };
        expectLong(call(request)).unwrap_or(0)
    }

    /// sets what this mod's own tile holds in the energy buffer it declared, in the buffer's kind
    pub fn setEnergy(self, pos: BlockPos, amount: i64) -> Result<()>
    {
        expectUnit(call(SharedRequest::SetEnergy { world: self.dimension, pos, amount }))
    }

    /// what a block's loader-kept tank holds and its capacity, none when the block has no such tank
    pub fn tankLevel(self, pos: BlockPos) -> Option<TankLevel>
    {
        match call(SharedRequest::TankLevel { world: self.dimension, pos })
        {
            HostReply::Tank(level) => level,
            _ => None,
        }
    }

    /// sets what this mod's own tile holds in the tank it declared, none empties it
    pub fn setTank(self, pos: BlockPos, stack: Option<&FluidStack>) -> Result<()>
    {
        expectUnit(call(SharedRequest::SetTank { world: self.dimension, pos, stack: stack.cloned() }))
    }

    /// needs api 1.33, pours fluid into a block from a face, the amount it took
    pub fn fillFluid(self, pos: BlockPos, face: i32, stack: &FluidStack, simulate: bool) -> i32
    {
        let request: SharedRequest =
            SharedRequest::FillFluid { world: self.dimension, pos, face, stack: stack.clone(), simulate };
        expectInt(call(request)).unwrap_or(0)
    }

    /// needs api 1.33, takes up to max fluid out of a block from a face
    pub fn drainFluid(self, pos: BlockPos, face: i32, max: i32, simulate: bool) -> Option<FluidStack>
    {
        expectFluid(call(SharedRequest::DrainFluid { world: self.dimension, pos, face, max, simulate }))
    }

    /// needs api 1.33, the registered fluid a block holds, vanilla water and lava included
    pub fn fluidAt(self, pos: BlockPos) -> Option<FluidBlock>
    {
        expectFluidBlock(call(WorldRequest::FluidAt { world: self.dimension, pos }))
    }

    /// needs api 1.33, the stacks a block would drop if a machine mined it, leaving the block in place
    pub fn blockDrops(self, pos: BlockPos) -> Vec<ItemStack>
    {
        match call(WorldRequest::BlockDrops { world: self.dimension, pos })
        {
            HostReply::Stacks(stacks) => stacks.into_iter().map(ItemStack::fromRaw).collect(),
            _ => Vec::new(),
        }
    }

    /// needs api 1.33, the single stack a ModLoader era machine takes from a block: one roll of its dropped id and
    /// quantity, None when it drops nothing
    pub fn blockDropStack(self, pos: BlockPos) -> Option<ItemStack>
    {
        match call(WorldRequest::BlockDropStack { world: self.dimension, pos })
        {
            HostReply::Stack(stack) => stack.map(ItemStack::fromRaw),
            _ => None,
        }
    }

    /// needs api 1.33, places the block a stack holds at pos with no player, as clicked on the face of the block
    /// behind it; true when it went down, the caller takes the item from its stack
    pub fn placeBlock(self, pos: BlockPos, face: i32, stack: &ItemStack) -> Result<bool>
    {
        let stack: StackData = stack.raw()?;
        expectBool(call(WorldRequest::PlaceBlock { world: self.dimension, pos, face, stack }))
    }

    /// needs api 1.33, drops a stack with a motion and a pickup delay in ticks, the dropped item's entity
    pub fn dropItem(self, pos: [f64; 3], stack: &ItemStack, motion: [f64; 3], delay: i32) -> Result<Entity>
    {
        let stack: StackData = stack.raw()?;
        let id: i32 = expectInt(call(EntityRequest::DropItem { world: self.dimension, pos, stack, motion, delay }))?;
        Ok(Entity { world: self, id })
    }

    /// needs api 1.33, how many slots the inventory at a block has, 0 for none; tileSlot reads them by index
    pub fn inventorySize(self, pos: BlockPos) -> i32
    {
        expectInt(call(InventoryRequest::InventorySize { world: self.dimension, pos })).unwrap_or(0)
    }

    /// needs api 1.33, the biome a column holds, as minecraft:desert or a mod biome's name
    pub fn biomeAt(self, x: i32, z: i32) -> Option<Identifier>
    {
        expectId(call(WorldgenRequest::BiomeAt { world: self.dimension, x, z }))
    }

    /// needs api 1.43, what may spawn of a kind in a biome, vanilla or a mod's, with every mod's entries and edits
    pub fn biomeSpawns(biome: &str, kind: crate::CreatureKind) -> Vec<crate::SpawnEntry>
    {
        let Ok(biome) = Identifier::parse(biome)
        else
        {
            return Vec::new();
        };

        match call(WorldgenRequest::BiomeSpawns { biome, kind })
        {
            mod_protocol::abi::HostReply::SpawnEntries(entries) => entries,
            _ => Vec::new(),
        }
    }

    /// needs api 1.33, every entity whose bounding box meets a box in world coordinates
    pub fn entitiesIn(self, min: [f64; 3], max: [f64; 3]) -> Vec<EntityInfo>
    {
        expectEntities(call(EntityRequest::EntitiesIn { world: self.dimension, min, max }))
    }

    pub fn block(self, pos: BlockPos) -> i32
    {
        expectInt(call(WorldRequest::GetBlock { world: self.dimension, pos })).unwrap_or(0)
    }

    pub fn height(self, x: i32, z: i32) -> i32
    {
        expectInt(call(WorldRequest::Height { world: self.dimension, x, z })).unwrap_or(0)
    }

    pub fn isAir(self, pos: BlockPos) -> bool { self.block(pos) == 0 }

    /// air here with neighbour updates, the way a block removes itself
    pub fn clear(self, pos: BlockPos) -> Result<bool> { self.setBlock(pos, 0, 0) }

    /// needs api 1.37, the block's registry name, a hidden state page answers with its block
    pub fn blockName(self, pos: BlockPos) -> Option<Identifier>
    {
        crate::nameOf(crate::RegistryKind::Block, self.block(pos))
    }

    /// needs api 1.37, the block here is the named one, a bare name means this mod's
    pub fn is(self, pos: BlockPos, name: &str) -> bool { crate::blockIs(self.block(pos), name) }

    /// needs api 1.37, the block here sits in the tag, so other mods' blocks count too
    pub fn tagged(self, pos: BlockPos, tag: &str) -> bool { crate::blockTagged(self.block(pos), tag) }

    /// needs api 1.37, places a block by name with its default state
    pub fn place(self, pos: BlockPos, name: &str) -> Result<bool>
    {
        let block: Identifier = Identifier::parseIn(name, &crate::modId())?;
        self.setBlockState(pos, StateData::of(block))
    }

    pub fn entitiesAlong(self, from: [f64; 3], to: [f64; 3], radius: f64, exclude: Option<i32>) -> Vec<EntityInfo>
    {
        expectEntities(call(EntityRequest::EntitiesAlong { world: self.dimension, from, to, radius, exclude }))
    }

    pub fn rayTrace(self, from: [f64; 3], to: [f64; 3], liquids: bool) -> Option<RayHit>
    {
        match call(WorldRequest::RayTrace { world: self.dimension, from, to, liquids })
        {
            HostReply::Hit(hit) => hit,
            _ => None,
        }
    }

    /// needs api 1.8, the rgb a vanilla map draws the block in, 0 for air
    pub fn mapColor(block: i32) -> u32
    {
        expectInt(call(RegistryRequest::BlockMapColor { block })).map_or(0, |color| color as u32)
    }

    pub fn blockInfo(block: i32) -> BlockInfo
    {
        match call(RegistryRequest::BlockInfo { block })
        {
            HostReply::BlockInfo(Some(info)) => info,
            _ => BlockInfo::default(),
        }
    }

    pub fn meta(self, pos: BlockPos) -> i32
    {
        expectInt(call(WorldRequest::GetMeta { world: self.dimension, pos })).unwrap_or(0)
    }

    /// from api 1.47 the block may be a name
    pub fn setBlock(self, pos: BlockPos, block: impl Into<crate::BlockRef>, meta: i32) -> Result<bool>
    {
        let block: i32 = block.into().id()?;
        expectBool(call(WorldRequest::SetBlock { world: self.dimension, pos, block, meta, notify: true }))
    }

    /// needs api 1.55, places a structure template of any mod with its first corner at origin, turned a quarter turn
    /// clockwise rotation times, chests rolled from seed, authority only; true when it changed a block
    pub fn placeTemplate(
        self,
        template: impl crate::IntoName,
        origin: BlockPos,
        rotation: i32,
        seed: i64,
    ) -> Result<bool>
    {
        let template: mod_protocol::Identifier = template.intoName()?;
        crate::expectBool(call(WorldgenRequest::PlaceTemplate
        {
            world: self.dimension,
            template,
            origin,
            rotation,
            seed,
        }))
    }

    /// needs api 1.53, an event the block here hears through BlockBehavior::onBlockEvent on the server and on every
    /// client near it, as pistons and note blocks do, authority only
    pub fn blockEvent(self, pos: BlockPos, kind: i32, data: i32) -> Result<()>
    {
        expectUnit(call(WorldRequest::BlockEvent { world: self.dimension, pos, kind, data }))
    }

    pub fn setBlockQuiet(self, pos: BlockPos, block: impl Into<crate::BlockRef>, meta: i32) -> Result<bool>
    {
        let block: i32 = block.into().id()?;
        expectBool(call(WorldRequest::SetBlock { world: self.dimension, pos, block, meta, notify: false }))
    }

    /// needs api 1.11, a block change that keeps the tile entity with its inventory and open screens, as a furnace
    /// swaps its idle and lit blocks
    pub fn swapBlock(self, pos: BlockPos, block: impl Into<crate::BlockRef>, meta: i32) -> Result<bool>
    {
        let block: i32 = block.into().id()?;
        expectBool(call(WorldRequest::SwapBlock { world: self.dimension, pos, block, meta }))
    }

    /// needs api 1.16, turns one column of a mod dimension to one of its biome source's biomes, kept and synced
    pub fn setBiome(self, x: i32, z: i32, biome: &str) -> Result<()>
    {
        let biome: Identifier = Identifier::parse(biome).map_err(|error| Error(format!("biome {biome}: {error:?}")))?;
        expectUnit(call(WorldgenRequest::SetBiome { world: self.dimension, x, z, biome }))
    }

    /// needs api 1.12, rain for some ticks and thunder with it when asked, or clear skies when raining is false
    pub fn setWeather(self, raining: bool, thundering: bool, ticks: i32) -> Result<()>
    {
        expectUnit(call(WorldRequest::Weather { world: self.dimension, raining, thundering, ticks }))
    }

    /// the block at a spot by name with its named properties, compare these instead of numeric ids
    pub fn blockState(self, pos: BlockPos) -> Option<StateData>
    {
        crate::expectState(call(WorldRequest::BlockState { world: self.dimension, pos })).ok().flatten()
    }

    /// needs api 1.39, the named state here from the per epoch cache, cheap enough for ticks and renders
    pub fn stateAt(self, pos: BlockPos) -> Option<Rc<StateData>> { crate::stateOf(self.block(pos), self.meta(pos)) }

    /// from api 1.38 another state of the block already there changes only its metadata and keeps its tile entity
    pub fn setBlockState(self, pos: BlockPos, state: StateData) -> Result<bool>
    {
        expectBool(call(WorldRequest::SetBlockState { world: self.dimension, pos, state, notify: true }))
    }

    pub fn lightning(self, pos: [f64; 3]) -> Result<()>
    {
        expectUnit(call(WorldRequest::Lightning { world: self.dimension, pos }))
    }

    /// the block's usual drops at a spot, as a harvest callback dropping them twice needs
    pub fn dropBlockItems(self, pos: BlockPos, block: impl Into<crate::BlockRef>, meta: i32) -> Result<()>
    {
        let block: i32 = block.into().id()?;
        expectUnit(call(WorldRequest::DropBlockItems { world: self.dimension, pos, block, meta }))
    }

    pub fn breakEffect(self, pos: BlockPos, block: impl Into<crate::BlockRef>, meta: i32) -> Result<()>
    {
        let block: i32 = block.into().id()?;
        expectUnit(call(WorldRequest::BlockEffect { world: self.dimension, pos, block, meta }))
    }

    pub fn setMeta(self, pos: BlockPos, meta: i32) -> Result<()>
    {
        expectUnit(call(WorldRequest::SetMeta { world: self.dimension, pos, meta }))
    }

    pub fn notifyNeighbors(self, pos: BlockPos, block: impl Into<crate::BlockRef>) -> Result<()>
    {
        let block: i32 = block.into().id()?;
        expectUnit(call(WorldRequest::NotifyNeighbors { world: self.dimension, pos, block }))
    }

    /// needs api 1.47, tells the neighbors the block now at pos changed, as a placed or broken block does
    pub fn notifyAround(self, pos: BlockPos) -> Result<()> { self.notifyNeighbors(pos, self.block(pos)) }

    pub fn scheduleTick(self, pos: BlockPos, block: impl Into<crate::BlockRef>, delay: i32) -> Result<()>
    {
        let block: i32 = block.into().id()?;
        expectUnit(call(WorldRequest::ScheduleTick { world: self.dimension, pos, block, delay }))
    }

    pub fn markBlockForRender(self, pos: BlockPos) { fire(WorldRequest::MarkForRender { world: self.dimension, pos }); }

    pub fn light(self, pos: BlockPos) -> i32
    {
        expectInt(call(WorldRequest::Light { world: self.dimension, pos })).unwrap_or(0)
    }

    pub fn isPowered(self, pos: BlockPos) -> bool
    {
        expectBool(call(WorldRequest::IsPowered { world: self.dimension, pos })).unwrap_or(false)
    }

    /// the time this dimension's sky shows on this side, None follows the world time
    pub fn setSkyTime(self, time: Option<i64>) { fire(WorldRequest::SetSkyTime { world: self.dimension, time }); }

    pub fn time(self) -> i64 { expectLong(call(WorldRequest::WorldTime { world: self.dimension })).unwrap_or(0) }

    /// draws from the world random so a mod stays in step with vanilla randomness on the authority
    pub fn random(self, bound: i32) -> i32
    {
        expectInt(call(WorldRequest::WorldRandom { world: self.dimension, bound })).unwrap_or(0)
    }

    pub fn chance(self, probability: f64) -> bool
    {
        let resolution: i32 = 1000000;
        (self.random(resolution) as f64) < probability * resolution as f64
    }

    /// needs api 1.7, a mob spawner of this entity at the spot, vanilla names such as Spider or mod identifiers
    pub fn setSpawner(self, pos: BlockPos, entity: &str) -> Result<()>
    {
        expectUnit(call(WorldRequest::SetSpawner { world: self.dimension, pos, entity: entity.to_owned() }))
    }

    /// the block's random tick right now, if it ticks randomly at all
    pub fn randomTick(self, pos: BlockPos) -> Result<()>
    {
        expectUnit(call(WorldRequest::RandomTick { world: self.dimension, pos }))
    }

    /// bone meal applied to the block
    pub fn fertilize(self, pos: BlockPos) -> Result<()>
    {
        expectUnit(call(WorldRequest::Fertilize { world: self.dimension, pos }))
    }

    /// needs api 1.7, the seed the world was made with
    pub fn seed(self) -> i64 { expectLong(call(WorldgenRequest::WorldSeed { world: self.dimension })).unwrap_or(0) }

    /// needs api 1.10, records a structure this mod built here so every mod can find it, on the owning side only
    pub fn addStructure(self, start: StructureStart) -> Result<()>
    {
        expectUnit(call(WorldgenRequest::AddStructure { world: self.dimension, start }))
    }

    /// needs api 1.10, the recorded structures crossing a box, each with only the pieces that cross it, of one kind
    /// when named
    pub fn structures(self, bounds: StructureBox, kind: Option<&Identifier>) -> Result<Vec<StructureStart>>
    {
        let StructureBox { m_min: min, m_max: max } = bounds;

        expectStarts(call(WorldgenRequest::FindStructures
        {
            world: self.dimension,
            min,
            max,
            structure: kind.cloned(),
        }))
    }

    /// needs api 1.10, the recorded structures whose pieces hold the block
    pub fn structuresAt(self, pos: BlockPos) -> Result<Vec<StructureStart>>
    {
        let at: [i32; 3] = [pos.x, pos.y, pos.z];
        self.structures(StructureBox::new(at, at), None)
    }

    /// needs api 1.10, the nearest structure of a kind within radius chunks of a spot
    pub fn locate(self, kind: &Identifier, from: BlockPos, radius: i32) -> Result<Option<Located>>
    {
        expectLocated(call(WorldgenRequest::LocateStructure
        {
            world: self.dimension,
            structure: kind.clone(),
            from,
            radius,
        }))
    }

    pub fn isAuthority(self) -> bool
    {
        expectBool(call(WorldRequest::IsAuthority { world: self.dimension })).unwrap_or(false)
    }

    pub fn spawnItem(self, pos: [f64; 3], stack: ItemStack) -> Result<()>
    {
        expectUnit(call(EntityRequest::SpawnItem { world: self.dimension, pos, stack: stack.raw()? }))
    }

    pub fn spawnEntity(self, kind: &str, pos: [f64; 3]) -> Result<Entity>
    {
        let kind: Identifier = Identifier::parse(kind)?;
        let id: i32 =
            expectInt(call(EntityRequest::SpawnEntity { world: self.dimension, kind, pos, data: Vec::new() }))?;
        Ok(Entity { world: self, id })
    }

    pub fn entity(self, id: i32) -> Option<EntityInfo>
    {
        expectEntity(call(EntityRequest::Entity { world: self.dimension, id }))
    }

    pub fn entitiesNear(self, pos: [f64; 3], radius: f64) -> Vec<EntityInfo>
    {
        expectEntities(call(EntityRequest::EntitiesNear { world: self.dimension, pos, radius }))
    }

    pub fn players(self) -> Vec<Player>
    {
        expectEntities(call(PlayerRequest::Players { world: Some(self.dimension) }))
            .into_iter()
            .map(|info| Player { id: info.id })
            .collect()
    }

    pub fn playSound(self, pos: [f64; 3], sound: &str, volume: f32, pitch: f32)
    {
        fire(WorldRequest::PlaySound { world: self.dimension, pos, sound: sound.to_owned(), volume, pitch });
    }

    pub fn particle(self, name: &str, pos: [f64; 3], motion: [f64; 3])
    {
        fire(WorldRequest::Particle { world: self.dimension, name: name.to_owned(), pos, motion });
    }

    pub fn explode(self, pos: [f64; 3], power: f32) -> Result<()>
    {
        expectUnit(call(WorldRequest::Explode { world: self.dimension, pos, power }))
    }

    /// the mod data of a tile entity, any serde type, stored in the world save
    pub fn tileData<T: DeserializeOwned + Default>(self, pos: BlockPos) -> T
    {
        fetchData(call(StoredRequest::TileData { world: self.dimension, pos }))
    }

    pub fn setTileData<T: Serialize>(self, pos: BlockPos, value: &T) -> Result<()>
    {
        expectUnit(call(StoredRequest::SetTileData { world: self.dimension, pos, data: encodeStored(value) }))
    }

    pub fn tileSlot(self, pos: BlockPos, slot: i32) -> Option<ItemStack>
    {
        expectStack(call(InventoryRequest::TileSlot { world: self.dimension, pos, slot }))
            .ok()
            .flatten()
            .map(ItemStack::fromRaw)
    }

    pub fn setTileSlot(self, pos: BlockPos, slot: i32, stack: Option<ItemStack>) -> Result<()>
    {
        let stack: Option<StackData> = stack.map(|stack| stack.raw()).transpose()?;
        expectUnit(call(InventoryRequest::SetTileSlot { world: self.dimension, pos, slot, stack }))
    }
}

/// a data component on a stack in place, readable for any component and writable for this mod's own namespace
pub fn stackComponent<T: DeserializeOwned + Default>(at: StackRef, component: &str) -> T
{
    let Ok(component) = Identifier::parse(component)
    else
    {
        return T::default();
    };

    fetchData(call(StoredRequest::StackComponent { at, component }))
}

/// an empty value removes the component, stacks merge only when their components match
pub fn setStackComponent<T: Serialize>(at: StackRef, component: &str, value: &T) -> Result<()>
{
    let component: Identifier = Identifier::parse(component)?;
    expectUnit(call(StoredRequest::SetStackComponent { at, component, data: encodeStored(value) }))
}

pub fn worldData<T: DeserializeOwned + Default>(key: &str) -> T
{
    fetchData(call(StoredRequest::WorldData { key: key.to_owned() }))
}

/// needs api 1.8, the pixels of a texture drawn at runtime, argb rows from the top left; Draw images and model ops
/// that name it draw these pixels from then on, a bare name is taken in this mod's namespace
pub fn setDynamicTexture(name: &str, width: u32, height: u32, pixels: Vec<u32>) -> crate::Result<()>
{
    let name: Identifier = crate::ownIdentifier(name)?;
    expectUnit(call(ScreenRequest::SetDynamicTexture { name, width, height, pixels }))
}

pub fn setWorldData<T: Serialize>(key: &str, value: &T)
{
    fire(StoredRequest::SetWorldData { key: key.to_owned(), data: encodeStored(value) });
}

/// every stack a creative item list shows, subtypes included, in numeric id order, each a full stack
/// needs api 1.33, a value a mod keeps outside any world, shared by every save of this game or server folder
pub fn sharedData<T: DeserializeOwned + Default>(key: &str) -> T
{
    fetchData(call(StoredRequest::SharedData { key: key.to_owned() }))
}

/// needs api 1.33, replaces a shared value, saved at once
pub fn setSharedData<T: Serialize>(key: &str, value: &T) -> Result<()>
{
    expectUnit(call(StoredRequest::SetSharedData { key: key.to_owned(), data: encodeStored(value) }))
}

/// needs api 1.33, redraws a sprite from square argb pixels made in code, as a TextureFX does each tick; client only
pub fn paintSprite(texture: &str, size: u32, pixels: Vec<u32>) -> Result<()>
{
    let texture: Identifier = Identifier::parse(texture).map_err(|error| Error(format!("bad sprite name: {error}")))?;
    expectUnit(call(ScreenRequest::PaintSprite { texture, size, pixels }))
}

/// needs api 1.35, the square argb pixels and size of a sprite as the active texture packs draw it, the base a
/// TextureFX copies before painting; None on a server or for a texture no sheet holds
pub fn spritePixels(texture: &str) -> Option<(u32, Vec<u32>)>
{
    let texture: Identifier = Identifier::parse(texture).ok()?;

    match call(ScreenRequest::SpritePixels { texture })
    {
        HostReply::Pixels { size, pixels } if size > 0 => Some((size, pixels)),
        _ => None,
    }
}

/// needs api 1.33, what the crafting table makes of a 3 by 3 grid given in row order
pub fn craftResult(grid: &[Option<ItemStack>; 9]) -> Option<ItemStack>
{
    let grid: Vec<Option<StackData>> =
        grid.iter().map(|slot| slot.as_ref().and_then(|stack| stack.raw().ok())).collect();
    expectStack(call(InventoryRequest::CraftResult { grid })).ok().flatten().map(ItemStack::fromRaw)
}

/// needs api 1.33, the member a stack turns into when several mods add the same material under a shared c: tag
pub fn unified(stack: &ItemStack) -> ItemStack
{
    let Ok(raw) = stack.raw()
    else
    {
        return stack.clone();
    };

    expectStack(call(RegistryRequest::Unified { stack: raw }))
        .ok()
        .flatten()
        .map_or_else(|| stack.clone(), ItemStack::fromRaw)
}

/// needs api 1.33, the stack a crafting or burning use of an item leaves behind, as an empty bucket
pub fn containerItem(stack: &ItemStack) -> Option<ItemStack>
{
    let stack: StackData = stack.raw().ok()?;
    expectStack(call(RegistryRequest::ContainerItem { stack })).ok().flatten().map(ItemStack::fromRaw)
}

/// needs api 1.33, how many ticks a stack burns in a furnace, mod fuels included, 0 for none
pub fn fuelTicks(stack: &ItemStack) -> i32
{
    let Ok(stack) = stack.raw()
    else
    {
        return 0;
    };

    expectInt(call(RegistryRequest::FuelTicks { stack })).unwrap_or(0)
}

pub fn itemCatalog() -> Vec<ItemStack>
{
    match call(ViewerRequest::ItemCatalog)
    {
        HostReply::Stacks(stacks) => stacks.into_iter().map(ItemStack::fromRaw).collect(),
        _ => Vec::new(),
    }
}

/// needs api 1.31, every registered living entity a mob spawner can make, vanilla and mod, in numeric id order
pub fn mobList() -> Vec<Identifier>
{
    match call(EntityRequest::MobList)
    {
        HostReply::Ids(ids) => ids,
        _ => Vec::new(),
    }
}

/// the crafting and smelting recipes the game knows, mod recipes included
pub fn recipeList() -> Vec<RecipeView>
{
    match call(ViewerRequest::RecipeList)
    {
        HostReply::Recipes(recipes) => recipes,
        _ => Vec::new(),
    }
}

/// needs api 1.33, every drop table a recipe viewer shows: beta's mob, block and dungeon drops and every mod's loot
pub fn lootList() -> Vec<LootView>
{
    match call(ViewerRequest::LootList)
    {
        HostReply::Loot(tables) => tables,
        _ => Vec::new(),
    }
}

/// needs api 1.35, every machine recipe viewers show with its workstations and recipes
pub fn machineList() -> Vec<MachineView>
{
    match call(ViewerRequest::MachineList)
    {
        HostReply::Machines(machines) => machines,
        _ => Vec::new(),
    }
}

/// needs api 1.33, the stacks a chest loot table rolls with a random seeded from seed, so worldgen stays repeatable
pub fn rollLoot(table: &Identifier, seed: i64) -> Vec<ItemStack>
{
    match call(InventoryRequest::RollLoot { table: table.clone(), seed })
    {
        HostReply::Stacks(stacks) => stacks.into_iter().map(ItemStack::fromRaw).collect(),
        _ => Vec::new(),
    }
}

/// changes whenever ids are remapped or recipe tables rebuilt, so caches of the catalog or recipe list can reload
pub fn contentGeneration() -> i32 { expectInt(call(ViewerRequest::ContentGeneration)).unwrap_or(0) }

/// needs api 1.12, the terrain atlas slot of a block texture for the texture hooks, None when it has none; call it
/// once while registering for any texture no block spec names, so the atlas makes room for it
pub fn terrainSlot(texture: &str) -> Option<i32>
{
    expectInt(call(ScreenRequest::TerrainSlot { texture: texture.to_owned() })).ok().filter(|slot| *slot >= 0)
}

/// needs api 1.13, cuts a terrain texture into columns by rows tiles that each become a sprite of their own; call it
/// while registering, then find each tile's slot with terrainSlot(tileName(texture, column, row))
pub fn terrainTiles(texture: &str, columns: i32, rows: i32) -> crate::Result<()>
{
    expectUnit(call(ScreenRequest::TerrainTiles { texture: texture.to_owned(), columns, rows }))
}

/// the texture name of one tile terrainTiles cut, counted from the left and the top
pub fn tileName(texture: &str, column: i32, row: i32) -> String
{
    let (namespace, path): (&str, &str) = texture.split_once(':').unwrap_or(("minecraft", texture));
    format!("loader:tiles/{namespace}/{path}/{column}_{row}")
}

/// needs api 1.23, the item sheet slot of a texture for the item icon hooks, None when it has none; call it while
/// registering for any texture no item spec names, so the sheet makes room for it
pub fn itemSlot(texture: &str) -> Option<i32>
{
    expectInt(call(ScreenRequest::ItemSlot { texture: texture.to_owned() })).ok().filter(|slot| *slot >= 0)
}

/// the client font's width of a string, zero on a server
pub fn textWidth(text: &str) -> i32 { expectInt(call(ScreenRequest::TextWidth { text: text.to_owned() })).unwrap_or(0) }

/// the scaled gui size of the client screen, none on a server
pub fn screenSize() -> Option<[i32; 2]>
{
    match call(ScreenRequest::ScreenSize)
    {
        HostReply::Size(width, height) => Some([width, height]),
        _ => None,
    }
}

/// needs api 2.1, the client clipboard as text, none on a server or when it holds no text
pub fn clipboard() -> Option<String>
{
    match call(ScreenRequest::Clipboard)
    {
        HostReply::Text(text) => Some(text),
        _ => None,
    }
}

pub fn localPlayer() -> Option<Player>
{
    expectEntity(call(PlayerRequest::LocalPlayer)).map(|info| Player { id: info.id })
}

pub fn isClient() -> bool { expectBool(call(MetaRequest::IsClient)).unwrap_or(false) }

pub fn isLoaded(modid: &str) -> bool
{
    expectBool(call(MetaRequest::IsLoaded { modid: modid.to_owned() })).unwrap_or(false)
}

/// needs api 1.4, the name a mod's manifest gives, Minecraft for the vanilla namespace and empty when not loaded
pub fn modName(modid: &str) -> String
{
    expectText(call(MetaRequest::ModName { modid: modid.to_owned() })).unwrap_or_default()
}

pub fn translate(key: &str) -> String
{
    expectText(call(MetaRequest::Translate { key: key.to_owned() })).unwrap_or_else(|_| key.to_owned())
}

fn colorizer(foliage: bool, temperature: f64, rainfall: f64) -> u32
{
    const WHITE: u32 = 0xFFFFFF;

    expectInt(call(WorldgenRequest::Colorizer { foliage, temperature, rainfall }))
        .map_or(WHITE, |color| color as u32 & WHITE)
}

/// needs api 1.7, ColorizerGrass at a temperature and rainfall clamped to 0 to 1
pub fn grassColor(temperature: f64, rainfall: f64) -> u32 { colorizer(false, temperature, rainfall) }

/// needs api 1.7, ColorizerFoliage at a temperature and rainfall clamped to 0 to 1
pub fn foliageColor(temperature: f64, rainfall: f64) -> u32 { colorizer(true, temperature, rainfall) }

/// needs api 1.17, the damage type of the hit an attack, hurt or onHurt call is handling
pub fn damageType() -> Option<Identifier> { expectId(call(EntityRequest::DamageType)) }
