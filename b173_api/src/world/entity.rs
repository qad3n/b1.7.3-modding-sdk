use super::{World, encode, stored};
use crate::rt::call;
use crate::{ItemStack, Result, expectBool, expectStack, expectUnit};
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

    pub fn damage(self, amount: i32, source: Option<i32>) -> bool
    {
        expectBool(call(HostCall::DamageEntity { world: self.world.dimension, id: self.id, source, amount }))
            .unwrap_or(false)
    }

    /// needs api 1.18, attributes are minecraft:max_health, movement_speed, attack_damage, armor and
    /// knockback_resistance
    pub fn setModifier(self, attribute: &str, modifier: &str, amount: f64, operation: ModifierOperation) -> Result<()>
    {
        expectUnit(call(HostCall::SetModifier
        {
            world: self.world.dimension,
            id: self.id,
            attribute: Identifier::parse(attribute)?,
            modifier: Identifier::parse(modifier)?,
            amount,
            operation,
        }))
    }

    /// needs api 1.18
    pub fn removeModifier(self, attribute: &str, modifier: &str) -> Result<()>
    {
        expectUnit(call(HostCall::RemoveModifier
        {
            world: self.world.dimension,
            id: self.id,
            attribute: Identifier::parse(attribute)?,
            modifier: Identifier::parse(modifier)?,
        }))
    }

    /// needs api 1.22, level 0 is the first level
    pub fn addEffect(self, effect: &str, level: i32, ticks: i32) -> Result<()>
    {
        let effect: Identifier = Identifier::parse(effect)?;
        expectUnit(call(HostCall::AddEffect { world: self.world.dimension, id: self.id, effect, level, ticks }))
    }

    /// needs api 1.22
    pub fn removeEffect(self, effect: &str) -> Result<()>
    {
        let effect: Identifier = Identifier::parse(effect)?;
        expectUnit(call(HostCall::RemoveEffect { world: self.world.dimension, id: self.id, effect }))
    }

    /// needs api 1.22, None when the entity does not have the effect
    pub fn effectLevel(self, effect: &str) -> Option<i32>
    {
        let effect: Identifier = Identifier::parse(effect).ok()?;
        match call(HostCall::EffectLevel { world: self.world.dimension, id: self.id, effect })
        {
            HostReply::Int(level) if level >= 0 => Some(level),
            _ => None,
        }
    }

    /// needs api 1.17, kind names a vanilla damage type such as minecraft:fall
    pub fn damageAs(self, amount: i32, source: Option<i32>, kind: &str) -> Result<bool>
    {
        let kind: Identifier = Identifier::parse(kind)?;
        expectBool(call(HostCall::DamageEntityAs { world: self.world.dimension, id: self.id, source, amount, kind }))
    }

    pub fn kill(self) { let _ = call(HostCall::KillEntity { world: self.world.dimension, id: self.id }); }

    /// needs api 1.8, the stack a dropped item carries, None for any other entity
    pub fn itemStack(self) -> Option<ItemStack>
    {
        expectStack(call(HostCall::ItemEntityStack { world: self.world.dimension, id: self.id }))
            .ok()
            .flatten()
            .map(ItemStack::fromRaw)
    }

    /// bytes the server mirrors to every client that sees the entity, saved with it, one slot per mod so mods never clash
    pub fn synced<T: DeserializeOwned + Default>(self) -> T
    {
        stored(call(HostCall::SyncedData { world: self.world.dimension, id: self.id }))
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
        let _ = call(HostCall::SetEntityField { world: self.world.dimension, id: self.id, field, value });
    }

    pub fn walkTo(self, pos: BlockPos, speed: f32)
    {
        let _ = call(HostCall::MoveTowards { world: self.world.dimension, id: self.id, pos, speed });
    }

    pub fn data<T: DeserializeOwned + Default>(self) -> T
    {
        stored(call(HostCall::EntityData { world: self.world.dimension, id: self.id }))
    }

    pub fn setData<T: Serialize>(self, value: &T) -> Result<()>
    {
        expectUnit(call(HostCall::SetEntityData { world: self.world.dimension, id: self.id, data: encode(value) }))
    }
}
