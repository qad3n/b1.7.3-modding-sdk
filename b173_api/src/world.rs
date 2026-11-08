use crate::rt::call;
use crate::{
    Error, ItemStack, Result, expectBool, expectBytes, expectEntities, expectEntity, expectInt, expectStack, expectUnit,
};
use mod_protocol::Identifier;
use mod_protocol::abi::{BlockCtx as RawCtx, BlockPos, EntityInfo, HostCall, HostReply, Target};
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

    pub fn block(self, pos: BlockPos) -> i32
    {
        expectInt(call(HostCall::GetBlock { world: self.dimension, pos })).unwrap_or(0)
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

impl Player
{
    pub fn chat(self, message: &str)
    {
        let _ = call(HostCall::Chat { player: Some(self.id), message: message.to_owned() });
    }

    pub fn give(self, stack: ItemStack) -> Result<()>
    {
        expectUnit(call(HostCall::Give { player: self.id, stack: stack.raw()? }))
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

    pub fn kill(self) { let _ = call(HostCall::KillEntity { world: self.world.dimension, id: self.id }); }

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

pub fn worldData<T: DeserializeOwned + Default>(key: &str) -> T
{
    expectBytes(call(HostCall::WorldData { key: key.to_owned() }))
        .ok()
        .filter(|bytes| !bytes.is_empty())
        .and_then(|bytes| decode(&bytes).ok())
        .unwrap_or_default()
}

pub fn setWorldData<T: Serialize>(key: &str, value: &T)
{
    let _ = call(HostCall::SetWorldData { key: key.to_owned(), data: encode(value) });
}

pub fn localPlayer() -> Option<Player> { expectEntity(call(HostCall::LocalPlayer)).map(|info| Player { id: info.id }) }

pub fn isClient() -> bool { expectBool(call(HostCall::IsClient)).unwrap_or(false) }

pub fn isLoaded(modid: &str) -> bool
{
    expectBool(call(HostCall::IsLoaded { modid: modid.to_owned() })).unwrap_or(false)
}

pub fn translate(key: &str) -> String
{
    match call(HostCall::Translate { key: key.to_owned() })
    {
        HostReply::Text(text) => text,
        _ => key.to_owned(),
    }
}
