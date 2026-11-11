use crate::behavior::{BlockBehavior, EffectBehavior, EntityBehavior, ItemBehavior, TileBehavior, Use};
use crate::events::Event;
use crate::screen::{Draw, Screen};
use crate::services::{Lookup, Service};
use crate::terrain::ChunkBlocks;
use crate::{BlockCtx, Entity, ItemStack, Player, RegisterCtx, World};
use mod_protocol::abi::
{
    BlockCall, ChunkInfo, EffectCall, EntityCall, GuestCall, GuestReply, ItemCall, Lifecycle, Sender, TileCall,
};
use std::cell::{RefCell, RefMut};
use std::rc::Rc;

/// a running command: who sent it, its arguments and the reply shown to the sender
pub struct Command
{
    pub sender: Sender,
    pub args: Vec<String>,
    pub(crate) m_reply: String,
}

impl Command
{
    pub fn player(&self) -> Option<Player> { self.sender.player.map(|id| Player { id }) }

    pub fn world(&self) -> World { World { dimension: self.sender.world } }

    pub fn reply(&mut self, text: &str)
    {
        if !self.m_reply.is_empty()
        {
            self.m_reply.push('\n');
        }

        self.m_reply.push_str(text);
    }

    pub fn arg(&self, index: usize) -> Option<&str> { self.args.get(index).map(String::as_str) }
}

type EventHandler = Rc<dyn Fn(&mut Event)>;
type CommandHandler = Rc<dyn Fn(&mut Command) -> crate::Result<()>>;
type ChannelHandler = Rc<dyn Fn(Player, &[u8])>;
type HudHandler = Rc<dyn Fn(&mut Draw, f32, i32, i32)>;
type LayerHandler = Rc<dyn Fn(&mod_protocol::abi::EntityInfo, &[u8]) -> Vec<mod_protocol::abi::PlayerLayer>>;
type InterModHandler = Rc<dyn Fn(&str, &str, &[u8])>;
type TerrainHandler = Rc<dyn Fn(&ChunkInfo, &mut ChunkBlocks)>;
type BiomeHandler = Rc<dyn Fn(&ChunkInfo) -> crate::ChunkBiomes>;
type SpawnHandler = Rc<dyn Fn(&crate::SpawnQuery) -> Option<Vec<mod_protocol::abi::SpawnEntry>>>;
type FinderHandler = Rc<dyn Fn(i64, i32, i32) -> bool>;
type ConfigListener = Rc<dyn Fn(&str)>;
type TooltipHandler = Rc<dyn Fn(&ItemStack, &mut Vec<String>)>;
type WorldRenderHandler = Rc<dyn Fn(f32, [f64; 3]) -> Vec<mod_protocol::abi::ModelOp>>;

#[derive(Default)]
struct Tables
{
    m_blocks: Vec<Rc<dyn BlockBehavior>>,
    m_items: Vec<Rc<dyn ItemBehavior>>,
    m_entities: Vec<Rc<dyn EntityBehavior>>,
    m_tiles: Vec<Rc<dyn TileBehavior>>,
    m_effects: Vec<Rc<dyn EffectBehavior>>,
    m_events: Vec<EventHandler>,
    m_commands: Vec<CommandHandler>,
    m_keys: Vec<Rc<dyn Fn(bool)>>,
    m_channels: Vec<ChannelHandler>,
    m_screens: Vec<Rc<RefCell<dyn Screen>>>,
    m_huds: Vec<HudHandler>,
    m_overlays: Vec<Rc<RefCell<dyn crate::screen::Overlay>>>,
    m_layers: Vec<LayerHandler>,
    m_services: Vec<Rc<dyn Service>>,
    m_lookups: Vec<Rc<dyn Lookup>>,
    m_generators: Vec<Rc<dyn Fn(ChunkInfo)>>,
    m_terrains: Vec<TerrainHandler>,
    m_biomeSources: Vec<BiomeHandler>,
    m_spawnLists: Vec<SpawnHandler>,
    m_finders: Vec<FinderHandler>,
    m_interMod: Option<InterModHandler>,
    m_configListeners: Vec<ConfigListener>,
    m_tooltips: Vec<TooltipHandler>,
    m_worldRenders: Vec<WorldRenderHandler>,
}

thread_local!
{
    static TABLES: RefCell<Tables> = RefCell::new(Tables::default());
}

macro_rules! adder
{
    ($name:ident, $field:ident, $kind:ty) =>
    {
        pub(crate) fn $name(value: $kind) -> u32
        {
            TABLES.with(|tables|
            {
                let mut tables: RefMut<Tables> = tables.borrow_mut();
                tables.$field.push(value);
                (tables.$field.len() - 1) as u32
            })
        }
    };
}

adder!(addBlock, m_blocks, Rc<dyn BlockBehavior>);
adder!(addItem, m_items, Rc<dyn ItemBehavior>);
adder!(addEntity, m_entities, Rc<dyn EntityBehavior>);
adder!(addTile, m_tiles, Rc<dyn TileBehavior>);
adder!(addEffect, m_effects, Rc<dyn EffectBehavior>);
adder!(addEvent, m_events, EventHandler);
adder!(addCommand, m_commands, CommandHandler);
adder!(addKey, m_keys, Rc<dyn Fn(bool)>);
adder!(addChannel, m_channels, ChannelHandler);
adder!(addHud, m_huds, HudHandler);
adder!(addLayer, m_layers, LayerHandler);
adder!(addService, m_services, Rc<dyn Service>);
adder!(addLookup, m_lookups, Rc<dyn Lookup>);
adder!(addGenerator, m_generators, Rc<dyn Fn(ChunkInfo)>);
adder!(addTerrain, m_terrains, TerrainHandler);
adder!(addBiomeSource, m_biomeSources, BiomeHandler);
adder!(addSpawnList, m_spawnLists, SpawnHandler);
adder!(addFinder, m_finders, FinderHandler);
adder!(addConfigListener, m_configListeners, ConfigListener);
adder!(addTooltip, m_tooltips, TooltipHandler);
adder!(addWorldRender, m_worldRenders, WorldRenderHandler);

pub(crate) fn addOverlay(overlay: impl crate::screen::Overlay + 'static) -> u32
{
    TABLES.with(|tables|
    {
        let mut tables: RefMut<Tables> = tables.borrow_mut();
        tables.m_overlays.push(Rc::new(RefCell::new(overlay)));
        (tables.m_overlays.len() - 1) as u32
    })
}

pub(crate) fn addScreen(screen: impl Screen + 'static) -> u32
{
    TABLES.with(|tables|
    {
        let mut tables: RefMut<Tables> = tables.borrow_mut();
        tables.m_screens.push(Rc::new(RefCell::new(screen)));
        (tables.m_screens.len() - 1) as u32
    })
}

pub(crate) fn setInterMod(handler: InterModHandler)
{
    TABLES.with(|tables| tables.borrow_mut().m_interMod = Some(handler));
}

// handlers are cloned out first, so a handler may register or call back into the host freely
fn get<T: Clone>(pick: impl FnOnce(&Tables) -> Option<&T>) -> Option<T>
{
    TABLES.with(|tables| pick(&tables.borrow()).cloned())
}

fn block(handle: u32, call: BlockCall) -> GuestReply
{
    let Some(behavior) = get(|tables| tables.m_blocks.get(handle as usize))
    else
    {
        return GuestReply::Error(format!("no block behavior {handle}"));
    };

    match call
    {
        BlockCall::Activated { ctx, player } =>
        {
            GuestReply::Bool(behavior.onActivated(&mut wrap(ctx), Player { id: player }))
        }

        BlockCall::Clicked { ctx, player } => unit(|| behavior.onClicked(&mut wrap(ctx), Player { id: player })),
        BlockCall::Added { ctx } => unit(|| behavior.onAdded(&mut wrap(ctx))),
        BlockCall::Removed { ctx } => unit(|| behavior.onRemoved(&mut wrap(ctx))),
        BlockCall::Placed { ctx, face } => unit(|| behavior.onPlaced(&mut wrap(ctx), face)),

        BlockCall::PlacedBy { ctx, entity } =>
        {
            unit(|| behavior.onPlacedBy(&mut wrap(ctx), Entity { world: World { dimension: ctx.world }, id: entity }))
        }

        BlockCall::NeighborChanged { ctx, neighbor } => unit(|| behavior.onNeighborChanged(&mut wrap(ctx), neighbor)),
        BlockCall::UpdateTick { ctx } => unit(|| behavior.onUpdateTick(&mut wrap(ctx))),
        BlockCall::RandomDisplayTick { ctx } => unit(|| behavior.onRandomDisplayTick(&mut wrap(ctx))),
        BlockCall::EntityCollided { ctx, entity } => unit(||
        {
            behavior.onEntityCollided(&mut wrap(ctx), Entity { world: World { dimension: ctx.world }, id: entity })
        }),

        BlockCall::EntityWalking { ctx, entity } => unit(||
        {
            behavior.onEntityWalking(&mut wrap(ctx), Entity { world: World { dimension: ctx.world }, id: entity })
        }),
        BlockCall::DestroyedByPlayer { ctx } => unit(|| behavior.onDestroyedByPlayer(&mut wrap(ctx))),
        BlockCall::DestroyedByExplosion { ctx } => unit(|| behavior.onDestroyedByExplosion(&mut wrap(ctx))),
        BlockCall::Harvest { ctx, player } => unit(|| behavior.onHarvest(&mut wrap(ctx), Player { id: player })),
        BlockCall::CanPlaceAt { ctx } => GuestReply::Bool(behavior.canPlaceAt(&mut wrap(ctx))),
        BlockCall::CanStay { ctx } => GuestReply::Bool(behavior.canStay(&mut wrap(ctx))),
        BlockCall::PoweringTo { ctx, direction } => GuestReply::Bool(behavior.poweringTo(&mut wrap(ctx), direction)),

        BlockCall::IndirectlyPoweringTo { ctx, direction } =>
        {
            GuestReply::Bool(behavior.indirectlyPoweringTo(&mut wrap(ctx), direction))
        }

        BlockCall::CanProvidePower { .. } => GuestReply::Bool(behavior.canProvidePower()),
        BlockCall::Texture { face, meta } => GuestReply::Int(behavior.texture(face, meta)),
        BlockCall::ColorMultiplier { ctx } => GuestReply::Int(behavior.colorMultiplier(&mut wrap(ctx)) as i32),

        BlockCall::FaceTexture { ctx, face } =>
        {
            GuestReply::Int(behavior.faceTexture(&mut wrap(ctx), face).unwrap_or(-1))
        }

        BlockCall::Strength { ctx, player } =>
        {
            GuestReply::Float(behavior.strength(&mut wrap(ctx), Player { id: player }).unwrap_or(-1.0))
        }

        BlockCall::Overlay { ctx, face } => GuestReply::Overlay(behavior.overlay(&mut wrap(ctx), face)),
        BlockCall::FaceRotation { ctx } => match behavior.faceRotations(&mut wrap(ctx))
        {
            Some(turns) => GuestReply::Bytes(turns.to_vec()),
            None => GuestReply::Unit,
        },

        BlockCall::Connections { ctx } => GuestReply::Int(i32::from(behavior.connections(&mut wrap(ctx)))),

        BlockCall::Brightness { ctx, base } =>
        {
            GuestReply::Float(behavior.brightness(&mut wrap(ctx), base).unwrap_or(-1.0))
        }
    }
}

fn wrap(raw: mod_protocol::abi::BlockCtx) -> BlockCtx { BlockCtx { raw } }

// a void callback answers Unit, which the host reads as handled
fn unit(body: impl FnOnce()) -> GuestReply
{
    body();
    GuestReply::Bool(true)
}

fn useReply(used: Use) -> GuestReply
{
    match used
    {
        Use::Pass => GuestReply::Use { handled: false, stack: None },
        Use::Done(stack) => GuestReply::Use { handled: true, stack: stack.and_then(|stack| stack.toRaw().ok()) },
    }
}

fn item(handle: u32, call: ItemCall) -> GuestReply
{
    let Some(behavior) = get(|tables| tables.m_items.get(handle as usize))
    else
    {
        return GuestReply::Error(format!("no item behavior {handle}"));
    };

    match call
    {
        ItemCall::RightClick { world, player, stack } => useReply(behavior.onRightClick(
            World { dimension: world },
            Player { id: player },
            ItemStack::fromRaw(stack),
        )),
        ItemCall::UseOnBlock { world, player, stack, pos, face } => useReply(behavior.onUseOnBlock(
            World { dimension: world },
            Player { id: player },
            ItemStack::fromRaw(stack),
            pos,
            face,
        )),
        ItemCall::HitEntity { stack, target, attacker } =>
        {
            GuestReply::Bool(behavior.onHitEntity(ItemStack::fromRaw(stack), located(target), attacker))
        }

        ItemCall::InteractEntity { stack, target, .. } =>
        {
            unit(|| behavior.onInteractEntity(ItemStack::fromRaw(stack), located(target)))
        }

        ItemCall::BlockDestroyed { stack, block, pos, player } =>
        {
            GuestReply::Bool(behavior.onBlockDestroyed(ItemStack::fromRaw(stack), block, pos, Player { id: player }))
        }

        ItemCall::Icon { damage } => GuestReply::Int(behavior.icon(damage)),

        ItemCall::StrVsBlock { stack, block } =>
        {
            GuestReply::Float(behavior.strVsBlock(ItemStack::fromRaw(stack), block))
        }

        ItemCall::CanHarvest { block } => GuestReply::Bool(behavior.canHarvest(block)),

        ItemCall::Crafted { stack, player } =>
        {
            unit(|| behavior.onCrafted(ItemStack::fromRaw(stack), Player { id: player }))
        }

        ItemCall::UseDuration { stack } => GuestReply::Int(behavior.useDuration(ItemStack::fromRaw(stack))),
        ItemCall::UseAction { stack } => GuestReply::Int(behavior.useAction(ItemStack::fromRaw(stack)) as i32),
        ItemCall::UsingTick { world, player, stack, left } => unit(||
        {
            behavior.onUsingTick(World { dimension: world }, Player { id: player }, ItemStack::fromRaw(stack), left)
        }),
        ItemCall::StoppedUsing { world, player, stack, left } => useReply(behavior.onStoppedUsing(
            World { dimension: world },
            Player { id: player },
            ItemStack::fromRaw(stack),
            left,
        )),

        ItemCall::UseIcon { stack, used } =>
        {
            GuestReply::Int(behavior.useIcon(ItemStack::fromRaw(stack), used).unwrap_or(-1))
        }

        ItemCall::Name { stack } => match behavior.name(ItemStack::fromRaw(stack))
        {
            Some(name) => GuestReply::Lines(vec![name]),
            None => GuestReply::Unit,
        },

        ItemCall::UseFinish { world, player, stack } =>
        {
            useReply(behavior.onUseFinish(World { dimension: world }, Player { id: player }, ItemStack::fromRaw(stack)))
        }
    }
}

fn entity(handle: u32, call: EntityCall) -> GuestReply
{
    let Some(behavior) = get(|tables| tables.m_entities.get(handle as usize))
    else
    {
        return GuestReply::Error(format!("no entity behavior {handle}"));
    };

    match call
    {
        EntityCall::Tick { entity } => unit(|| behavior.onTick(entityOf(&entity), &entity)),
        EntityCall::LivingUpdate { entity } => unit(|| behavior.onLivingUpdate(entityOf(&entity), &entity)),

        EntityCall::Interact { entity, player } =>
        {
            GuestReply::Bool(behavior.onInteract(entityOf(&entity), Player { id: player }))
        }

        EntityCall::Hurt { entity, source, amount } => match behavior.onHurt(entityOf(&entity), source, amount)
        {
            Some(amount) => GuestReply::Int(amount),
            None => GuestReply::Bool(false),
        },
        EntityCall::Death { entity, source } => unit(|| behavior.onDeath(entityOf(&entity), source)),
        EntityCall::CanSpawn { entity } => GuestReply::Bool(behavior.canSpawn(entityOf(&entity), &entity)),
        EntityCall::Pose { input } => GuestReply::Pose(behavior.pose(&input)),
        EntityCall::Ai { entity } => unit(|| behavior.onAi(entityOf(&entity), &entity)),
        EntityCall::Travel { entity, strafe, forward } =>
        {
            GuestReply::Bool(behavior.onTravel(entityOf(&entity), &entity, strafe, forward))
        }
    }
}

// item calls carry no dimension, the host finds the entity in whichever world holds it
fn located(id: i32) -> Entity
{
    World::OVERWORLD.entity(id).map_or(Entity { world: World::OVERWORLD, id }, |info| entityOf(&info))
}

fn entityOf(info: &mod_protocol::abi::EntityInfo) -> Entity
{
    Entity { world: World { dimension: info.world }, id: info.id }
}

fn effect(handle: u32, call: EffectCall) -> GuestReply
{
    let Some(behavior) = get(|tables| tables.m_effects.get(handle as usize))
    else
    {
        return GuestReply::Error(format!("no effect behavior {handle}"));
    };

    match call
    {
        EffectCall::Added { entity, level, .. } => unit(|| behavior.onAdded(entityOf(&entity), level)),
        EffectCall::Tick { entity, level, ticks, .. } => unit(|| behavior.onTick(entityOf(&entity), level, ticks)),
        EffectCall::Removed { entity, .. } => unit(|| behavior.onRemoved(entityOf(&entity))),
    }
}

fn tile(handle: u32, call: TileCall) -> GuestReply
{
    let Some(behavior) = get(|tables| tables.m_tiles.get(handle as usize))
    else
    {
        return GuestReply::Error(format!("no tile behavior {handle}"));
    };

    match call
    {
        TileCall::Tick { ctx } => unit(|| behavior.onTick(&mut BlockCtx { raw: ctx })),
        TileCall::Removed { ctx } => unit(|| behavior.onRemoved(&mut BlockCtx { raw: ctx })),

        TileCall::Pose { ctx, partial, ticks } =>
        {
            GuestReply::Pose(behavior.pose(&mut BlockCtx { raw: ctx }, partial, ticks))
        }

        TileCall::InsertItem { ctx, face, stack, simulate } =>
        {
            GuestReply::Int(behavior.insertItem(&mut BlockCtx { raw: ctx }, face, ItemStack::fromRaw(stack), simulate))
        }

        TileCall::ExtractItem { ctx, face, max, simulate } =>
        {
            let taken: Option<ItemStack> = behavior.extractItem(&mut BlockCtx { raw: ctx }, face, max, simulate);
            GuestReply::Stack(taken.and_then(|stack| stack.raw().ok()))
        }

        TileCall::EnergyDemand { ctx, face } =>
        {
            GuestReply::Demand(behavior.energyDemand(&mut BlockCtx { raw: ctx }, face))
        }

        TileCall::ReceiveEnergy { ctx, face, amount, simulate, .. } =>
        {
            GuestReply::Long(behavior.receiveEnergy(&mut BlockCtx { raw: ctx }, face, amount, simulate))
        }

        TileCall::FillFluid { ctx, face, stack, simulate } =>
        {
            GuestReply::Int(behavior.fillFluid(&mut BlockCtx { raw: ctx }, face, &stack, simulate))
        }

        TileCall::DrainFluid { ctx, face, max, simulate } =>
        {
            GuestReply::Fluid(behavior.drainFluid(&mut BlockCtx { raw: ctx }, face, max, simulate))
        }
    }
}

pub(crate) fn handle(call: GuestCall) -> GuestReply
{
    match call
    {
        GuestCall::Lifecycle(Lifecycle::Register(modid)) =>
        {
            crate::setModId(&modid);

            let mut ctx: RegisterCtx = RegisterCtx { m_modId: modid };
            match crate::rt::runMain(&mut ctx)
            {
                Ok(()) => GuestReply::Unit,
                Err(error) => GuestReply::Error(error.0),
            }
        }

        GuestCall::Lifecycle(Lifecycle::Remapped) =>
        {
            crate::bumpEpoch();
            GuestReply::Unit
        }

        GuestCall::Lifecycle(Lifecycle::InterMod) => GuestReply::Unit,

        GuestCall::Lifecycle(Lifecycle::ConfigChanged(key)) =>
        {
            let listeners: Vec<ConfigListener> = TABLES.with(|tables| tables.borrow().m_configListeners.clone());
            for listener in listeners
            {
                listener(&key);
            }

            GuestReply::Unit
        }

        GuestCall::Block { handle, call } => block(handle, call),
        GuestCall::Item { handle, call } => item(handle, call),
        GuestCall::Entity { handle, call } => entity(handle, call),
        GuestCall::Tile { handle, call } => tile(handle, call),
        GuestCall::Event { handler, event } => match get(|tables| tables.m_events.get(handler as usize))
        {
            Some(listener) =>
            {
                let mut wrapped: Event = Event { m_envelope: event };
                listener(&mut wrapped);
                GuestReply::Event(wrapped.m_envelope)
            }

            None => GuestReply::Error(format!("no event handler {handler}")),
        },

        GuestCall::Command { handler, sender, args } => match get(|tables| tables.m_commands.get(handler as usize))
        {
            Some(command) =>
            {
                let mut running: Command = Command { sender, args, m_reply: String::new() };
                match command(&mut running)
                {
                    Ok(()) => GuestReply::Bytes(running.m_reply.into_bytes()),
                    Err(error) => GuestReply::Bytes(format!("\u{a7}c{}", error.0).into_bytes()),
                }
            }

            None => GuestReply::Error(format!("no command handler {handler}")),
        },

        GuestCall::Key { handler, pressed } => match get(|tables| tables.m_keys.get(handler as usize))
        {
            Some(key) => unit(|| key(pressed)),
            None => GuestReply::Error(format!("no key handler {handler}")),
        },

        GuestCall::Channel { handler, player, data } => match get(|tables| tables.m_channels.get(handler as usize))
        {
            Some(channel) => unit(|| channel(Player { id: player }, &data)),
            None => GuestReply::Error(format!("no channel handler {handler}")),
        },

        GuestCall::Screen { handler, op } => match get(|tables| tables.m_screens.get(handler as usize))
        {
            Some(screen) => match crate::screen::run(&mut *screen.borrow_mut(), op)
            {
                Some(list) => GuestReply::Draw(list),
                None => GuestReply::Bool(true),
            },
            None => GuestReply::Error(format!("no screen {handler}")),
        },

        GuestCall::PlayerLayers { handler, player, synced } =>
        {
            match get(|tables| tables.m_layers.get(handler as usize))
            {
                Some(layer) => GuestReply::Layers(layer(&player, &synced)),
                None => GuestReply::Error(format!("no player layer {handler}")),
            }
        }

        GuestCall::Overlay { handler, op } => match get(|tables| tables.m_overlays.get(handler as usize))
        {
            Some(overlay) => crate::screen::runOverlay(&mut *overlay.borrow_mut(), op),
            None => GuestReply::Error(format!("no overlay {handler}")),
        },

        GuestCall::Tooltip { handler, stack } => match get(|tables| tables.m_tooltips.get(handler as usize))
        {
            Some(tooltip) =>
            {
                let mut lines: Vec<String> = Vec::new();
                tooltip(&ItemStack::fromRaw(stack), &mut lines);
                GuestReply::Lines(lines)
            }

            None => GuestReply::Error(format!("no tooltip {handler}")),
        },

        GuestCall::WorldRender { handler, partial, camera } =>
        {
            match get(|tables| tables.m_worldRenders.get(handler as usize))
            {
                Some(render) =>
                {
                    GuestReply::Pose(mod_protocol::abi::Pose { pre: Vec::new(), model: render(partial, camera) })
                }

                None => GuestReply::Error(format!("no world render {handler}")),
            }
        }

        GuestCall::Hud { handler, partial, width, height } => match get(|tables| tables.m_huds.get(handler as usize))
        {
            Some(hud) =>
            {
                let mut draw: Draw = Draw::default();
                hud(&mut draw, partial, width, height);
                GuestReply::Draw(draw.m_list)
            }

            None => GuestReply::Error(format!("no hud {handler}")),
        },

        GuestCall::Service { handler, method, args } => match get(|tables| tables.m_services.get(handler as usize))
        {
            Some(service) => match service.call(&method, &args)
            {
                Ok(bytes) => GuestReply::Bytes(bytes),
                Err(error) => GuestReply::Error(error.0),
            },
            None => GuestReply::Error(format!("no service {handler}")),
        },

        GuestCall::Lookup { handler, world, pos, side, method, args } =>
        {
            match get(|tables| tables.m_lookups.get(handler as usize))
            {
                Some(lookup) => match lookup.call(world, pos, side, &method, &args)
                {
                    Ok(bytes) => GuestReply::Bytes(bytes),
                    Err(error) => GuestReply::Error(error.0),
                },
                None => GuestReply::Error(format!("no lookup {handler}")),
            }
        }

        GuestCall::InterMod { from, kind, data } =>
        {
            if let Some(handler) = TABLES.with(|tables| tables.borrow().m_interMod.clone())
            {
                handler(&from, &kind, &data);
            }

            GuestReply::Unit
        }

        GuestCall::Generate { handler, chunk } => match get(|tables| tables.m_generators.get(handler as usize))
        {
            Some(generator) => unit(|| generator(chunk)),
            None => GuestReply::Error(format!("no generator {handler}")),
        },

        GuestCall::Terrain { handler, chunk } => match get(|tables| tables.m_terrains.get(handler as usize))
        {
            Some(terrain) =>
            {
                let mut blocks: ChunkBlocks = ChunkBlocks::default();
                terrain(&chunk, &mut blocks);
                GuestReply::Chunk { blocks: blocks.m_blocks, meta: blocks.m_meta }
            }

            None => GuestReply::Error(format!("no terrain generator {handler}")),
        },

        GuestCall::BiomeSource { handler, chunk } => match get(|tables| tables.m_biomeSources.get(handler as usize))
        {
            Some(source) =>
            {
                let biomes: crate::ChunkBiomes = source(&chunk);
                GuestReply::Biomes { biomes: biomes.biomes }
            }

            None => GuestReply::Error(format!("no biome source {handler}")),
        },

        GuestCall::Spawns { handler, world, kind, pos } =>
        {
            match get(|tables| tables.m_spawnLists.get(handler as usize))
            {
                Some(spawns) =>
                {
                    let query: crate::SpawnQuery =
                        crate::SpawnQuery { world: crate::world::World { dimension: world }, kind, pos };
                    GuestReply::Spawns { entries: spawns(&query) }
                }

                None => GuestReply::Error(format!("no spawn list {handler}")),
            }
        }

        GuestCall::StructureFinder { handler, seed, chunk_x, chunk_z } =>
        {
            match get(|tables| tables.m_finders.get(handler as usize))
            {
                Some(finder) => GuestReply::Bool(finder(seed, chunk_x, chunk_z)),
                None => GuestReply::Error(format!("no structure finder {handler}")),
            }
        }

        GuestCall::Effect { handle, call } => effect(handle, call),
    }
}
