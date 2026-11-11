use crate::behavior::{self, BlockBehavior, ItemBehavior};
use crate::rt::call;
use crate::{ItemRef, Result, dispatch, expectUnit, ownIdentifier};
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
        let id: Identifier = ownIdentifier(name)?;
        expectUnit(call(HostCall::RegisterBlock { name: id.to_string(), spec: spec.m_spec }))?;
        Ok(ItemRef::Named(id))
    }

    pub fn replace(&self, id: &str, spec: BlockSpecBuilder) -> Result<()>
    {
        expectUnit(call(HostCall::ReplaceBlock { id: ownIdentifier(id)?, spec: spec.m_spec }))
    }

    /// named states for one of this mod's blocks, saved by name so properties may change between versions
    pub fn states(&self, name: &str, spec: BlockStatesSpec) -> Result<()>
    {
        expectUnit(call(HostCall::RegisterBlockStates { block: ownIdentifier(name)?, spec }))
    }

    /// changes chosen properties of any block, other mods changing the same field are reported as conflicts
    pub fn modify(&self, spec: BlockModifySpec) -> Result<()> { expectUnit(call(HostCall::ModifyBlock { spec })) }

    /// adds an entity to biome spawn lists, or with weight 0 removes it, for vanilla and other mods' mobs too
    pub fn modifySpawns(&self, spec: SpawnModifySpec) -> Result<()>
    {
        expectUnit(call(HostCall::ModifySpawns { spec }))
    }

    pub fn id(&self, name: &str) -> Result<i32> { crate::resolve(RegistryKind::Block, &ownIdentifier(name)?) }
}

pub struct BlockSpecBuilder
{
    pub m_spec: BlockSpec,
}

impl BlockSpecBuilder
{
    pub fn template(template: &str) -> BlockSpecBuilder
    {
        BlockSpecBuilder
        {
            m_spec: BlockSpec
            {
                template: super::named(Identifier::parse(template), "template"),
                ..BlockSpec::default()
            },
        }
    }

    setters!
    {
        hardness(f32) => hardness;
        resistance(f32) => resistance;
        light(f32) => light;
        material(&str) => material;
        texture(&str) => texture;
        faces(mod_protocol::spec::FaceTextures) => textures;
        color(u32) => color;
        sound(&str) => sound;
        name(&str) => name;
        drops(Vec<mod_protocol::spec::DropSpec>) => drops;
        fuel(i32) => fuel;
        model(mod_protocol::spec::BlockModelSpec) => model;
        /// a bed of the bed template, its sprites picked by half and facing
        bed(mod_protocol::spec::BedFaces) => bed;
        slipperiness(f32) => slipperiness;
        opacity(i32) => opacity;
        leaves(bool) => leaves;
        collision(Vec<mod_protocol::spec::Cuboid>) => collision;
    }

    pub fn tool(mut self, class: ToolClass, level: i32) -> Self
    {
        self.m_spec.tool = Some(ToolSpec { class, level });
        self
    }

    pub fn tickRandomly(mut self) -> Self
    {
        self.m_spec.tick_randomly = Some(true);
        self
    }

    pub fn tileEntity(mut self, tile: &str) -> Self
    {
        self.m_spec.tile_entity = super::named(ownIdentifier(tile), "tile entity");
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

    pub fn sustainsLeaves(mut self) -> Self
    {
        self.m_spec.sustains_leaves = Some(true);
        self
    }

    pub fn tag(mut self, tag: &str) -> Self
    {
        if let Some(tag) = super::named(Identifier::parse(tag), "tag")
        {
            self.m_spec.tags.push(tag);
        }

        self
    }

    pub fn vanillaFallback(mut self, block: &str) -> Self
    {
        self.m_spec.vanilla_fallback = super::named(Identifier::parse(block), "vanilla fallback");
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
