use crate::rt::call;
use crate::world::Entity;
use crate::{Result, expectBytes, expectUnit};
use mod_protocol::abi::HostCall;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::marker::PhantomData;

/// needs api 1.52, a value of type T this mod keeps per entity, declared once as a static:
/// `static MOOD: EntityState<Mood> = EntityState::new("mood");`
/// The loader keeps it and drops it when the entity leaves its world, so nothing leaks and no map of entity ids is
/// needed. saved() writes it with the entity's save; for players kept() also carries it through death
pub struct EntityState<T>
{
    m_key: &'static str,
    m_saved: bool,
    m_kept: bool,
    m_kind: PhantomData<fn() -> T>,
}

impl<T> EntityState<T>
{
    pub const fn new(key: &'static str) -> Self
    {
        EntityState { m_key: key, m_saved: false, m_kept: false, m_kind: PhantomData }
    }

    pub const fn saved(self) -> Self { EntityState { m_saved: true, ..self } }

    pub const fn kept(self) -> Self { EntityState { m_saved: true, m_kept: true, ..self } }
}

impl<T: Serialize + DeserializeOwned + Default> EntityState<T>
{
    /// the value, or T's default when none was set
    pub fn get(&self, entity: Entity) -> T { self.find(entity).unwrap_or_default() }

    pub fn find(&self, entity: Entity) -> Option<T>
    {
        let call: HostCall = HostCall::EntityValue
        {
            world: entity.world.dimension,
            id: entity.id,
            key: self.m_key.to_owned(),
            saved: self.m_saved,
        };
        let warn = |error: crate::Error| crate::warn(&format!("entity value {}: {error}", self.m_key));
        let bytes: Vec<u8> = expectBytes(crate::rt::call(call)).map_err(warn).ok()?;
        if bytes.is_empty()
        {
            return None;
        }

        crate::fromBytes(&bytes).map_err(warn).ok()
    }

    pub fn set(&self, entity: Entity, value: &T) -> Result<()> { self.write(entity, crate::toBytes(value)) }

    pub fn update(&self, entity: Entity, change: impl FnOnce(&mut T)) -> Result<()>
    {
        let mut value: T = self.get(entity);
        change(&mut value);
        self.set(entity, &value)
    }

    pub fn clear(&self, entity: Entity) -> Result<()> { self.write(entity, Vec::new()) }

    fn write(&self, entity: Entity, data: Vec<u8>) -> Result<()>
    {
        expectUnit(call(HostCall::SetEntityValue
        {
            world: entity.world.dimension,
            id: entity.id,
            key: self.m_key.to_owned(),
            saved: self.m_saved,
            kept: self.m_kept,
            data,
        }))
    }
}
