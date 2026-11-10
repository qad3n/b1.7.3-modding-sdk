use crate::behavior::{self, BlockBehavior, EntityBehavior, ItemBehavior, TileBehavior};
use crate::dispatch::{self, Command};
use crate::events::{Event, EventType};
use crate::rt::call;
use crate::screen::{Draw, Screen};
use crate::services::{Lookup, Service};
use crate::terrain::ChunkBlocks;
use crate::{Config, Error, ItemRef, ItemStack, Player, Result, expectUnit, ownId};
use mod_protocol::Identifier;
use mod_protocol::abi::
{
    BiomeSpec, BlockPos, ChunkInfo, DimensionSpec, EventKind, HostCall, HostReply, Permission, Priority, RegistryKind,
    ScreenSpec, SlotSpec, SpawnEntry, StructureSpec,
};
use mod_protocol::handshake::ChannelDirection;
use mod_protocol::spec::
{
    Behavior, BlockModifySpec, BlockSpec, BlockStatesSpec, CreatureKind, EntitySpec, FuelSpec, ItemSpec, RecipeSpec,
    RecordSpec, ShapedSpec, ShapelessSpec, SmeltingSpec, SpawnModifySpec, StackSpec, TileEntitySpec, ToolClass,
    ToolSpec, WorldgenKind, WorldgenSpec,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;
use std::rc::Rc;

fn parse(name: &str) -> Result<Identifier>
{
    Identifier::parseIn(name, &crate::modId()).map_err(|error| Error(error.to_string()))
}

fn stackSpec(stack: &ItemStack) -> Result<StackSpec>
{
    let item: Identifier = stack.item.name().ok_or("a recipe stack needs a registered item")?;
    Ok(StackSpec { item, count: stack.count, meta: stack.damage })
}

// the context handed to #[mod_main], registration only works while the loader is in its register phase
pub struct RegisterCtx
{
    pub(crate) m_modId: String,
}

impl RegisterCtx
{
    pub fn modId(&self) -> &str { &self.m_modId }

    pub fn blocks(&mut self) -> Blocks { Blocks }

    pub fn items(&mut self) -> Items { Items }

    pub fn entities(&mut self) -> Entities { Entities }

    pub fn tileEntities(&mut self) -> TileEntities { TileEntities }

    pub fn recipes(&mut self) -> Recipes { Recipes }

    pub fn events(&mut self) -> Events { Events }

    pub fn commands(&mut self) -> Commands { Commands }

    pub fn keybinds(&mut self) -> Keybinds { Keybinds }

    pub fn channels(&mut self) -> Channels { Channels }

    pub fn screens(&mut self) -> Screens { Screens }

    pub fn config(&mut self) -> Config { Config }

    pub fn services(&mut self) -> Services { Services }

    pub fn tags(&mut self) -> Tags { Tags }

    pub fn worldgen(&mut self) -> Worldgen { Worldgen }

    pub fn dimensions(&mut self) -> Dimensions { Dimensions }

    pub fn structures(&mut self) -> Structures { Structures }

    pub fn lang(&mut self) -> Lang { Lang }
}

pub struct Blocks;

impl Blocks
{
    pub fn register(&self, name: &str, spec: BlockSpecBuilder) -> Result<ItemRef>
    {
        let id: Identifier = parse(name)?;
        expectUnit(call(HostCall::RegisterBlock { name: id.to_string(), spec: spec.m_spec }))?;
        Ok(ItemRef::Named(id))
    }

    pub fn replace(&self, id: &str, spec: BlockSpecBuilder) -> Result<()>
    {
        expectUnit(call(HostCall::ReplaceBlock { id: parse(id)?, spec: spec.m_spec }))
    }

    // named states for one of this mod's blocks, saved by name so properties may change between versions
    pub fn states(&self, name: &str, spec: BlockStatesSpec) -> Result<()>
    {
        expectUnit(call(HostCall::RegisterBlockStates { block: parse(name)?, spec }))
    }

    // changes chosen properties of any block, other mods changing the same field are reported as conflicts
    pub fn modify(&self, spec: BlockModifySpec) -> Result<()> { expectUnit(call(HostCall::ModifyBlock { spec })) }

    // adds an entity to biome spawn lists, or with weight 0 removes it, for vanilla and other mods' mobs too
    pub fn modifySpawns(&self, spec: SpawnModifySpec) -> Result<()>
    {
        expectUnit(call(HostCall::ModifySpawns { spec }))
    }

    pub fn id(&self, name: &str) -> Result<i32> { crate::resolve(RegistryKind::Block, &parse(name)?) }
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
        self.m_spec.tile_entity = parse(tile).ok();
        self
    }

    pub fn fuel(mut self, ticks: i32) -> Self
    {
        self.m_spec.fuel = Some(ticks);
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

    // a bed of the bed template, its sprites picked by half and facing
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
        let methods = behavior::methods(behavior.overridden(), &behavior::BLOCK_METHODS);
        let handle: u32 = dispatch::addBlock(Rc::new(behavior));
        self.m_spec.behavior = Some(Behavior { handle, methods });
        self
    }
}

pub struct Items;

impl Items
{
    pub fn register(&self, name: &str, spec: ItemSpecBuilder) -> Result<ItemRef>
    {
        let id: Identifier = parse(name)?;
        expectUnit(call(HostCall::RegisterItem { name: id.to_string(), spec: spec.m_spec }))?;
        Ok(ItemRef::Named(id))
    }

    // content from data/ files and other mods registers before code runs, so lookups by name work here
    pub fn get(&self, name: &str) -> Result<ItemRef>
    {
        let id: Identifier = parse(name)?;
        Ok(ItemRef::Named(id))
    }
}

pub struct ItemSpecBuilder
{
    pub m_spec: ItemSpec,
}

impl ItemSpecBuilder
{
    pub fn new() -> ItemSpecBuilder { ItemSpecBuilder { m_spec: ItemSpec::default() } }

    pub fn template(template: &str) -> ItemSpecBuilder
    {
        ItemSpecBuilder { m_spec: ItemSpec { template: Identifier::parse(template).ok(), ..ItemSpec::default() } }
    }

    pub fn texture(mut self, texture: &str) -> Self
    {
        self.m_spec.texture = Some(texture.to_owned());
        self
    }

    pub fn name(mut self, name: &str) -> Self
    {
        self.m_spec.name = Some(name.to_owned());
        self
    }

    pub fn maxStack(mut self, size: i32) -> Self
    {
        self.m_spec.max_stack = Some(size);
        self
    }

    pub fn maxDamage(mut self, damage: i32) -> Self
    {
        self.m_spec.max_damage = Some(damage);
        self
    }

    pub fn food(mut self, heal: i32) -> Self
    {
        self.m_spec.food = Some(mod_protocol::spec::FoodSpec { heal, wolf_food: false });
        self
    }

    pub fn tool(mut self, class: ToolClass, level: i32, speed: f32, damage: i32, durability: i32) -> Self
    {
        self.m_spec.tool = Some(mod_protocol::spec::ItemToolSpec { class, level, speed, damage, durability });
        self.m_spec.full3d = Some(true);
        self
    }

    pub fn armor(mut self, armor: mod_protocol::spec::ArmorSpec) -> Self
    {
        self.m_spec.armor = Some(armor);
        self
    }

    pub fn fuel(mut self, ticks: i32) -> Self
    {
        self.m_spec.fuel = Some(ticks);
        self
    }

    pub fn color(mut self, color: u32) -> Self
    {
        self.m_spec.color = Some(color);
        self
    }

    pub fn record(mut self, sound: &str, title: &str) -> Self
    {
        self.m_spec.record = Some(RecordSpec { sound: sound.to_owned(), title: title.to_owned() });
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

    pub fn behavior(mut self, behavior: impl ItemBehavior + 'static) -> Self
    {
        let methods = behavior::methods(behavior.overridden(), &behavior::ITEM_METHODS);
        let handle: u32 = dispatch::addItem(Rc::new(behavior));
        self.m_spec.behavior = Some(Behavior { handle, methods });
        self
    }
}

impl Default for ItemSpecBuilder
{
    fn default() -> Self { ItemSpecBuilder::new() }
}

pub struct Entities;

impl Entities
{
    pub fn register(&self, name: &str, spec: EntitySpec, behavior: Option<Rc<dyn EntityBehavior>>) -> Result<()>
    {
        let mut spec: EntitySpec = spec;
        if let Some(behavior) = behavior
        {
            let methods = behavior::methods(behavior.overridden(), &behavior::ENTITY_METHODS);
            spec.behavior = Some(Behavior { handle: dispatch::addEntity(behavior), methods });
        }
        expectUnit(call(HostCall::RegisterEntity { name: parse(name)?.to_string(), spec }))
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
            let methods = behavior::methods(behavior.overridden(), &behavior::TILE_METHODS);
            spec.behavior = Some(Behavior { handle: dispatch::addTile(behavior), methods });
        }
        expectUnit(call(HostCall::RegisterTileEntity { name: parse(name)?.to_string(), spec }))
    }
}

pub struct Recipes;

pub struct ShapedBuilder
{
    m_name: String,
    m_result: ItemStack,
    m_pattern: Vec<String>,
    m_key: BTreeMap<String, StackSpec>,
    m_error: Option<Error>,
}

impl ShapedBuilder
{
    pub fn pattern<const N: usize>(self, rows: [&str; N]) -> Self { self.rows(&rows) }

    // the pattern as a slice, for recipe tables whose rows differ in count
    pub fn rows(mut self, rows: &[&str]) -> Self
    {
        self.m_pattern = rows.iter().map(|row| (*row).to_owned()).collect();
        self
    }

    pub fn key(self, symbol: char, item: impl Into<ItemRef>) -> Self { self.keyStack(symbol, ItemStack::of(item, 1)) }

    // a key that needs one metadata, or any metadata with damage -1
    pub fn keyStack(mut self, symbol: char, stack: ItemStack) -> Self
    {
        match stackSpec(&stack)
        {
            Ok(stack) =>
            {
                self.m_key.insert(symbol.to_string(), stack);
            }
            Err(error) => self.m_error = Some(error),
        }
        self
    }

    pub fn build(self) -> Result<()>
    {
        if let Some(error) = self.m_error
        {
            return Err(error);
        }
        let spec: RecipeSpec = RecipeSpec::Shaped(ShapedSpec {
            pattern: self.m_pattern,
            key: self.m_key,
            result: stackSpec(&self.m_result)?,
        });
        expectUnit(call(HostCall::RegisterRecipe { name: self.m_name, spec }))
    }
}

impl Recipes
{
    pub fn shaped(&self, name: &str, result: ItemStack) -> ShapedBuilder
    {
        ShapedBuilder {
            m_name: name.to_owned(),
            m_result: result,
            m_pattern: Vec::new(),
            m_key: BTreeMap::new(),
            m_error: None,
        }
    }

    pub fn shapeless(&self, name: &str, result: ItemStack, ingredients: &[ItemStack]) -> Result<()>
    {
        let ingredients: Vec<StackSpec> = ingredients.iter().map(stackSpec).collect::<Result<_>>()?;
        let spec: RecipeSpec = RecipeSpec::Shapeless(ShapelessSpec { ingredients, result: stackSpec(&result)? });
        expectUnit(call(HostCall::RegisterRecipe { name: name.to_owned(), spec }))
    }

    pub fn smelting(&self, name: &str, input: impl Into<ItemRef>, result: ItemStack) -> Result<()>
    {
        let input: Identifier = input.into().name().ok_or("smelting input is not registered")?;
        let spec: RecipeSpec = RecipeSpec::Smelting(SmeltingSpec { input, result: stackSpec(&result)? });
        expectUnit(call(HostCall::RegisterRecipe { name: name.to_owned(), spec }))
    }

    pub fn fuel(&self, name: &str, item: impl Into<ItemRef>, ticks: i32) -> Result<()>
    {
        let item: Identifier = item.into().name().ok_or("fuel item is not registered")?;
        expectUnit(call(HostCall::RegisterRecipe {
            name: name.to_owned(),
            spec: RecipeSpec::Fuel(FuelSpec { item, ticks }),
        }))
    }

    pub fn remove(&self, id: &str) -> Result<()> { expectUnit(call(HostCall::RemoveRecipe { id: parse(id)? })) }
}

pub struct Events;

impl Events
{
    pub fn on<E: EventType>(&self, priority: Priority, handler: impl Fn(&mut Event) + 'static) -> Result<()>
    {
        self.onKind(E::KIND, priority, false, handler)
    }

    pub fn onKind(
        &self,
        kind: EventKind,
        priority: Priority,
        receiveCanceled: bool,
        handler: impl Fn(&mut Event) + 'static,
    ) -> Result<()>
    {
        let handle: u32 = dispatch::addEvent(Rc::new(handler));
        expectUnit(call(HostCall::Subscribe {
            kind,
            custom: None,
            priority,
            receive_canceled: receiveCanceled,
            handler: handle,
        }))
    }

    // a custom event another mod defined, the defining mod must load first
    pub fn onCustom(&self, event: &str, priority: Priority, handler: impl Fn(&mut Event) + 'static) -> Result<()>
    {
        let handle: u32 = dispatch::addEvent(Rc::new(handler));
        let custom: Identifier = parse(event)?;
        expectUnit(call(HostCall::Subscribe {
            kind: EventKind::Custom,
            custom: Some(custom),
            priority,
            receive_canceled: false,
            handler: handle,
        }))
    }

    pub fn define(&self, event: &str) -> Result<()> { expectUnit(call(HostCall::DefineEvent { id: parse(event)? })) }
}

pub struct Commands;

impl Commands
{
    pub fn register(
        &self,
        name: &str,
        permission: Permission,
        handler: impl Fn(&mut Command) -> Result<()> + 'static,
    ) -> Result<()>
    {
        self.registerWithUsage(name, permission, "", handler)
    }

    pub fn registerWithUsage(
        &self,
        name: &str,
        permission: Permission,
        usage: &str,
        handler: impl Fn(&mut Command) -> Result<()> + 'static,
    ) -> Result<()>
    {
        let handle: u32 = dispatch::addCommand(Rc::new(handler));
        expectUnit(call(HostCall::RegisterCommand {
            name: name.to_owned(),
            permission,
            usage: usage.to_owned(),
            handler: handle,
        }))
    }
}

pub struct Keybinds;

impl Keybinds
{
    // key is an LWJGL key code, the handler runs on press and release
    pub fn register(&self, name: &str, key: i32, handler: impl Fn(bool) + 'static) -> Result<()>
    {
        let handle: u32 = dispatch::addKey(Rc::new(handler));
        expectUnit(call(HostCall::RegisterKeybind { name: name.to_owned(), key, handler: handle }))
    }
}

pub struct Channels;

impl Channels
{
    // typed payloads with any serde type, handlers run on the main thread and must validate serverbound input
    pub fn register<T: DeserializeOwned + 'static>(
        &self,
        name: &str,
        direction: ChannelDirection,
        handler: impl Fn(T, Player) + 'static,
    ) -> Result<()>
    {
        let handle: u32 =
            dispatch::addChannel(Rc::new(move |player: Player, bytes: &[u8]| match crate::world::decode::<T>(bytes)
            {
                Ok(message) => handler(message, player),
                Err(error) => crate::warn(&format!("bad payload: {error}")),
            }));
        expectUnit(call(HostCall::RegisterChannel {
            name: name.to_owned(),
            version: "1".to_owned(),
            direction,
            handler: handle,
        }))
    }

    pub fn registerRaw(
        &self,
        name: &str,
        version: &str,
        direction: ChannelDirection,
        handler: impl Fn(Player, &[u8]) + 'static,
    ) -> Result<()>
    {
        let handle: u32 = dispatch::addChannel(Rc::new(handler));
        expectUnit(call(HostCall::RegisterChannel {
            name: name.to_owned(),
            version: version.to_owned(),
            direction,
            handler: handle,
        }))
    }
}

pub struct Screens;

pub struct ScreenBuilder
{
    pub m_spec: ScreenSpec,
}

impl ScreenBuilder
{
    pub fn new(title: &str, width: i32, height: i32) -> ScreenBuilder
    {
        ScreenBuilder { m_spec: ScreenSpec { width, height, title: title.to_owned(), ..ScreenSpec::default() } }
    }

    pub fn transparent(mut self) -> Self
    {
        self.m_spec.transparent = true;
        self
    }

    pub fn background(mut self, texture: &str) -> Self
    {
        self.m_spec.background = Some(texture.to_owned());
        self
    }

    pub fn slot(mut self, index: i32, x: i32, y: i32) -> Self
    {
        self.m_spec.slots.push(SlotSpec { index, x, y, output: false });
        self
    }

    pub fn outputSlot(mut self, index: i32, x: i32, y: i32) -> Self
    {
        self.m_spec.slots.push(SlotSpec { index, x, y, output: true });
        self
    }

    pub fn playerInventory(mut self, x: i32, y: i32) -> Self
    {
        self.m_spec.player_inventory = Some([x, y]);
        self
    }
}

impl Screens
{
    pub fn register(&self, name: &str, spec: ScreenBuilder, screen: impl Screen + 'static) -> Result<()>
    {
        let handle: u32 = dispatch::addScreen(screen);
        expectUnit(call(HostCall::RegisterScreen { name: name.to_owned(), spec: spec.m_spec, handler: handle }))
    }

    // layers over every player model, from the player and its synced bytes
    pub fn playerLayer(
        &self,
        handler: impl Fn(&mod_protocol::abi::EntityInfo, &[u8]) -> Vec<mod_protocol::abi::PlayerLayer> + 'static,
    ) -> Result<()>
    {
        let handle: u32 = dispatch::addLayer(Rc::new(handler));
        expectUnit(call(HostCall::RegisterPlayerLayer { handler: handle }))
    }

    pub fn overlay(&self, target: &str, overlay: impl crate::screen::Overlay + 'static) -> Result<()>
    {
        let handle: u32 = dispatch::addOverlay(overlay);
        expectUnit(call(HostCall::RegisterOverlay { target: target.to_owned(), handler: handle }))
    }

    // needs api 1.4, lines pushed here show under the item name in every vanilla slot tooltip
    pub fn tooltip(&self, handler: impl Fn(&ItemStack, &mut Vec<String>) + 'static) -> Result<()>
    {
        let handle: u32 = dispatch::addTooltip(Rc::new(handler));
        expectUnit(call(HostCall::RegisterTooltip { handler: handle }))
    }

    // needs api 1.8, ops drawn after the world each frame, in block units from the camera position the handler gets
    pub fn worldRender(
        &self,
        handler: impl Fn(f32, [f64; 3]) -> Vec<mod_protocol::abi::ModelOp> + 'static,
    ) -> Result<()>
    {
        let handle: u32 = dispatch::addWorldRender(Rc::new(handler));
        expectUnit(call(HostCall::RegisterWorldRender { handler: handle }))
    }

    pub fn hud(&self, handler: impl Fn(&mut Draw, f32, i32, i32) + 'static) -> Result<()>
    {
        let handle: u32 = dispatch::addHud(Rc::new(handler));
        expectUnit(call(HostCall::RegisterHud { handler: handle }))
    }
}

pub struct Services;

impl Services
{
    pub fn provide(&self, name: &str, version: u32, service: impl Service + 'static) -> Result<()>
    {
        let handle: u32 = dispatch::addService(Rc::new(service));
        expectUnit(call(HostCall::ProvideService {
            name: Identifier::parse(name).map_err(|error| Error(error.to_string()))?,
            version,
            handler: handle,
        }))
    }

    pub fn lookup(&self, api: &str, block: &str, lookup: impl Lookup + 'static) -> Result<()>
    {
        let handle: u32 = dispatch::addLookup(Rc::new(lookup));
        let api: Identifier = Identifier::parse(api).map_err(|error| Error(error.to_string()))?;
        expectUnit(call(HostCall::RegisterLookup { api, block: parse(block)?, handler: handle }))
    }

    pub fn onInterMod(&self, handler: impl Fn(&str, &str, &[u8]) + 'static) { dispatch::setInterMod(Rc::new(handler)); }
}

pub struct Tags;

impl Tags
{
    pub fn add(&self, registry: RegistryKind, tag: &str, values: &[&str]) -> Result<()>
    {
        let values: Vec<Identifier> = values.iter().map(|value| parse(value)).collect::<Result<_>>()?;
        expectUnit(call(HostCall::AddTag {
            registry,
            tag: Identifier::parse(tag).map_err(|error| Error(error.to_string()))?,
            values,
        }))
    }

    pub fn values(&self, registry: RegistryKind, tag: &str) -> Vec<Identifier> { tagValues(registry, tag) }
}

pub fn tagValues(registry: RegistryKind, tag: &str) -> Vec<Identifier>
{
    let Ok(tag) = Identifier::parse(tag)
    else
    {
        return Vec::new();
    };
    match call(HostCall::TagValues { registry, tag })
    {
        HostReply::Ids(ids) => ids,
        _ => Vec::new(),
    }
}

pub struct Worldgen;

impl Worldgen
{
    pub fn ore(&self, name: &str, block: &str, veinSize: i32, perChunk: i32, y: [i32; 2]) -> Result<()>
    {
        let spec: WorldgenSpec = WorldgenSpec {
            kind: WorldgenKind::Ore,
            block: parse(block)?,
            dimension: Identifier::minecraft("overworld"),
            vein_size: veinSize,
            per_chunk: perChunk,
            y,
            replaces: None,
            biomes: Vec::new(),
        };
        expectUnit(call(HostCall::RegisterWorldgen { name: name.to_owned(), spec }))
    }

    pub fn feature(&self, name: &str, spec: WorldgenSpec) -> Result<()>
    {
        expectUnit(call(HostCall::RegisterWorldgen { name: name.to_owned(), spec }))
    }
}

pub struct Dimensions;

impl Dimensions
{
    // the handle it returns sets everything else about the dimension, so handlers never name the wrong one
    pub fn register(&self, name: &str, spec: DimensionSpec) -> Result<Dimension>
    {
        expectUnit(call(HostCall::RegisterDimension { name: name.to_owned(), spec }))?;
        Ok(Dimension { m_id: parse(name)? })
    }

    pub fn biome(&self, name: &str, spec: BiomeSpec) -> Result<()>
    {
        expectUnit(call(HostCall::RegisterBiome { name: name.to_owned(), spec }))
    }

    // the generator runs once per populated chunk of the dimension, vanilla dimensions take generators too
    pub fn generator(&self, dimension: &str, handler: impl Fn(ChunkInfo) + 'static) -> Result<()>
    {
        addGenerator(parse(dimension)?, handler)
    }

    // needs api 1.7, the mod names the biome of every column, vanilla spawning snow and grass colors follow it
    pub fn biomeSource(
        &self,
        dimension: &str,
        biomes: &[&str],
        handler: impl Fn(&ChunkInfo) -> crate::ChunkBiomes + 'static,
    ) -> Result<()>
    {
        addBiomeSource(parse(dimension)?, biomes, handler)
    }

    // the terrain generator fills every new chunk of a mod dimension, the generator above then populates it
    pub fn terrain(&self, dimension: &str, handler: impl Fn(&ChunkInfo, &mut ChunkBlocks) + 'static) -> Result<()>
    {
        addTerrain(parse(dimension)?, handler)
    }
}

// one dimension this mod registered, each call sets one part of it and hands the dimension back for the next
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Dimension
{
    m_id: Identifier,
}

impl Dimension
{
    pub fn id(&self) -> &Identifier { &self.m_id }

    // the numeric id worlds of it carry, known once registration is over
    pub fn numeric(&self) -> Option<i32> { crate::resolve(RegistryKind::Dimension, &self.m_id).ok() }

    // fills every new chunk, 128 high, before the generators populate it
    pub fn terrain(&self, handler: impl Fn(&ChunkInfo, &mut ChunkBlocks) + 'static) -> Result<&Self>
    {
        addTerrain(self.m_id.clone(), handler)?;
        Ok(self)
    }

    // runs once per populated chunk, structures and decoration go here
    pub fn generator(&self, handler: impl Fn(ChunkInfo) + 'static) -> Result<&Self>
    {
        addGenerator(self.m_id.clone(), handler)?;
        Ok(self)
    }

    // needs api 1.7, the biome of every column, an index into biomes
    pub fn biomeSource(
        &self,
        biomes: &[&str],
        handler: impl Fn(&ChunkInfo) -> crate::ChunkBiomes + 'static,
    ) -> Result<&Self>
    {
        addBiomeSource(self.m_id.clone(), biomes, handler)?;
        Ok(self)
    }

    // needs api 1.9, a world as tall as height, a multiple of 16 up to 256; generators and structures fill all of it
    // while players place blocks only below the build limit, terrain handlers still fill the bottom 128
    pub fn height(&self, height: i32, buildLimit: i32) -> Result<&Self>
    {
        expectUnit(call(HostCall::DimensionHeight { dimension: self.m_id.clone(), height, build_limit: buildLimit }))?;
        Ok(self)
    }

    // needs api 1.9, the creatures that may spawn at each natural spawn attempt, None keeps the biome's own list and
    // an empty list spawns nothing there; with this the biome spawn lists of the dimension go unused
    pub fn spawns(&self, handler: impl Fn(&SpawnQuery) -> Option<Vec<SpawnEntry>> + 'static) -> Result<&Self>
    {
        let handle: u32 = dispatch::addSpawnList(Rc::new(handler));
        expectUnit(call(HostCall::RegisterSpawns { dimension: self.m_id.clone(), handler: handle }))?;
        Ok(self)
    }
}

pub struct Structures;

impl Structures
{
    // needs api 1.10, a kind of structure this mod builds; the dimension is a bare name for this mod's own or a full
    // identifier such as minecraft:overworld
    pub fn register(&self, name: &str, label: &str, dimension: &str) -> Result<Structure>
    {
        addStructure(name, label, dimension, None)
    }

    // needs api 1.10, as register with a finder that answers whether one starts in a chunk of a seed, so locate also
    // finds the ones no chunk has built yet
    pub fn withFinder(
        &self,
        name: &str,
        label: &str,
        dimension: &str,
        finder: impl Fn(i64, i32, i32) -> bool + 'static,
    ) -> Result<Structure>
    {
        addStructure(name, label, dimension, Some(dispatch::addFinder(Rc::new(finder))))
    }
}

fn addStructure(name: &str, label: &str, dimension: &str, finder: Option<u32>) -> Result<Structure>
{
    let spec: StructureSpec = StructureSpec { name: label.to_owned(), dimension: parse(dimension)? };
    expectUnit(call(HostCall::RegisterStructure { name: name.to_owned(), spec, finder }))?;
    Ok(Structure { m_id: parse(name)? })
}

// one structure kind this mod registered
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Structure
{
    m_id: Identifier,
}

impl Structure
{
    pub fn id(&self) -> &Identifier { &self.m_id }
}

// one natural spawn attempt a spawn list handler answers
#[derive(Clone, Copy, Debug)]
pub struct SpawnQuery
{
    pub world: crate::world::World,
    pub kind: CreatureKind,
    pub pos: BlockPos,
}

// a weighted spawn list entry, vanilla names such as minecraft:zombie or identifiers, bare names in this mod
pub fn spawnEntry(entity: &str, weight: i32) -> Result<SpawnEntry> { Ok(SpawnEntry { entity: parse(entity)?, weight }) }

fn addGenerator(dimension: Identifier, handler: impl Fn(ChunkInfo) + 'static) -> Result<()>
{
    let handle: u32 = dispatch::addGenerator(Rc::new(handler));
    expectUnit(call(HostCall::RegisterGenerator { dimension, handler: handle }))
}

fn addTerrain(dimension: Identifier, handler: impl Fn(&ChunkInfo, &mut ChunkBlocks) + 'static) -> Result<()>
{
    let handle: u32 = dispatch::addTerrain(Rc::new(handler));
    expectUnit(call(HostCall::RegisterTerrain { dimension, handler: handle }))
}

fn addBiomeSource(
    dimension: Identifier,
    biomes: &[&str],
    handler: impl Fn(&ChunkInfo) -> crate::ChunkBiomes + 'static,
) -> Result<()>
{
    let biomes: Vec<Identifier> = biomes.iter().map(|name| parse(name)).collect::<Result<_>>()?;
    let handle: u32 = dispatch::addBiomeSource(Rc::new(handler));
    expectUnit(call(HostCall::RegisterBiomeSource { dimension, biomes, handler: handle }))
}

pub struct Lang;

impl Lang
{
    pub fn translate(&self, key: &str) -> String { crate::world::translate(key) }
}

pub fn ownIdentifier(name: &str) -> Result<Identifier> { ownId(name) }

pub fn bytes<T: Serialize>(value: &T) -> Vec<u8> { crate::world::encode(value) }

pub fn fromBytes<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> { crate::world::decode(bytes) }

pub fn toBytes<T: Serialize>(value: &T) -> Vec<u8> { crate::world::encode(value) }
