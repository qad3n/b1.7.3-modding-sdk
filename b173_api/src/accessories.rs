use crate::rt::call;
use crate::world::{Entity, Player};
use crate::{IntoName, ItemStack, Result, expectBool, expectUnit, ownIdentifier};
use mod_protocol::Identifier;
use mod_protocol::abi::{ContentRequest, EntityRequest, HostReply, PlayerRequest, StackData, Worn};
use mod_protocol::spec::AccessorySlotSpec;

/// needs api 1.40, accessory slots players wear one item in, shown in mod panels beside the vanilla inventory
pub struct Accessories;

impl Accessories
{
    /// a slot named <modid>:<name> taking items of tag, icon the sprite an empty slot shows
    pub fn register(&self, name: &str, tag: &str, icon: Option<&str>) -> Result<crate::refs::SlotRef>
    {
        let spec: AccessorySlotSpec =
            AccessorySlotSpec { tag: ownIdentifier(tag.trim_start_matches('#'))?, icon: icon.map(str::to_owned) };
        crate::register::registered(name, |id| ContentRequest::RegisterAccessorySlot { name: id.clone(), spec })
    }
}

/// an accessory slot and what is worn in it
#[derive(Clone, PartialEq, Debug)]
pub struct WornStack
{
    pub slot: Identifier,
    pub icon: Option<String>,
    pub stack: Option<ItemStack>,
}

impl Entity
{
    /// needs api 1.40, every accessory slot in registration order with what this entity wears there, on clients too
    pub fn accessories(self) -> Vec<WornStack>
    {
        let worn: Vec<Worn> = match call(EntityRequest::Accessories { world: self.world.dimension, entity: self.id })
        {
            HostReply::Accessories(worn) => worn,
            _ => Vec::new(),
        };

        worn.into_iter()
            .map(|worn| WornStack { slot: worn.slot, icon: worn.icon, stack: worn.stack.map(ItemStack::fromRaw) })
            .collect()
    }

    /// needs api 1.40, puts one item the slot's tag takes in an accessory slot, or empties it, authority only
    pub fn setAccessory(self, slot: impl IntoName, stack: Option<ItemStack>) -> Result<()>
    {
        let stack: Option<StackData> = stack.map(|stack| stack.raw()).transpose()?;
        let call: EntityRequest =
            EntityRequest::SetAccessory { world: self.world.dimension, entity: self.id, slot: slot.intoName()?, stack };
        expectUnit(crate::rt::call(call))
    }
}

impl Player
{
    /// needs api 1.40, a click on an accessory slot: the cursor stack and the slot trade places when the slot takes the
    /// cursor stack, authority only, false when it does not
    pub fn swapAccessory(self, slot: impl IntoName) -> bool
    {
        let Ok(slot) = slot.intoName()
        else
        {
            return false;
        };

        expectBool(call(PlayerRequest::SwapAccessory { player: self.id, slot })).unwrap_or(false)
    }
}
