use crate::rt::call;
use crate::terrain::structure::StructureBox;
use crate::
{
    Error, ItemStack, Result, expectBool, expectBytes, expectEntities, expectEntity, expectInt, expectLocated,
    expectStack, expectStarts, expectUnit,
};
use mod_protocol::Identifier;
use mod_protocol::abi::
{
    BlockCtx as RawCtx, BlockInfo, BlockPos, EntityField, EntityInfo, HostCall, HostReply, Located, ModifierOperation,
    RayHit, RecipeView, StackRef, StateData, StructureStart, Target,
};
use serde::Serialize;
use serde::de::DeserializeOwned;

pub fn encode<T: Serialize>(value: &T) -> Vec<u8> { mod_protocol::wire::encode(value) }

pub fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T>
{
    mod_protocol::wire::decode(bytes).map_err(|error| Error(error.to_string()))
}

// a loaded world, named by its dimension id
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct World
{
    pub dimension: i32,
}

impl World
{
    pub const OVERWORLD: World = World { dimension: 0 };
    pub const NETHER: World = World { dimension: -1 };

    // needs api 1.26, the count that did not fit; simulate only reports
    pub fn insertItem(self, pos: BlockPos, face: i32, stack: ItemStack, simulate: bool) -> Result<i32>
    {
        let stack: mod_protocol::abi::StackData = stack.raw()?;
        match call(HostCall::InsertItem { world: self.dimension, pos, face, stack, simulate })
        {
            HostReply::Int(left) => Ok(left),
            HostReply::Error(error) => Err(Error(error)),
            other => Err(Error(format!("unexpected reply {other:?}"))),
        }
    }

    // needs api 1.26, simulate leaves the items where they are
    pub fn extractItem(self, pos: BlockPos, face: i32, max: i32, simulate: bool) -> Option<ItemStack>
    {
        expectStack(call(HostCall::ExtractItem { world: self.dimension, pos, face, max, simulate }))
            .ok()
            .flatten()
            .map(ItemStack::fromRaw)
    }

    pub fn block(self, pos: BlockPos) -> i32
    {
        expectInt(call(HostCall::GetBlock { world: self.dimension, pos })).unwrap_or(0)
    }

    pub fn height(self, x: i32, z: i32) -> i32
    {
        expectInt(call(HostCall::Height { world: self.dimension, x, z })).unwrap_or(0)
    }

    pub fn isAir(self, pos: BlockPos) -> bool { self.block(pos) == 0 }

    pub fn entitiesAlong(self, from: [f64; 3], to: [f64; 3], radius: f64, exclude: Option<i32>) -> Vec<EntityInfo>
    {
        match call(HostCall::EntitiesAlong { world: self.dimension, from, to, radius, exclude })
        {
            HostReply::Entities(found) => found,
            _ => Vec::new(),
        }
    }

    pub fn rayTrace(self, from: [f64; 3], to: [f64; 3], liquids: bool) -> Option<RayHit>
    {
        match call(HostCall::RayTrace { world: self.dimension, from, to, liquids })
        {
            HostReply::Hit(hit) => hit,
            _ => None,
        }
    }

    // needs api 1.8, the rgb a vanilla map draws the block in, 0 for air
    pub fn mapColor(block: i32) -> u32
    {
        match call(HostCall::BlockMapColor { block })
        {
            HostReply::Int(color) => color as u32,
            _ => 0,
        }
    }

    pub fn blockInfo(block: i32) -> BlockInfo
    {
        match call(HostCall::BlockInfo { block })
        {
            HostReply::BlockInfo(Some(info)) => info,
            _ => BlockInfo::default(),
        }
    }

    pub fn meta(self, pos: BlockPos) -> i32
    {
        expectInt(call(HostCall::GetMeta { world: self.dimension, pos })).unwrap_or(0)
    }

    pub fn setBlock(self, pos: BlockPos, block: i32, meta: i32) -> Result<bool>
    {
        expectBool(call(HostCall::SetBlock { world: self.dimension, pos, block, meta, notify: true }))
    }

    pub fn setBlockQuiet(self, pos: BlockPos, block: i32, meta: i32) -> Result<bool>
    {
        expectBool(call(HostCall::SetBlock { world: self.dimension, pos, block, meta, notify: false }))
    }

    // needs api 1.11, a block change that keeps the tile entity with its inventory and open screens, as a furnace
    // swaps its idle and lit blocks
    pub fn swapBlock(self, pos: BlockPos, block: i32, meta: i32) -> Result<bool>
    {
        expectBool(call(HostCall::SwapBlock { world: self.dimension, pos, block, meta }))
    }

    // needs api 1.16, turns one column of a mod dimension to one of its biome source's biomes, kept and synced
    pub fn setBiome(self, x: i32, z: i32, biome: &str) -> Result<()>
    {
        let biome: Identifier = Identifier::parse(biome).map_err(|error| Error(format!("biome {biome}: {error:?}")))?;
        expectUnit(call(HostCall::SetBiome { world: self.dimension, x, z, biome }))
    }

    // needs api 1.12, rain for some ticks and thunder with it when asked, or clear skies when raining is false
    pub fn setWeather(self, raining: bool, thundering: bool, ticks: i32) -> Result<()>
    {
        expectUnit(call(HostCall::Weather { world: self.dimension, raining, thundering, ticks }))
    }

    // the block at a spot by name with its named properties, compare these instead of numeric ids
    pub fn blockState(self, pos: BlockPos) -> Option<StateData>
    {
        crate::expectState(call(HostCall::BlockState { world: self.dimension, pos })).ok().flatten()
    }

    pub fn setBlockState(self, pos: BlockPos, state: StateData) -> Result<bool>
    {
        expectBool(call(HostCall::SetBlockState { world: self.dimension, pos, state, notify: true }))
    }

    pub fn lightning(self, pos: [f64; 3]) -> Result<()>
    {
        expectUnit(call(HostCall::Lightning { world: self.dimension, pos }))
    }

    // the block's usual drops at a spot, as a harvest callback dropping them twice needs
    pub fn dropBlockItems(self, pos: BlockPos, block: i32, meta: i32) -> Result<()>
    {
        expectUnit(call(HostCall::DropBlockItems { world: self.dimension, pos, block, meta }))
    }

    pub fn breakEffect(self, pos: BlockPos, block: i32, meta: i32) -> Result<()>
    {
        expectUnit(call(HostCall::BlockEffect { world: self.dimension, pos, block, meta }))
    }

    pub fn setMeta(self, pos: BlockPos, meta: i32) -> Result<()>
    {
        expectUnit(call(HostCall::SetMeta { world: self.dimension, pos, meta }))
    }

    pub fn notifyNeighbors(self, pos: BlockPos, block: i32) -> Result<()>
    {
        expectUnit(call(HostCall::NotifyNeighbors { world: self.dimension, pos, block }))
    }

    pub fn scheduleTick(self, pos: BlockPos, block: i32, delay: i32) -> Result<()>
    {
        expectUnit(call(HostCall::ScheduleTick { world: self.dimension, pos, block, delay }))
    }

    pub fn markBlockForRender(self, pos: BlockPos)
    {
        let _ = call(HostCall::MarkForRender { world: self.dimension, pos });
    }

    pub fn light(self, pos: BlockPos) -> i32
    {
        expectInt(call(HostCall::Light { world: self.dimension, pos })).unwrap_or(0)
    }

    pub fn isPowered(self, pos: BlockPos) -> bool
    {
        expectBool(call(HostCall::IsPowered { world: self.dimension, pos })).unwrap_or(false)
    }

    // the time this dimension's sky shows on this side, None follows the world time
    pub fn setSkyTime(self, time: Option<i64>) { let _ = call(HostCall::SetSkyTime { world: self.dimension, time }); }

    pub fn time(self) -> i64
    {
        match call(HostCall::WorldTime { world: self.dimension })
        {
            HostReply::Long(time) => time,
            _ => 0,
        }
    }

    // draws from the world random so a mod stays in step with vanilla randomness on the authority
    pub fn random(self, bound: i32) -> i32
    {
        expectInt(call(HostCall::WorldRandom { world: self.dimension, bound })).unwrap_or(0)
    }

    pub fn chance(self, probability: f64) -> bool { (self.random(1_000_000) as f64) < probability * 1_000_000.0 }

    // needs api 1.7, a mob spawner of this entity at the spot, vanilla names such as Spider or mod identifiers
    pub fn setSpawner(self, pos: BlockPos, entity: &str) -> Result<()>
    {
        expectUnit(call(HostCall::SetSpawner { world: self.dimension, pos, entity: entity.to_owned() }))
    }

    // the block's random tick right now, if it ticks randomly at all
    pub fn randomTick(self, pos: BlockPos) -> Result<()>
    {
        expectUnit(call(HostCall::RandomTick { world: self.dimension, pos }))
    }

    // bone meal applied to the block
    pub fn fertilize(self, pos: BlockPos) -> Result<()>
    {
        expectUnit(call(HostCall::Fertilize { world: self.dimension, pos }))
    }

    // needs api 1.7, the seed the world was made with
    pub fn seed(self) -> i64
    {
        match call(HostCall::WorldSeed { world: self.dimension })
        {
            HostReply::Long(seed) => seed,
            _ => 0,
        }
    }

    // needs api 1.10, records a structure this mod built here so every mod can find it, on the owning side only
    pub fn addStructure(self, start: StructureStart) -> Result<()>
    {
        expectUnit(call(HostCall::AddStructure { world: self.dimension, start }))
    }

    // needs api 1.10, the recorded structures crossing a box, each with only the pieces that cross it, of one kind
    // when named
    pub fn structures(self, bounds: StructureBox, kind: Option<&Identifier>) -> Result<Vec<StructureStart>>
    {
        let StructureBox { m_min: min, m_max: max } = bounds;
        expectStarts(call(HostCall::FindStructures { world: self.dimension, min, max, structure: kind.cloned() }))
    }

    // needs api 1.10, the recorded structures whose pieces hold the block
    pub fn structuresAt(self, pos: BlockPos) -> Result<Vec<StructureStart>>
    {
        let at: [i32; 3] = [pos.x, pos.y, pos.z];
        self.structures(StructureBox::new(at, at), None)
    }

    // needs api 1.10, the nearest structure of a kind within radius chunks of a spot
    pub fn locate(self, kind: &Identifier, from: BlockPos, radius: i32) -> Result<Option<Located>>
    {
        expectLocated(call(HostCall::LocateStructure { world: self.dimension, structure: kind.clone(), from, radius }))
    }

    pub fn isAuthority(self) -> bool
    {
        expectBool(call(HostCall::IsAuthority { world: self.dimension })).unwrap_or(false)
    }

    pub fn spawnItem(self, pos: [f64; 3], stack: ItemStack) -> Result<()>
    {
        expectUnit(call(HostCall::SpawnItem { world: self.dimension, pos, stack: stack.raw()? }))
    }

    pub fn spawnEntity(self, kind: &str, pos: [f64; 3]) -> Result<Entity>
    {
        let kind: Identifier = Identifier::parse(kind).map_err(|error| Error(error.to_string()))?;
        let id: i32 = expectInt(call(HostCall::SpawnEntity { world: self.dimension, kind, pos, data: Vec::new() }))?;
        Ok(Entity { world: self, id })
    }

    pub fn entity(self, id: i32) -> Option<EntityInfo>
    {
        expectEntity(call(HostCall::Entity { world: self.dimension, id }))
    }

    pub fn entitiesNear(self, pos: [f64; 3], radius: f64) -> Vec<EntityInfo>
    {
        expectEntities(call(HostCall::EntitiesNear { world: self.dimension, pos, radius }))
    }

    pub fn players(self) -> Vec<Player>
    {
        expectEntities(call(HostCall::Players { world: Some(self.dimension) }))
            .into_iter()
            .map(|info| Player { id: info.id })
            .collect()
    }

    pub fn playSound(self, pos: [f64; 3], sound: &str, volume: f32, pitch: f32)
    {
        let _ = call(HostCall::PlaySound { world: self.dimension, pos, sound: sound.to_owned(), volume, pitch });
    }

    pub fn particle(self, name: &str, pos: [f64; 3], motion: [f64; 3])
    {
        let _ = call(HostCall::Particle { world: self.dimension, name: name.to_owned(), pos, motion });
    }

    pub fn explode(self, pos: [f64; 3], power: f32) -> Result<()>
    {
        expectUnit(call(HostCall::Explode { world: self.dimension, pos, power }))
    }

    // the mod data of a tile entity, any serde type, stored in the world save
    pub fn tileData<T: DeserializeOwned + Default>(self, pos: BlockPos) -> T
    {
        expectBytes(call(HostCall::TileData { world: self.dimension, pos }))
            .ok()
            .filter(|bytes| !bytes.is_empty())
            .and_then(|bytes| decode(&bytes).ok())
            .unwrap_or_default()
    }

    pub fn setTileData<T: Serialize>(self, pos: BlockPos, value: &T) -> Result<()>
    {
        expectUnit(call(HostCall::SetTileData { world: self.dimension, pos, data: encode(value) }))
    }

    // this mod's own data on any tile entity, a vanilla chest or another mod's machine, saved with it
    pub fn tileAttachment<T: DeserializeOwned + Default>(self, pos: BlockPos) -> T
    {
        expectBytes(call(HostCall::TileAttachment { world: self.dimension, pos }))
            .ok()
            .filter(|bytes| !bytes.is_empty())
            .and_then(|bytes| decode(&bytes).ok())
            .unwrap_or_default()
    }

    pub fn setTileAttachment<T: Serialize>(self, pos: BlockPos, value: &T) -> Result<()>
    {
        expectUnit(call(HostCall::SetTileAttachment { world: self.dimension, pos, data: encode(value) }))
    }

    // this mod's own data on a chunk, saved with it
    pub fn chunkAttachment<T: DeserializeOwned + Default>(self, chunkX: i32, chunkZ: i32) -> T
    {
        expectBytes(call(HostCall::ChunkAttachment { world: self.dimension, chunk_x: chunkX, chunk_z: chunkZ }))
            .ok()
            .filter(|bytes| !bytes.is_empty())
            .and_then(|bytes| decode(&bytes).ok())
            .unwrap_or_default()
    }

    pub fn setChunkAttachment<T: Serialize>(self, chunkX: i32, chunkZ: i32, value: &T) -> Result<()>
    {
        let data: Vec<u8> = encode(value);
        expectUnit(call(HostCall::SetChunkAttachment { world: self.dimension, chunk_x: chunkX, chunk_z: chunkZ, data }))
    }

    pub fn tileSlot(self, pos: BlockPos, slot: i32) -> Option<ItemStack>
    {
        expectStack(call(HostCall::TileSlot { world: self.dimension, pos, slot }))
            .ok()
            .flatten()
            .map(ItemStack::fromRaw)
    }

    pub fn setTileSlot(self, pos: BlockPos, slot: i32, stack: Option<ItemStack>) -> Result<()>
    {
        let stack = stack.map(|stack| stack.raw()).transpose()?;
        expectUnit(call(HostCall::SetTileSlot { world: self.dimension, pos, slot, stack }))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Player
{
    pub id: i32,
}

// the name a stack shows in its own language, vanilla items included
pub fn displayName(stack: &ItemStack) -> String
{
    match stack.raw().map(|stack| call(HostCall::DisplayName { stack }))
    {
        Ok(HostReply::Text(name)) => name,
        _ => String::new(),
    }
}

// the time of every loaded world
pub fn setWorldTime(time: i64) { let _ = call(HostCall::SetWorldTime { time }); }

impl Player
{
    pub fn isOp(self) -> bool { expectBool(call(HostCall::IsOp { player: self.id })).unwrap_or(false) }

    // a slot of the container screen the player has open, by container slot number
    pub fn openSlot(self, slot: i32) -> Option<ItemStack>
    {
        expectStack(call(HostCall::OpenSlot { player: self.id, slot })).ok().flatten().map(ItemStack::fromRaw)
    }

    pub fn chat(self, message: &str)
    {
        let _ = call(HostCall::Chat { player: Some(self.id), message: message.to_owned() });
    }

    pub fn give(self, stack: ItemStack) -> Result<()>
    {
        expectUnit(call(HostCall::Give { player: self.id, stack: stack.raw()? }))
    }

    // needs api 1.20, holds its current item in use for ticks, the item's onUsingTick, onStoppedUsing and onUseFinish
    // follow; 0 lets go without telling the item
    pub fn setItemInUse(self, ticks: i32) -> Result<()>
    {
        expectUnit(call(HostCall::SetItemInUse { player: self.id, ticks }))
    }

    // needs api 1.20, lets go of the item in use as releasing the use key does
    pub fn stopUsingItem(self) -> Result<()> { expectUnit(call(HostCall::StopUsingItem { player: self.id })) }

    // needs api 1.20, ticks left on the item in use, 0 when nothing is in use
    pub fn itemInUse(self) -> i32
    {
        self.info().map_or(0, |info| {
            Entity { world: World { dimension: info.world }, id: self.id }.field(EntityField::ItemInUse) as i32
        })
    }

    // needs api 1.14, how fast it digs a block with what it holds, as EntityPlayer.getCurrentPlayerStrVsBlock
    pub fn digSpeed(self, block: i32) -> f32
    {
        match call(HostCall::DigSpeed { player: self.id, block })
        {
            HostReply::Float(speed) => speed,
            _ => 1.0,
        }
    }

    pub fn held(self) -> Option<ItemStack>
    {
        expectStack(call(HostCall::HeldStack { player: self.id })).ok().flatten().map(ItemStack::fromRaw)
    }

    pub fn setHeld(self, stack: Option<ItemStack>) -> Result<()>
    {
        let stack = stack.map(|stack| stack.raw()).transpose()?;
        expectUnit(call(HostCall::SetHeldStack { player: self.id, stack }))
    }

    // needs api 1.25, true when newly earned; as in beta the parent comes first
    pub fn unlockAchievement(self, achievement: &str) -> Result<bool>
    {
        let achievement: Identifier = Identifier::parse(achievement).map_err(|error| Error(error.to_string()))?;
        expectBool(call(HostCall::UnlockAchievement { player: self.id, achievement }))
    }

    // needs api 1.25
    pub fn hasAchievement(self, achievement: &str) -> bool
    {
        let Ok(achievement) = Identifier::parse(achievement)
        else
        {
            return false;
        };
        expectBool(call(HostCall::HasAchievement { player: self.id, achievement })).unwrap_or(false)
    }

    // needs api 1.25, stats are named such as <modid>:bosses_killed or mined/<block id>
    pub fn addStat(self, stat: &str, amount: i32) -> Result<()>
    {
        expectUnit(call(HostCall::AddStat { player: self.id, stat: stat.to_owned(), amount }))
    }

    // needs api 1.25
    pub fn stat(self, stat: &str) -> i32
    {
        match call(HostCall::StatValue { player: self.id, stat: stat.to_owned() })
        {
            HostReply::Int(value) => value,
            _ => 0,
        }
    }

    // needs api 1.7, wears the held tool as a use would and breaks it past its last use
    pub fn damageHeld(self, amount: i32) -> Result<()>
    {
        expectUnit(call(HostCall::DamageHeld { player: self.id, amount }))
    }

    pub fn teleport(self, dimension: i32, pos: [f64; 3]) -> Result<()>
    {
        expectUnit(call(HostCall::Teleport { player: self.id, dimension, pos }))
    }

    pub fn openScreen(self, screen: &str, pos: Option<BlockPos>, data: Vec<u8>) -> Result<()>
    {
        let screen: Identifier = crate::ownId(screen)?;
        expectUnit(call(HostCall::OpenScreen { player: self.id, screen, pos, data }))
    }

    pub fn closeScreen(self) { let _ = call(HostCall::CloseScreen { player: self.id }); }

    pub fn cursor(self) -> Option<ItemStack>
    {
        expectStack(call(HostCall::CursorStack { player: self.id })).ok().flatten().map(ItemStack::fromRaw)
    }

    pub fn setCursor(self, stack: Option<ItemStack>) -> Result<()>
    {
        let stack = stack.map(|stack| stack.raw()).transpose()?;
        expectUnit(call(HostCall::SetCursorStack { player: self.id, stack }))
    }

    pub fn slot(self, slot: i32) -> Option<ItemStack>
    {
        expectStack(call(HostCall::InventorySlot { player: self.id, slot })).ok().flatten().map(ItemStack::fromRaw)
    }

    pub fn setSlot(self, slot: i32, stack: Option<ItemStack>) -> Result<()>
    {
        let stack = stack.map(|stack| stack.raw()).transpose()?;
        expectUnit(call(HostCall::SetInventorySlot { player: self.id, slot, stack }))
    }

    pub fn data<T: DeserializeOwned + Default>(self, key: &str) -> T
    {
        expectBytes(call(HostCall::PlayerData { player: self.id, key: key.to_owned() }))
            .ok()
            .filter(|bytes| !bytes.is_empty())
            .and_then(|bytes| decode(&bytes).ok())
            .unwrap_or_default()
    }

    pub fn setData<T: Serialize>(self, key: &str, value: &T) -> Result<()>
    {
        expectUnit(call(HostCall::SetPlayerData { player: self.id, key: key.to_owned(), data: encode(value) }))
    }

    pub fn canSend(self, channel: &str) -> bool
    {
        crate::ownId(channel)
            .is_ok_and(|channel| expectBool(call(HostCall::CanSend { player: self.id, channel })).unwrap_or(false))
    }

    pub fn info(self) -> Option<EntityInfo>
    {
        expectEntities(call(HostCall::Players { world: None })).into_iter().find(|info| info.id == self.id)
    }
}

// every player in every loaded dimension
pub fn allPlayers() -> Vec<Player>
{
    expectEntities(call(HostCall::Players { world: None })).into_iter().map(|info| Player { id: info.id }).collect()
}

pub fn broadcast(message: &str) { let _ = call(HostCall::Chat { player: None, message: message.to_owned() }); }

pub fn send<T: Serialize>(channel: &str, target: Target, message: &T) -> Result<()>
{
    let channel: Identifier = crate::ownId(channel)?;
    expectUnit(call(HostCall::Send { channel, target, data: encode(message) }))
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Entity
{
    pub world: World,
    pub id: i32,
}

impl Entity
{
    pub fn info(self) -> Option<EntityInfo> { self.world.entity(self.id) }

    pub fn setPosition(self, pos: [f64; 3]) -> Result<()>
    {
        expectUnit(call(HostCall::MoveEntity {
            world: self.world.dimension,
            id: self.id,
            pos: Some(pos),
            motion: None,
            rotation: None,
        }))
    }

    pub fn setMotion(self, motion: [f64; 3]) -> Result<()>
    {
        expectUnit(call(HostCall::MoveEntity {
            world: self.world.dimension,
            id: self.id,
            pos: None,
            motion: Some(motion),
            rotation: None,
        }))
    }

    pub fn damage(self, amount: i32, source: Option<i32>) -> bool
    {
        expectBool(call(HostCall::DamageEntity { world: self.world.dimension, id: self.id, source, amount }))
            .unwrap_or(false)
    }

    // needs api 1.18, attributes are minecraft:max_health, movement_speed, attack_damage, armor and
    // knockback_resistance
    pub fn setModifier(self, attribute: &str, modifier: &str, amount: f64, operation: ModifierOperation) -> Result<()>
    {
        expectUnit(call(HostCall::SetModifier {
            world: self.world.dimension,
            id: self.id,
            attribute: Identifier::parse(attribute).map_err(|error| Error(error.to_string()))?,
            modifier: Identifier::parse(modifier).map_err(|error| Error(error.to_string()))?,
            amount,
            operation,
        }))
    }

    // needs api 1.18
    pub fn removeModifier(self, attribute: &str, modifier: &str) -> Result<()>
    {
        expectUnit(call(HostCall::RemoveModifier {
            world: self.world.dimension,
            id: self.id,
            attribute: Identifier::parse(attribute).map_err(|error| Error(error.to_string()))?,
            modifier: Identifier::parse(modifier).map_err(|error| Error(error.to_string()))?,
        }))
    }

    // needs api 1.22, level 0 is the first level
    pub fn addEffect(self, effect: &str, level: i32, ticks: i32) -> Result<()>
    {
        let effect: Identifier = Identifier::parse(effect).map_err(|error| Error(error.to_string()))?;
        expectUnit(call(HostCall::AddEffect { world: self.world.dimension, id: self.id, effect, level, ticks }))
    }

    // needs api 1.22
    pub fn removeEffect(self, effect: &str) -> Result<()>
    {
        let effect: Identifier = Identifier::parse(effect).map_err(|error| Error(error.to_string()))?;
        expectUnit(call(HostCall::RemoveEffect { world: self.world.dimension, id: self.id, effect }))
    }

    // needs api 1.22, None when the entity does not have the effect
    pub fn effectLevel(self, effect: &str) -> Option<i32>
    {
        let effect: Identifier = Identifier::parse(effect).ok()?;
        match call(HostCall::EffectLevel { world: self.world.dimension, id: self.id, effect })
        {
            HostReply::Int(level) if level >= 0 => Some(level),
            _ => None,
        }
    }

    // needs api 1.17, kind names a vanilla damage type such as minecraft:fall
    pub fn damageAs(self, amount: i32, source: Option<i32>, kind: &str) -> Result<bool>
    {
        let kind: Identifier = Identifier::parse(kind).map_err(|error| Error(error.to_string()))?;
        expectBool(call(HostCall::DamageEntityAs { world: self.world.dimension, id: self.id, source, amount, kind }))
    }

    pub fn kill(self) { let _ = call(HostCall::KillEntity { world: self.world.dimension, id: self.id }); }

    // needs api 1.8, the stack a dropped item carries, None for any other entity
    pub fn itemStack(self) -> Option<ItemStack>
    {
        expectStack(call(HostCall::ItemEntityStack { world: self.world.dimension, id: self.id }))
            .ok()
            .flatten()
            .map(ItemStack::fromRaw)
    }

    // bytes the server mirrors to every client that sees the entity, saved with it, one slot per mod so mods never clash
    pub fn synced<T: DeserializeOwned + Default>(self) -> T
    {
        expectBytes(call(HostCall::SyncedData { world: self.world.dimension, id: self.id }))
            .ok()
            .filter(|bytes| !bytes.is_empty())
            .and_then(|bytes| decode(&bytes).ok())
            .unwrap_or_default()
    }

    pub fn setSynced<T: Serialize>(self, value: &T) -> Result<()>
    {
        expectUnit(call(HostCall::SetSyncedData { world: self.world.dimension, id: self.id, data: encode(value) }))
    }

    pub fn field(self, field: EntityField) -> f64
    {
        match call(HostCall::EntityField { world: self.world.dimension, id: self.id, field })
        {
            HostReply::Float(value) => value as f64,
            HostReply::Bool(flag) => f64::from(u8::from(flag)),
            _ => 0.0,
        }
    }

    pub fn flag(self, field: EntityField) -> bool { self.field(field) > 0.0 }

    // rides vehicle, or leaves the current one with None
    pub fn mount(self, vehicle: Option<i32>) -> Result<()>
    {
        expectUnit(call(HostCall::Mount { world: self.world.dimension, id: self.id, vehicle }))
    }

    pub fn riding(self) -> Option<i32> { Some(self.field(EntityField::RidingId) as i32).filter(|id| *id >= 0) }

    pub fn rider(self) -> Option<i32> { Some(self.field(EntityField::RiderId) as i32).filter(|id| *id >= 0) }

    pub fn setField(self, field: EntityField, value: f64)
    {
        let _ = call(HostCall::SetEntityField { world: self.world.dimension, id: self.id, field, value });
    }

    pub fn walkTo(self, pos: BlockPos, speed: f32)
    {
        let _ = call(HostCall::MoveTowards { world: self.world.dimension, id: self.id, pos, speed });
    }

    pub fn data<T: DeserializeOwned + Default>(self) -> T
    {
        expectBytes(call(HostCall::EntityData { world: self.world.dimension, id: self.id }))
            .ok()
            .filter(|bytes| !bytes.is_empty())
            .and_then(|bytes| decode(&bytes).ok())
            .unwrap_or_default()
    }

    pub fn setData<T: Serialize>(self, value: &T) -> Result<()>
    {
        expectUnit(call(HostCall::SetEntityData { world: self.world.dimension, id: self.id, data: encode(value) }))
    }
}

// what a block callback knows: where it runs and whether this side owns the world
#[derive(Clone, Copy, Debug)]
pub struct BlockCtx
{
    pub raw: RawCtx,
}

impl BlockCtx
{
    pub fn world(&self) -> World { World { dimension: self.raw.world } }

    pub fn pos(&self) -> BlockPos { self.raw.pos }

    pub fn block(&self) -> i32 { self.raw.block }

    // this block's named state, the way to read variants of a block with more than 16 states
    pub fn state(&self) -> Option<StateData> { self.world().blockState(self.raw.pos) }

    pub fn setState(&self, state: StateData) -> Result<bool> { self.world().setBlockState(self.raw.pos, state) }

    pub fn meta(&self) -> u8 { self.raw.meta as u8 }

    pub fn isAuthority(&self) -> bool { self.raw.authority }

    pub fn setMeta(&mut self, meta: u8)
    {
        if self.world().setMeta(self.raw.pos, meta as i32).is_ok()
        {
            self.raw.meta = meta as i32;
        }
    }

    pub fn tileData<T: DeserializeOwned + Default>(&self) -> T { self.world().tileData(self.raw.pos) }

    pub fn setTileData<T: Serialize>(&self, value: &T) -> Result<()> { self.world().setTileData(self.raw.pos, value) }

    pub fn sendTracking<T: Serialize>(&self, channel: &str, message: &T) -> Result<()>
    {
        send(channel, Target::Tracking(self.raw.world, self.raw.pos), message)
    }
}

// a data component on a stack in place, readable for any component and writable for this mod's own namespace
pub fn stackComponent<T: DeserializeOwned + Default>(at: StackRef, component: &str) -> T
{
    let Ok(component) = Identifier::parse(component)
    else
    {
        return T::default();
    };
    expectBytes(call(HostCall::StackComponent { at, component }))
        .ok()
        .filter(|bytes| !bytes.is_empty())
        .and_then(|bytes| decode(&bytes).ok())
        .unwrap_or_default()
}

// an empty value removes the component, stacks merge only when their components match
pub fn setStackComponent<T: Serialize>(at: StackRef, component: &str, value: &T) -> Result<()>
{
    let component: Identifier = Identifier::parse(component).map_err(|error| Error(error.to_string()))?;
    expectUnit(call(HostCall::SetStackComponent { at, component, data: encode(value) }))
}

pub fn worldData<T: DeserializeOwned + Default>(key: &str) -> T
{
    expectBytes(call(HostCall::WorldData { key: key.to_owned() }))
        .ok()
        .filter(|bytes| !bytes.is_empty())
        .and_then(|bytes| decode(&bytes).ok())
        .unwrap_or_default()
}

// needs api 1.8, the pixels of a texture drawn at runtime, argb rows from the top left; Draw images and model ops
// that name it draw these pixels from then on, a bare name is taken in this mod's namespace
pub fn setDynamicTexture(name: &str, width: u32, height: u32, pixels: Vec<u32>) -> crate::Result<()>
{
    let name: Identifier = crate::ownId(name)?;
    expectUnit(call(HostCall::SetDynamicTexture { name, width, height, pixels }))
}

pub fn setWorldData<T: Serialize>(key: &str, value: &T)
{
    let _ = call(HostCall::SetWorldData { key: key.to_owned(), data: encode(value) });
}

// every stack a creative item list shows, subtypes included, in numeric id order, each a full stack
pub fn itemCatalog() -> Vec<ItemStack>
{
    match call(HostCall::ItemCatalog)
    {
        HostReply::Stacks(stacks) => stacks.into_iter().map(ItemStack::fromRaw).collect(),
        _ => Vec::new(),
    }
}

// the crafting and smelting recipes the game knows, mod recipes included
pub fn recipeList() -> Vec<RecipeView>
{
    match call(HostCall::RecipeList)
    {
        HostReply::Recipes(recipes) => recipes,
        _ => Vec::new(),
    }
}

// changes whenever ids are remapped or recipe tables rebuilt, so caches of the catalog or recipe list can reload
pub fn contentGeneration() -> i32 { expectInt(call(HostCall::ContentGeneration)).unwrap_or(0) }

// the client font's width of a string, zero on a server
// needs api 1.12, the terrain atlas slot of a block texture for the texture hooks, None when it has none; call it
// once while registering for any texture no block spec names, so the atlas makes room for it
pub fn terrainSlot(texture: &str) -> Option<i32>
{
    expectInt(call(HostCall::TerrainSlot { texture: texture.to_owned() })).ok().filter(|slot| *slot >= 0)
}

// needs api 1.13, cuts a terrain texture into columns by rows tiles that each become a sprite of their own; call it
// while registering, then find each tile's slot with terrainSlot(tileName(texture, column, row))
pub fn terrainTiles(texture: &str, columns: i32, rows: i32) -> crate::Result<()>
{
    expectUnit(call(HostCall::TerrainTiles { texture: texture.to_owned(), columns, rows }))
}

// the texture name of one tile terrainTiles cut, counted from the left and the top
pub fn tileName(texture: &str, column: i32, row: i32) -> String
{
    let (namespace, path): (&str, &str) = texture.split_once(':').unwrap_or(("minecraft", texture));
    format!("loader:tiles/{namespace}/{path}/{column}_{row}")
}

// needs api 1.23, the item sheet slot of a texture for the item icon hooks, None when it has none; call it while
// registering for any texture no item spec names, so the sheet makes room for it
pub fn itemSlot(texture: &str) -> Option<i32>
{
    expectInt(call(HostCall::ItemSlot { texture: texture.to_owned() })).ok().filter(|slot| *slot >= 0)
}

pub fn textWidth(text: &str) -> i32 { expectInt(call(HostCall::TextWidth { text: text.to_owned() })).unwrap_or(0) }

// the scaled gui size of the client screen, none on a server
pub fn screenSize() -> Option<[i32; 2]>
{
    match call(HostCall::ScreenSize)
    {
        HostReply::Size(width, height) => Some([width, height]),
        _ => None,
    }
}

pub fn localPlayer() -> Option<Player> { expectEntity(call(HostCall::LocalPlayer)).map(|info| Player { id: info.id }) }

pub fn isClient() -> bool { expectBool(call(HostCall::IsClient)).unwrap_or(false) }

pub fn isLoaded(modid: &str) -> bool
{
    expectBool(call(HostCall::IsLoaded { modid: modid.to_owned() })).unwrap_or(false)
}

// needs api 1.4, the name a mod's manifest gives, Minecraft for the vanilla namespace and empty when not loaded
pub fn modName(modid: &str) -> String
{
    match call(HostCall::ModName { modid: modid.to_owned() })
    {
        HostReply::Text(text) => text,
        _ => String::new(),
    }
}

pub fn translate(key: &str) -> String
{
    match call(HostCall::Translate { key: key.to_owned() })
    {
        HostReply::Text(text) => text,
        _ => key.to_owned(),
    }
}

fn colorizer(foliage: bool, temperature: f64, rainfall: f64) -> u32
{
    match call(HostCall::Colorizer { foliage, temperature, rainfall })
    {
        HostReply::Int(color) => color as u32 & 0xFFFFFF,
        _ => 0xFFFFFF,
    }
}

// needs api 1.7, ColorizerGrass at a temperature and rainfall clamped to 0 to 1
pub fn grassColor(temperature: f64, rainfall: f64) -> u32 { colorizer(false, temperature, rainfall) }

// needs api 1.7, ColorizerFoliage at a temperature and rainfall clamped to 0 to 1
pub fn foliageColor(temperature: f64, rainfall: f64) -> u32 { colorizer(true, temperature, rainfall) }

// needs api 1.17, the damage type of the hit an attack, hurt or onHurt call is handling
pub fn damageType() -> Option<Identifier>
{
    match call(HostCall::DamageType)
    {
        HostReply::Id(kind) => kind,
        _ => None,
    }
}
