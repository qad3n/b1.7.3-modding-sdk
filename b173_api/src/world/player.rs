use super::{Entity, World, encode};
use crate::rt::{call, fire};
use crate::
{
    ItemStack, Result, expectBool, expectEntities, expectFloat, expectInt, expectStack, expectText, expectUnit,
    fetchData,
};
use mod_protocol::Identifier;
use mod_protocol::abi::{BlockPos, EntityField, EntityInfo, HostCall, StackData, Target};
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
    stack.raw().and_then(|stack| expectText(call(HostCall::DisplayName { stack }))).unwrap_or_default()
}

/// the time of every loaded world
pub fn setWorldTime(time: i64) { fire(HostCall::SetWorldTime { time }); }

impl Player
{
    pub fn isOp(self) -> bool { expectBool(call(HostCall::IsOp { player: self.id })).unwrap_or(false) }

    /// a slot of the container screen the player has open, by container slot number
    pub fn openSlot(self, slot: i32) -> Option<ItemStack>
    {
        expectStack(call(HostCall::OpenSlot { player: self.id, slot })).ok().flatten().map(ItemStack::fromRaw)
    }

    pub fn chat(self, message: &str) { fire(HostCall::Chat { player: Some(self.id), message: message.to_owned() }); }

    pub fn give(self, stack: ItemStack) -> Result<()>
    {
        expectUnit(call(HostCall::Give { player: self.id, stack: stack.raw()? }))
    }

    /// needs api 1.20, holds its current item in use for ticks, the item's onUsingTick, onStoppedUsing and onUseFinish
    /// follow; 0 lets go without telling the item
    pub fn setItemInUse(self, ticks: i32) -> Result<()>
    {
        expectUnit(call(HostCall::SetItemInUse { player: self.id, ticks }))
    }

    /// needs api 1.20, lets go of the item in use as releasing the use key does
    pub fn stopUsingItem(self) -> Result<()> { expectUnit(call(HostCall::StopUsingItem { player: self.id })) }

    /// needs api 1.20, ticks left on the item in use, 0 when nothing is in use
    pub fn itemInUse(self) -> i32
    {
        self.info().map_or(0, |info|
        {
            Entity { world: World { dimension: info.world }, id: self.id }.field(EntityField::ItemInUse) as i32
        })
    }

    /// needs api 1.14, how fast it digs a block with what it holds, as EntityPlayer.getCurrentPlayerStrVsBlock
    pub fn digSpeed(self, block: i32) -> f32
    {
        expectFloat(call(HostCall::DigSpeed { player: self.id, block })).unwrap_or(1.0)
    }

    pub fn held(self) -> Option<ItemStack>
    {
        expectStack(call(HostCall::HeldStack { player: self.id })).ok().flatten().map(ItemStack::fromRaw)
    }

    pub fn setHeld(self, stack: Option<ItemStack>) -> Result<()>
    {
        let stack: Option<StackData> = stack.map(|stack| stack.raw()).transpose()?;
        expectUnit(call(HostCall::SetHeldStack { player: self.id, stack }))
    }

    /// needs api 1.25, true when newly earned; as in beta the parent comes first
    pub fn unlockAchievement(self, achievement: &str) -> Result<bool>
    {
        let achievement: Identifier = Identifier::parse(achievement)?;
        expectBool(call(HostCall::UnlockAchievement { player: self.id, achievement }))
    }

    /// needs api 1.25
    pub fn hasAchievement(self, achievement: &str) -> bool
    {
        let Ok(achievement) = Identifier::parse(achievement)
        else
        {
            return false;
        };

        expectBool(call(HostCall::HasAchievement { player: self.id, achievement })).unwrap_or(false)
    }

    /// needs api 1.25, stats are named such as <modid>:bosses_killed or mined/<block id>
    pub fn addStat(self, stat: &str, amount: i32) -> Result<()>
    {
        expectUnit(call(HostCall::AddStat { player: self.id, stat: stat.to_owned(), amount }))
    }

    /// needs api 1.25
    pub fn stat(self, stat: &str) -> i32
    {
        expectInt(call(HostCall::StatValue { player: self.id, stat: stat.to_owned() })).unwrap_or(0)
    }

    /// needs api 1.7, wears the held tool as a use would and breaks it past its last use
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
        let screen: Identifier = crate::ownIdentifier(screen)?;
        expectUnit(call(HostCall::OpenScreen { player: self.id, screen, pos, data }))
    }

    pub fn closeScreen(self) { fire(HostCall::CloseScreen { player: self.id }); }

    pub fn cursor(self) -> Option<ItemStack>
    {
        expectStack(call(HostCall::CursorStack { player: self.id })).ok().flatten().map(ItemStack::fromRaw)
    }

    pub fn setCursor(self, stack: Option<ItemStack>) -> Result<()>
    {
        let stack: Option<StackData> = stack.map(|stack| stack.raw()).transpose()?;
        expectUnit(call(HostCall::SetCursorStack { player: self.id, stack }))
    }

    pub fn slot(self, slot: i32) -> Option<ItemStack>
    {
        expectStack(call(HostCall::InventorySlot { player: self.id, slot })).ok().flatten().map(ItemStack::fromRaw)
    }

    pub fn setSlot(self, slot: i32, stack: Option<ItemStack>) -> Result<()>
    {
        let stack: Option<StackData> = stack.map(|stack| stack.raw()).transpose()?;
        expectUnit(call(HostCall::SetInventorySlot { player: self.id, slot, stack }))
    }

    pub fn data<T: DeserializeOwned + Default>(self, key: &str) -> T
    {
        fetchData(call(HostCall::PlayerData { player: self.id, key: key.to_owned() }))
    }

    pub fn setData<T: Serialize>(self, key: &str, value: &T) -> Result<()>
    {
        expectUnit(call(HostCall::SetPlayerData { player: self.id, key: key.to_owned(), data: encode(value) }))
    }

    pub fn canSend(self, channel: &str) -> bool
    {
        crate::ownIdentifier(channel)
            .is_ok_and(|channel| expectBool(call(HostCall::CanSend { player: self.id, channel })).unwrap_or(false))
    }

    pub fn info(self) -> Option<EntityInfo>
    {
        expectEntities(call(HostCall::Players { world: None })).into_iter().find(|info| info.id == self.id)
    }
}

/// every player in every loaded dimension
pub fn allPlayers() -> Vec<Player>
{
    expectEntities(call(HostCall::Players { world: None })).into_iter().map(|info| Player { id: info.id }).collect()
}

pub fn broadcast(message: &str) { fire(HostCall::Chat { player: None, message: message.to_owned() }); }

pub fn send<T: Serialize>(channel: &str, target: Target, message: &T) -> Result<()>
{
    let channel: Identifier = crate::ownIdentifier(channel)?;
    expectUnit(call(HostCall::Send { channel, target, data: encode(message) }))
}
