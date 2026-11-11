use crate::behavior::{self, BlockBehavior, ItemBehavior};
use crate::rt::call;
use crate::{ItemRef, Result, dispatch, expectUnit, ownId};
use mod_protocol::Identifier;
use mod_protocol::abi::{HostCall, RegistryKind};
use mod_protocol::spec::
{
    Behavior, BlockMethod, BlockModifySpec, BlockSpec, BlockStatesSpec, ItemMethod, ItemSpec, SpawnModifySpec,
    ToolClass, ToolSpec,
};
use std::rc::Rc;

pub struct Blocks;

impl Blocks
{
    pub fn register(&self, name: &str, spec: BlockSpecBuilder) -> Result<ItemRef>
    {
        let id: Identifier = ownId(name)?;
        expectUnit(call(HostCall::RegisterBlock { name: id.to_string(), spec: spec.m_spec }))?;
        Ok(ItemRef::Named(id))
    }

    pub fn replace(&self, id: &str, spec: BlockSpecBuilder) -> Result<()>
    {
        expectUnit(call(HostCall::ReplaceBlock { id: ownId(id)?, spec: spec.m_spec }))
    }

    /// named states for one of this mod's blocks, saved by name so properties may change between versions
    pub fn states(&self, name: &str, spec: BlockStatesSpec) -> Result<()>
    {
        expectUnit(call(HostCall::RegisterBlockStates { block: ownId(name)?, spec }))
    }

    /// changes chosen properties of any block, other mods changing the same field are reported as conflicts
    pub fn modify(&self, spec: BlockModifySpec) -> Result<()> { expectUnit(call(HostCall::ModifyBlock { spec })) }

    /// adds an entity to biome spawn lists, or with weight 0 removes it, for vanilla and other mods' mobs too
    pub fn modifySpawns(&self, spec: SpawnModifySpec) -> Result<()>
    {
        expectUnit(call(HostCall::ModifySpawns { spec }))
    }

    pub fn id(&self, name: &str) -> Result<i32> { crate::resolve(RegistryKind::Block, &ownId(name)?) }
}

pub struct BlockSpecBuilder
{
    pub m_spec: BlockSpec,
}

impl BlockSpecBuilder
{
    pub fn template(template: &str) -> BlockSpecBuilder
    {
        BlockSpecBuilder { m_spec: BlockSpec { template: Identifier::parse(template).ok(), ..BlockSpec::default() } }
    }

    pub fn hardness(mut self, value: f32) -> Self
    {
        self.m_spec.hardness = Some(value);
        self
    }

    pub fn resistance(mut self, value: f32) -> Self
    {
        self.m_spec.resistance = Some(value);
        self
    }

    pub fn light(mut self, value: f32) -> Self
    {
        self.m_spec.light = Some(value);
        self
    }

    pub fn material(mut self, material: &str) -> Self
    {
        self.m_spec.material = Some(material.to_owned());
        self
    }

    pub fn texture(mut self, texture: &str) -> Self
    {
        self.m_spec.texture = Some(texture.to_owned());
        self
    }

    pub fn faces(mut self, faces: mod_protocol::spec::FaceTextures) -> Self
    {
        self.m_spec.textures = Some(faces);
        self
    }

    pub fn color(mut self, color: u32) -> Self
    {
        self.m_spec.color = Some(color);
        self
    }

    pub fn sound(mut self, sound: &str) -> Self
    {
        self.m_spec.sound = Some(sound.to_owned());
        self
    }

    pub fn name(mut self, name: &str) -> Self
    {
        self.m_spec.name = Some(name.to_owned());
        self
    }

    pub fn tool(mut self, class: ToolClass, level: i32) -> Self
    {
        self.m_spec.tool = Some(ToolSpec { class, level });
        self
    }

    pub fn drops(mut self, drops: Vec<mod_protocol::spec::DropSpec>) -> Self
    {
        self.m_spec.drops = Some(drops);
        self
    }

    pub fn tickRandomly(mut self) -> Self
    {
        self.m_spec.tick_randomly = Some(true);
        self
    }

    pub fn tileEntity(mut self, tile: &str) -> Self
    {
        self.m_spec.tile_entity = ownId(tile).ok();
        self
    }

    pub fn fuel(mut self, ticks: i32) -> Self
    {
        self.m_spec.fuel = Some(ticks);
        self
    }

    /// needs api 1.12, how the block's own item behaves in the hand, as an ItemBlock subclass placing onto water
    pub fn itemBehavior(mut self, behavior: impl ItemBehavior + 'static) -> Self
    {
        let methods: Vec<ItemMethod> = behavior::methods(behavior.overridden(), &behavior::ITEM_METHODS);
        let handle: u32 = dispatch::addItem(Rc::new(behavior));
        self.m_spec.item.get_or_insert_with(ItemSpec::default).behavior = Some(Behavior { handle, methods });
        self
    }

    pub fn flammable(mut self, encouragement: i32, flammability: i32) -> Self
    {
        self.m_spec.flammability = Some(mod_protocol::spec::Flammability { encouragement, flammability });
        self
    }

    pub fn model(mut self, model: mod_protocol::spec::BlockModelSpec) -> Self
    {
        self.m_spec.model = Some(model);
        self
    }

    /// a bed of the bed template, its sprites picked by half and facing
    pub fn bed(mut self, faces: mod_protocol::spec::BedFaces) -> Self
    {
        self.m_spec.bed = Some(faces);
        self
    }

    pub fn slipperiness(mut self, value: f32) -> Self
    {
        self.m_spec.slipperiness = Some(value);
        self
    }

    pub fn opacity(mut self, value: i32) -> Self
    {
        self.m_spec.opacity = Some(value);
        self
    }

    pub fn leaves(mut self, decays: bool) -> Self
    {
        self.m_spec.leaves = Some(decays);
        self
    }

    pub fn sustainsLeaves(mut self) -> Self
    {
        self.m_spec.sustains_leaves = Some(true);
        self
    }

    pub fn collision(mut self, boxes: Vec<mod_protocol::spec::Cuboid>) -> Self
    {
        self.m_spec.collision = Some(boxes);
        self
    }

    pub fn tag(mut self, tag: &str) -> Self
    {
        if let Ok(tag) = Identifier::parse(tag)
        {
            self.m_spec.tags.push(tag);
        }

        self
    }

    pub fn vanillaFallback(mut self, block: &str) -> Self
    {
        self.m_spec.vanilla_fallback = Identifier::parse(block).ok();
        self
    }

    pub fn behavior(mut self, behavior: impl BlockBehavior + 'static) -> Self
    {
        let methods: Vec<BlockMethod> = behavior::methods(behavior.overridden(), &behavior::BLOCK_METHODS);
        let handle: u32 = dispatch::addBlock(Rc::new(behavior));
        self.m_spec.behavior = Some(Behavior { handle, methods });
        self
    }
}
