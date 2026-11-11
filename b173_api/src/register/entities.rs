use crate::behavior::{self, EntityBehavior, TileBehavior};
use crate::rt::call;
use crate::{Result, dispatch, expectUnit, ownId};
use mod_protocol::abi::HostCall;
use mod_protocol::spec::{Behavior, EntityMethod, EntitySpec, TileEntitySpec, TileMethod};
use std::rc::Rc;

pub struct Entities;

impl Entities
{
    pub fn register(&self, name: &str, spec: EntitySpec, behavior: Option<Rc<dyn EntityBehavior>>) -> Result<()>
    {
        let mut spec: EntitySpec = spec;

        if let Some(behavior) = behavior
        {
            let methods: Vec<EntityMethod> = behavior::methods(behavior.overridden(), &behavior::ENTITY_METHODS);
            spec.behavior = Some(Behavior { handle: dispatch::addEntity(behavior), methods });
        }

        expectUnit(call(HostCall::RegisterEntity { name: ownId(name)?.to_string(), spec }))
    }
}

pub struct TileEntities;

impl TileEntities
{
    pub fn register(&self, name: &str, spec: TileEntitySpec, behavior: Option<Rc<dyn TileBehavior>>) -> Result<()>
    {
        let mut spec: TileEntitySpec = spec;

        if let Some(behavior) = behavior
        {
            let methods: Vec<TileMethod> = behavior::methods(behavior.overridden(), &behavior::TILE_METHODS);
            spec.behavior = Some(Behavior { handle: dispatch::addTile(behavior), methods });
        }

        expectUnit(call(HostCall::RegisterTileEntity { name: ownId(name)?.to_string(), spec }))
    }

    /// needs api 1.20, the model every tile of the type draws with, posed each frame by its behavior's pose
    pub fn model(&self, name: &str, model: crate::EntityModelSpec) -> Result<()>
    {
        expectUnit(call(HostCall::TileModel { tile: ownId(name)?, model }))
    }
}
