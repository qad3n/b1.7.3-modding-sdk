use super::{Entity, World, encode, encodeStored};
use crate::rt::{call, fire};
use crate::
{
    IntoName, ItemStack, Result, expectBool, expectEntities, expectFloat, expectInt, expectStack, expectText,
    expectUnit, fetchData,
};
use mod_protocol::Identifier;
use mod_protocol::abi::
{
    BlockPos, EntityField, EntityInfo, InventoryRequest, NetRequest, PlayerRequest, RegistryRequest, ScreenRequest,
    StackData, StoredRequest, Target, WorldRequest,
};
use serde::Serialize;
use serde::de::DeserializeOwned;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Player
{
    pub id: i32,
}

/// the name a stack shows in its own language, vanilla items included
pub fn displayName(stack: &ItemStack) -> String
{
    stack.raw().and_then(|stack| expectText(call(RegistryRequest::DisplayName { stack }))).unwrap_or_default()
}

/// the time of every loaded world
pub fn setWorldTime(time: i64) { fire(WorldRequest::SetWorldTime { time }); }

impl Player
{
    pub fn isOp(self) -> bool { expectBool(call(PlayerRequest::IsOp { player: self.id })).unwrap_or(false) }

    /// a slot of the container screen the player has open, by container slot number
    pub fn openSlot(self, slot: i32) -> Option<ItemStack>
    {
        expectStack(call(InventoryRequest::OpenSlot { player: self.id, slot })).ok().flatten().map(ItemStack::fromRaw)
    }

    pub fn chat(self, message: &str)
    {
        fire(PlayerRequest::Chat { player: Some(self.id), message: message.to_owned() });
    }

    pub fn give(self, stack: ItemStack) -> Result<()>
    {
        expectUnit(call(PlayerRequest::Give { player: self.id, stack: stack.raw()? }))
    }

    /// needs api 1.20, holds its current item in use for ticks, the item's onUsingTick, onStoppedUsing and onUseFinish
    /// follow; 0 lets go without telling the item
    pub fn setItemInUse(self, ticks: i32) -> Result<()>
    {
        expectUnit(call(PlayerRequest::SetItemInUse { player: self.id, ticks }))
    }

    /// needs api 1.20, lets go of the item in use as releasing the use key does
    pub fn stopUsingItem(self) -> Result<()> { expectUnit(call(PlayerRequest::StopUsingItem { player: self.id })) }

    /// needs api 1.20, ticks left on the item in use, 0 when nothing is in use
    pub fn itemInUse(self) -> i32
    {
        self.info().map_or(0, |info|
        {
            Entity { world: World { dimension: info.world }, id: self.id }.field(EntityField::ItemInUse) as i32
        })
    }

    /// needs api 1.14, how fast it digs a block with what it holds, as EntityPlayer.getCurrentPlayerStrVsBlock
    pub fn digSpeed(self, block: impl Into<crate::BlockRef>) -> f32
    {
        let Ok(block) = block.into().id()
        else
        {
            return 1.0;
        };

        expectFloat(call(PlayerRequest::DigSpeed { player: self.id, block })).unwrap_or(1.0)
    }

    pub fn held(self) -> Option<ItemStack>
    {
        expectStack(call(PlayerRequest::HeldStack { player: self.id })).ok().flatten().map(ItemStack::fromRaw)
    }

    pub fn setHeld(self, stack: Option<ItemStack>) -> Result<()>
    {
        let stack: Option<StackData> = stack.map(|stack| stack.raw()).transpose()?;
        expectUnit(call(PlayerRequest::SetHeldStack { player: self.id, stack }))
    }

    /// needs api 1.25, true when newly earned; as in beta the parent comes first
    pub fn unlockAchievement(self, achievement: impl IntoName) -> Result<bool>
    {
        let achievement: Identifier = achievement.vanillaName()?;
        expectBool(call(PlayerRequest::UnlockAchievement { player: self.id, achievement }))
    }

    /// needs api 1.25
    pub fn hasAchievement(self, achievement: impl IntoName) -> bool
    {
        let Ok(achievement) = achievement.vanillaName()
        else
        {
            return false;
        };

        expectBool(call(PlayerRequest::HasAchievement { player: self.id, achievement })).unwrap_or(false)
    }

    /// needs api 1.25, stats are named such as <modid>:bosses_killed or mined/<block id>, or a StatRef from api 1.56
    pub fn addStat(self, stat: impl std::fmt::Display, amount: i32) -> Result<()>
    {
        expectUnit(call(PlayerRequest::AddStat { player: self.id, stat: stat.to_string(), amount }))
    }

    /// needs api 1.25
    pub fn stat(self, stat: impl std::fmt::Display) -> i32
    {
        expectInt(call(PlayerRequest::StatValue { player: self.id, stat: stat.to_string() })).unwrap_or(0)
    }

    /// needs api 1.7, wears the held tool as a use would and breaks it past its last use
    pub fn damageHeld(self, amount: i32) -> Result<()>
    {
        expectUnit(call(PlayerRequest::DamageHeld { player: self.id, amount }))
    }

    pub fn teleport(self, dimension: i32, pos: [f64; 3]) -> Result<()>
    {
        expectUnit(call(PlayerRequest::Teleport { player: self.id, dimension, pos }))
    }

    pub fn openScreen(self, screen: &str, pos: Option<BlockPos>, data: Vec<u8>) -> Result<()>
    {
        let screen: Identifier = crate::ownIdentifier(screen)?;
        expectUnit(call(ScreenRequest::OpenScreen { player: self.id, screen, pos, data }))
    }

    pub fn closeScreen(self) { fire(ScreenRequest::CloseScreen { player: self.id }); }

    pub fn cursor(self) -> Option<ItemStack>
    {
        expectStack(call(InventoryRequest::CursorStack { player: self.id })).ok().flatten().map(ItemStack::fromRaw)
    }

    pub fn setCursor(self, stack: Option<ItemStack>) -> Result<()>
    {
        let stack: Option<StackData> = stack.map(|stack| stack.raw()).transpose()?;
        expectUnit(call(InventoryRequest::SetCursorStack { player: self.id, stack }))
    }

    pub fn slot(self, slot: i32) -> Option<ItemStack>
    {
        expectStack(call(InventoryRequest::InventorySlot { player: self.id, slot }))
            .ok()
            .flatten()
            .map(ItemStack::fromRaw)
    }

    pub fn setSlot(self, slot: i32, stack: Option<ItemStack>) -> Result<()>
    {
        let stack: Option<StackData> = stack.map(|stack| stack.raw()).transpose()?;
        expectUnit(call(InventoryRequest::SetInventorySlot { player: self.id, slot, stack }))
    }

    pub fn data<T: DeserializeOwned + Default>(self, key: &str) -> T
    {
        fetchData(call(StoredRequest::PlayerData { player: self.id, key: key.to_owned() }))
    }

    pub fn setData<T: Serialize>(self, key: &str, value: &T) -> Result<()>
    {
        expectUnit(call(StoredRequest::SetPlayerData
        {
            player: self.id,
            key: key.to_owned(),
            data: encodeStored(value),
        }))
    }

    pub fn canSend(self, channel: &str) -> bool
    {
        crate::ownIdentifier(channel)
            .is_ok_and(|channel| expectBool(call(NetRequest::CanSend { player: self.id, channel })).unwrap_or(false))
    }

    pub fn info(self) -> Option<EntityInfo>
    {
        expectEntities(call(PlayerRequest::Players { world: None })).into_iter().find(|info| info.id == self.id)
    }
}

/// every player in every loaded dimension
pub fn allPlayers() -> Vec<Player>
{
    expectEntities(call(PlayerRequest::Players { world: None }))
        .into_iter()
        .map(|info| Player { id: info.id })
        .collect()
}

pub fn broadcast(message: &str) { fire(PlayerRequest::Chat { player: None, message: message.to_owned() }); }

pub fn send<T: Serialize>(channel: &str, target: Target, message: &T) -> Result<()>
{
    let channel: Identifier = crate::ownIdentifier(channel)?;
    expectUnit(call(NetRequest::Send { channel, target, data: encode(message) }))
}
