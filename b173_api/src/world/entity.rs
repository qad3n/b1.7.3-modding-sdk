use super::{World, encode};
use crate::rt::{call, fire};
use crate::{IntoName, ItemStack, Result, expectBool, expectDouble, expectInt, expectStack, expectUnit, fetchData};
use mod_protocol::Identifier;
use mod_protocol::abi::{BlockPos, EntityField, EntityInfo, HostCall, HostReply, ModifierOperation};
use serde::Serialize;
use serde::de::DeserializeOwned;

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
        expectUnit(call(HostCall::MoveEntity
        {
            world: self.world.dimension,
            id: self.id,
            pos: Some(pos),
            motion: None,
            rotation: None,
        }))
    }

    pub fn setMotion(self, motion: [f64; 3]) -> Result<()>
    {
        expectUnit(call(HostCall::MoveEntity
        {
            world: self.world.dimension,
            id: self.id,
            pos: None,
            motion: Some(motion),
            rotation: None,
        }))
    }

    /// needs api 1.28, Entity.moveFlying: adds motion from strafe and forward input along the yaw, scaled by factor
    pub fn accelerate(self, strafe: f32, forward: f32, factor: f32) -> Result<()>
    {
        expectUnit(call(HostCall::Accelerate { world: self.world.dimension, id: self.id, strafe, forward, factor }))
    }

    /// needs api 1.28, Entity.moveEntity: moves by motion and stops at the blocks it runs into
    pub fn step(self, motion: [f64; 3]) -> Result<()>
    {
        expectUnit(call(HostCall::Step { world: self.world.dimension, id: self.id, motion }))
    }

    /// needs api 1.28, swings the legs by how far the entity moved this tick
    pub fn animateWalk(self) -> Result<()>
    {
        expectUnit(call(HostCall::AnimateWalk { world: self.world.dimension, id: self.id }))
    }

    pub fn damage(self, amount: i32, source: Option<i32>) -> bool
    {
        expectBool(call(HostCall::DamageEntity { world: self.world.dimension, id: self.id, source, amount }))
            .unwrap_or(false)
    }

    /// needs api 1.18, vanilla attributes are minecraft:max_health, movement_speed, attack_damage, armor,
    /// knockback_resistance and swim_speed, api 1.27 adds any a mod registers
    pub fn setModifier(
        self,
        attribute: impl IntoName,
        modifier: &str,
        amount: f64,
        operation: ModifierOperation,
    ) -> Result<()>
    {
        expectUnit(call(HostCall::SetModifier
        {
            world: self.world.dimension,
            id: self.id,
            attribute: attribute.vanillaName()?,
            modifier: Identifier::parse(modifier)?,
            amount,
            operation,
        }))
    }

    /// needs api 1.18
    pub fn removeModifier(self, attribute: impl IntoName, modifier: &str) -> Result<()>
    {
        expectUnit(call(HostCall::RemoveModifier
        {
            world: self.world.dimension,
            id: self.id,
            attribute: attribute.vanillaName()?,
            modifier: Identifier::parse(modifier)?,
        }))
    }

    /// needs api 1.27, the attribute's registered default with this entity's modifiers applied
    pub fn attribute(self, attribute: impl IntoName) -> Result<f64>
    {
        let attribute: Identifier = attribute.vanillaName()?;
        expectDouble(call(HostCall::AttributeValue { world: self.world.dimension, id: self.id, attribute }))
    }

    /// needs api 1.22, level 0 is the first level
    pub fn addEffect(self, effect: impl IntoName, level: i32, ticks: i32) -> Result<()>
    {
        let effect: Identifier = effect.vanillaName()?;
        expectUnit(call(HostCall::AddEffect { world: self.world.dimension, id: self.id, effect, level, ticks }))
    }

    /// needs api 1.22
    pub fn removeEffect(self, effect: impl IntoName) -> Result<()>
    {
        let effect: Identifier = effect.vanillaName()?;
        expectUnit(call(HostCall::RemoveEffect { world: self.world.dimension, id: self.id, effect }))
    }

    /// needs api 1.22, None when the entity does not have the effect
    pub fn effectLevel(self, effect: impl IntoName) -> Option<i32>
    {
        let effect: Identifier = effect.vanillaName().ok()?;
        expectInt(call(HostCall::EffectLevel { world: self.world.dimension, id: self.id, effect }))
            .ok()
            .filter(|level| *level >= 0)
    }

    /// needs api 1.17, kind names a vanilla damage type such as minecraft:fall
    pub fn damageAs(self, amount: i32, source: Option<i32>, kind: impl IntoName) -> Result<bool>
    {
        let kind: Identifier = kind.vanillaName()?;
        expectBool(call(HostCall::DamageEntityAs { world: self.world.dimension, id: self.id, source, amount, kind }))
    }

    pub fn kill(self) { fire(HostCall::KillEntity { world: self.world.dimension, id: self.id }); }

    /// needs api 1.8, the stack a dropped item carries, None for any other entity
    pub fn itemStack(self) -> Option<ItemStack>
    {
        expectStack(call(HostCall::ItemEntityStack { world: self.world.dimension, id: self.id }))
            .ok()
            .flatten()
            .map(ItemStack::fromRaw)
    }

    /// needs api 1.33, how many slots the entity's own inventory has, as a chest minecart's
    pub fn inventorySize(self) -> i32
    {
        expectInt(call(HostCall::EntityInventorySize { world: self.world.dimension, id: self.id })).unwrap_or(0)
    }

    /// needs api 1.33, one slot of the entity's own inventory
    pub fn slot(self, slot: i32) -> Option<ItemStack>
    {
        expectStack(call(HostCall::EntitySlot { world: self.world.dimension, id: self.id, slot }))
            .ok()
            .flatten()
            .map(ItemStack::fromRaw)
    }

    /// needs api 1.33, changes one slot of the entity's own inventory
    pub fn setSlot(self, slot: i32, stack: Option<&ItemStack>) -> Result<()>
    {
        let stack: Option<mod_protocol::abi::StackData> = stack.map(ItemStack::raw).transpose()?;
        expectUnit(call(HostCall::SetEntitySlot { world: self.world.dimension, id: self.id, slot, stack }))
    }

    /// needs api 1.33, changes the stack a dropped item carries, an empty count removes the entity
    pub fn setItemStack(self, stack: &ItemStack) -> Result<()>
    {
        let stack: mod_protocol::abi::StackData = stack.raw()?;
        expectUnit(call(HostCall::SetItemStack { world: self.world.dimension, id: self.id, stack }))
    }

    /// bytes the server mirrors to every client that sees the entity, saved with it, one slot per mod so mods never clash
    pub fn synced<T: DeserializeOwned + Default>(self) -> T
    {
        fetchData(call(HostCall::SyncedData { world: self.world.dimension, id: self.id }))
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

    /// rides vehicle, or leaves the current one with None
    pub fn mount(self, vehicle: Option<i32>) -> Result<()>
    {
        expectUnit(call(HostCall::Mount { world: self.world.dimension, id: self.id, vehicle }))
    }

    pub fn riding(self) -> Option<i32> { Some(self.field(EntityField::RidingId) as i32).filter(|id| *id >= 0) }

    pub fn rider(self) -> Option<i32> { Some(self.field(EntityField::RiderId) as i32).filter(|id| *id >= 0) }

    pub fn setField(self, field: EntityField, value: f64)
    {
        fire(HostCall::SetEntityField { world: self.world.dimension, id: self.id, field, value });
    }

    pub fn walkTo(self, pos: BlockPos, speed: f32)
    {
        fire(HostCall::MoveTowards { world: self.world.dimension, id: self.id, pos, speed });
    }

    pub fn data<T: DeserializeOwned + Default>(self) -> T
    {
        fetchData(call(HostCall::EntityData { world: self.world.dimension, id: self.id }))
    }

    pub fn setData<T: Serialize>(self, value: &T) -> Result<()>
    {
        expectUnit(call(HostCall::SetEntityData { world: self.world.dimension, id: self.id, data: encode(value) }))
    }
}
