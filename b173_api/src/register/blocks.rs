use super::builders::{Property, Tile, Variant};
use crate::behavior::{self, BlockBehavior, ItemBehavior};
use crate::refs::IntoName;
use crate::rt::call;
use crate::{ItemRef, Result, dispatch, expectUnit, ownIdentifier};
use mod_protocol::Identifier;
use mod_protocol::abi::{ContentRequest, ItemModelSpec, RegistryKind};
use mod_protocol::spec::
{
    Behavior, BlockMethod, BlockModifySpec, BlockSpec, BlockStatesSpec, ItemMethod, ItemSpec, LootSpec,
    SpawnModifySpec, StateModelSpec, ToolClass, ToolSpec,
};
use std::rc::Rc;

pub struct Blocks;

impl Blocks
{
    /// the block with everything its builder declared: tile entity, states, models and drops, in the order the
    /// loader needs them
    pub fn register(&self, name: &str, spec: BlockSpecBuilder) -> Result<ItemRef>
    {
        let id: Identifier = ownIdentifier(name)?;
        let BlockSpecBuilder { m_spec: mut block, m_extras: extras } = spec;

        if let Some(tile) = &extras.m_tile
        {
            let tileSpec: mod_protocol::spec::TileEntitySpec = tile.spec();
            expectUnit(call(ContentRequest::RegisterTileEntity { name: id.to_string(), spec: tileSpec }))?;
            block.tile_entity = Some(id.clone());

            if let Some(model) = tile.m_model.clone()
            {
                expectUnit(call(ContentRequest::TileModel { tile: id.clone(), model }))?;
            }
        }

        expectUnit(call(ContentRequest::RegisterBlock { name: id.to_string(), spec: block }))?;

        if !extras.m_states.properties.is_empty()
        {
            expectUnit(call(ContentRequest::RegisterBlockStates { block: id.clone(), spec: extras.m_states }))?;
        }

        if let Some(model) = extras.m_stateModel
        {
            expectUnit(call(ContentRequest::SetStateModel { block: id.clone(), model }))?;
        }

        if let Some(model) = extras.m_itemModel
        {
            expectUnit(call(ContentRequest::BlockItemModel { block: id.clone(), model }))?;
        }

        if let Some(loot) = extras.m_loot
        {
            expectUnit(call(ContentRequest::SetLoot
            {
                registry: RegistryKind::Block,
                target: id.clone(),
                spec: loot,
            }))?;
        }

        Ok(ItemRef::Named(id))
    }

    pub fn replace(&self, id: &str, spec: BlockSpecBuilder) -> Result<()>
    {
        expectUnit(call(ContentRequest::ReplaceBlock { id: ownIdentifier(id)?, spec: spec.m_spec }))
    }

    /// named states for one of this mod's blocks, saved by name so properties may change between versions
    pub fn states(&self, name: &str, spec: BlockStatesSpec) -> Result<()>
    {
        expectUnit(call(ContentRequest::RegisterBlockStates { block: ownIdentifier(name)?, spec }))
    }

    /// changes chosen properties of any block, other mods changing the same field are reported as conflicts
    pub fn modify(&self, spec: BlockModifySpec) -> Result<()> { expectUnit(call(ContentRequest::ModifyBlock { spec })) }

    /// adds an entity to biome spawn lists, or with weight 0 removes it, for vanilla and other mods' mobs too
    /// needs api 1.33, draws the block's item in 3d from model cuboids and its tile's model instead of a flat sprite
    pub fn itemModel(&self, name: &str, model: ItemModelSpec) -> Result<()>
    {
        expectUnit(call(ContentRequest::BlockItemModel { block: ownIdentifier(name)?, model }))
    }

    /// needs api 1.45, cuboids picked by named state, each shown for the states matching its when entries
    pub fn stateModel(&self, name: &str, model: mod_protocol::spec::StateModelSpec) -> Result<()>
    {
        expectUnit(call(ContentRequest::SetStateModel { block: ownIdentifier(name)?, model }))
    }

    pub fn modifySpawns(&self, spec: SpawnModifySpec) -> Result<()>
    {
        expectUnit(call(ContentRequest::ModifySpawns { spec }))
    }

    #[deprecated(note = "keep the ItemRef register returns, or name the block where a call takes a BlockRef")]
    pub fn id(&self, name: &str) -> Result<i32> { crate::resolve(RegistryKind::Block, &ownIdentifier(name)?) }
}

pub struct BlockSpecBuilder
{
    pub m_spec: BlockSpec,
    m_extras: Extras,
}

// what a block declares beside its spec, sent as separate calls by register
#[derive(Default)]
struct Extras
{
    m_tile: Option<Tile>,
    m_states: BlockStatesSpec,
    m_stateModel: Option<StateModelSpec>,
    m_itemModel: Option<ItemModelSpec>,
    m_loot: Option<LootSpec>,
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
            m_extras: Extras::default(),
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

    /// binds a tile entity registered elsewhere, or shared with other blocks
    pub fn tileEntity(mut self, tile: impl IntoName) -> Self
    {
        self.m_spec.tile_entity = super::named(tile.intoName(), "tile entity");
        self
    }

    /// a tile entity of the block's own name, registered with it
    pub fn tile(mut self, tile: Tile) -> Self
    {
        self.m_extras.m_tile = Some(tile);
        self
    }

    /// a named state property, saved by name so values may change between versions
    pub fn property(mut self, property: Property) -> Self
    {
        self.m_extras.m_states.properties.push(property.m_spec);
        self
    }

    /// the state a placed block starts in, by default every property's first value
    pub fn defaultState(mut self, property: &str, value: &str) -> Self
    {
        self.m_extras.m_states.default.insert(property.to_owned(), value.to_owned());
        self
    }

    /// textures for the states that match the variant
    pub fn variant(mut self, variant: Variant) -> Self
    {
        self.m_extras.m_states.variants.push(variant.m_spec);
        self
    }

    /// cuboids picked by named state
    pub fn stateModel(mut self, model: StateModelSpec) -> Self
    {
        self.m_extras.m_stateModel = Some(model);
        self
    }

    /// the block's item drawn in 3d from model cuboids and its tile's model
    pub fn itemModel(mut self, model: ItemModelSpec) -> Self
    {
        self.m_extras.m_itemModel = Some(model);
        self
    }

    /// drops added to the block's own, replace drops what beta or other mods added before
    pub fn loot(mut self, loot: LootSpec) -> Self
    {
        self.m_extras.m_loot = Some(loot);
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
