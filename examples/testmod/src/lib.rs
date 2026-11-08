use b173_api::prelude::*;
use b173_api::screen::Draw;
use b173_api::services;
use b173_api::world::{BlockCtx, Entity, Player, World};
use std::cell::Cell;

const CHARGE: &str = "charge";
const PING: &str = "ping";
const MACHINE_SCREEN: &str = "machine";
const PULSE: &str = "testmod:pulse";
const HUD_OPTION: &str = "hud.visible";
const HUD_COLOR_OPTION: &str = "hud.color";
const HUD_COLORS: [(&str, u32); 3] = [("yellow", 0xFFFF55), ("white", 0xFFFFFF), ("green", 0x55FF55)];

fn hudColor() -> u32
{
    let chosen: String = Config.get(HUD_COLOR_OPTION);
    HUD_COLORS.iter().find(|(name, _)| *name == chosen).map_or(HUD_COLORS[0].1, |(_, color)| *color)
}
const REALM: &str = "testmod:test_realm";
const MACHINE_SLOTS: i32 = 2;
const HUD_KEY: i32 = 37;

thread_local! {
    static HUD_VISIBLE: Cell<bool> = const { Cell::new(true) };
    static CHUNKS_POPULATED: Cell<u32> = const { Cell::new(0) };
}

#[derive(Serialize, Deserialize)]
struct ChargeSync
{
    pos: BlockPos,
    charge: u8,
}

#[derive(Serialize, Deserialize)]
struct Ping
{
    note: String,
}

#[derive(Serialize, Deserialize, Default)]
struct MachineState
{
    ticks: u64,
    energy: i64,
}

#[derive(Serialize, Deserialize, Default)]
struct GliderMemory
{
    hops: u32,
}

#[derive(Serialize, Deserialize, Default)]
struct Visits
{
    count: u32,
}

// right click raises the meta, which is the light level the block sends to nearby clients
struct ChargedOre;

#[block_behavior]
impl BlockBehavior for ChargedOre
{
    fn onActivated(&self, ctx: &mut BlockCtx, player: Player) -> bool
    {
        if !ctx.isAuthority()
        {
            return true;
        }
        let charge: u8 = ctx.meta().wrapping_add(1) & 15;
        ctx.setMeta(charge);
        let _ = ctx.sendTracking(CHARGE, &ChargeSync { pos: ctx.pos(), charge });
        player.chat(&format!("charge {charge}"));
        true
    }

    fn onNeighborChanged(&self, ctx: &mut BlockCtx, _neighbor: i32)
    {
        if ctx.world().isPowered(ctx.pos())
        {
            ctx.world().particle(
                "reddust",
                [ctx.pos().x as f64 + 0.5, ctx.pos().y as f64 + 1.0, ctx.pos().z as f64 + 0.5],
                [0.0; 3],
            );
        }
    }

    fn texture(&self, face: i32, meta: i32) -> i32 { if face == 1 && meta > 7 { 51 } else { 50 } }
}

struct Machine;

#[block_behavior]
impl BlockBehavior for Machine
{
    fn onActivated(&self, ctx: &mut BlockCtx, player: Player) -> bool
    {
        if ctx.isAuthority()
        {
            let _ = player.openScreen(MACHINE_SCREEN, Some(ctx.pos()), Vec::new());
        }
        true
    }

    fn onPlacedBy(&self, ctx: &mut BlockCtx, entity: Entity)
    {
        let _ = ctx.setTileData(&MachineState { ticks: 0, energy: entity.id as i64 % 7 });
    }
}

struct MachineTile;

#[tile_behavior]
impl TileBehavior for MachineTile
{
    fn onTick(&self, ctx: &mut BlockCtx)
    {
        if !ctx.isAuthority()
        {
            return;
        }
        let mut state: MachineState = ctx.tileData();
        state.ticks += 1;
        if state.ticks.is_multiple_of(20)
        {
            state.energy += 1;
            let world: World = ctx.world();
            if let Some(fuel) = world.tileSlot(ctx.pos(), 0)
                && fuel.count > 0
            {
                let _ = world.setTileSlot(ctx.pos(), 0, Some(ItemStack { count: fuel.count - 1, ..fuel }));
                let _ = world.setTileSlot(ctx.pos(), 1, Some(ItemStack::of("testmod:spark", 1)));
            }
            let _ = ctx.setTileData(&state);
        }
    }

    fn onRemoved(&self, ctx: &mut BlockCtx)
    {
        let state: MachineState = ctx.tileData();
        log(&format!("machine removed after {} ticks", state.ticks));
    }
}

struct Spark;

#[item_behavior]
impl ItemBehavior for Spark
{
    fn onRightClick(&self, world: World, player: Player, stack: ItemStack) -> Use
    {
        if let Some(info) = player.info()
        {
            world.particle("flame", info.pos, [0.0, 0.1, 0.0]);
            world.playSound(info.pos, "random.fizz", 1.0, 1.0);
            // per stack data: a component, so used sparks no longer stack with fresh ones
            let held: StackRef = StackRef::Held { player: player.id };
            let uses: u32 = stackComponent::<u32>(held, "testmod:uses") + 1;
            let _ = setStackComponent(held, "testmod:uses", &uses);
            // per chunk data: an attachment only this mod reads and writes
            let (chunkX, chunkZ): (i32, i32) = ((info.pos[0] as i32) >> 4, (info.pos[2] as i32) >> 4);
            let sparks: u32 = world.chunkAttachment::<u32>(chunkX, chunkZ) + 1;
            let _ = world.setChunkAttachment(chunkX, chunkZ, &sparks);
        }
        let remaining: i32 = stack.count - 1;
        Use::Done((remaining > 0).then_some(ItemStack { count: remaining, ..stack }))
    }

    fn onHitEntity(&self, _stack: ItemStack, target: Entity, attacker: i32) -> bool
    {
        target.damage(2, Some(attacker));
        true
    }
}

// 4 facings times 6 power levels is 24 named states, more than one block id holds, so the loader spans two ids
struct Dial;

const DIAL_POWER: [&str; 6] = ["0", "1", "2", "3", "4", "5"];

#[block_behavior]
impl BlockBehavior for Dial
{
    fn onActivated(&self, ctx: &mut BlockCtx, player: Player) -> bool
    {
        if !ctx.isAuthority()
        {
            return true;
        }
        let Some(mut state) = ctx.state()
        else
        {
            return false;
        };
        let power: usize = state.properties.get("power").and_then(|power| power.parse().ok()).unwrap_or(0);
        let next: &str = DIAL_POWER[(power + 1) % DIAL_POWER.len()];
        state.properties.insert("power".to_owned(), next.to_owned());
        let _ = ctx.setState(state);
        player.chat(&format!("dial power {next}"));
        true
    }
}

struct Glider;

#[entity_behavior]
impl EntityBehavior for Glider
{
    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        if info.motion[1] < -0.1
        {
            let _ = entity.setMotion([info.motion[0] * 1.05, -0.05, info.motion[2] * 1.05]);
        }
    }

    fn onInteract(&self, entity: Entity, player: Player) -> bool
    {
        let mut memory: GliderMemory = entity.data();
        memory.hops += 1;
        let _ = entity.setData(&memory);
        let _ = entity.setMotion([0.0, 0.8, 0.0]);
        player.chat(&format!("the glider hopped {} times", memory.hops));
        true
    }

    fn onHurt(&self, _entity: Entity, source: Option<i32>, amount: i32) -> Option<i32>
    {
        // fall damage has no source and gliders never take it
        source.map(|_| amount)
    }

    fn canSpawn(&self, _entity: Entity, info: &EntityInfo) -> bool { info.pos[1] > 60.0 }
}

struct MachineScreen
{
    m_pressed: u32,
}

impl Screen for MachineScreen
{
    fn init(&mut self, _width: i32, _height: i32, _pos: Option<BlockPos>, _data: &[u8]) { self.m_pressed = 0; }

    fn draw(&mut self, draw: &mut Draw, _mouse: [i32; 2], _partial: f32)
    {
        draw.text(60, 20, &format!("pressed {}", self.m_pressed), 0x404040).button(1, 110, 50, 50, 20, "Pulse").item(
            40,
            50,
            &ItemStack::of("testmod:spark", 1),
        );
    }

    fn button(&mut self, id: i32)
    {
        if id == 1
        {
            self.m_pressed += 1;
            let _ = send(PING, Target::Server, &Ping { note: format!("button {}", self.m_pressed) });
        }
    }
}

// the energy mod provides a storage service and a block lookup, this mod uses both only when energy is installed
fn energyReport(world: World, pos: BlockPos) -> String
{
    if !services::has(energy_api::SERVICE, energy_api::VERSION)
    {
        return "energy mod not installed".to_owned();
    }
    let total: Result<energy_api::Amount> =
        services::callService(energy_api::SERVICE, energy_api::VERSION, energy_api::methods::TOTAL, &());
    let stored: Result<Option<energy_api::Amount>> =
        services::lookup(energy_api::LOOKUP, world.dimension, pos, 0, energy_api::methods::STORED, &());
    format!(
        "network total {:?}, below {:?}",
        total.ok().map(|amount| amount.0),
        stored.ok().flatten().map(|amount| amount.0)
    )
}

fn registerContent(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.blocks().register(
        "charged_ore",
        BlockSpecBuilder::template(vanilla::block::DIAMOND_ORE)
            .hardness(3.0)
            .texture("testmod:block/charged_ore")
            .tool(ToolClass::Pickaxe, 1)
            .tag("c:ores/charged")
            .behavior(ChargedOre),
    )?;
    ctx.tileEntities().register(
        "machine",
        TileEntitySpec { inventory: MACHINE_SLOTS, ticks: true, sync: true, behavior: None },
        Some(std::rc::Rc::new(MachineTile)),
    )?;
    ctx.blocks().register(
        "machine",
        BlockSpecBuilder::template(vanilla::block::IRON_BLOCK)
            .hardness(4.0)
            .texture("testmod:block/machine")
            .tileEntity("machine")
            .sound("metal")
            .behavior(Machine),
    )?;
    ctx.blocks().register(
        "pedestal",
        BlockSpecBuilder::template(vanilla::block::COBBLESTONE).hardness(1.0).texture("minecraft:terrain/16").model(
            BlockModelSpec {
                cuboids: vec![
                    Cuboid { from: [0.0, 0.0, 0.0], to: [16.0, 4.0, 16.0], ..Cuboid::default() },
                    Cuboid {
                        from: [5.0, 4.0, 5.0],
                        to: [11.0, 14.0, 11.0],
                        texture: Some("testmod:block/machine".to_owned()),
                        ..Cuboid::default()
                    },
                ],
                ..BlockModelSpec::default()
            },
        ),
    )?;
    ctx.blocks().register(
        "dial",
        BlockSpecBuilder::template(vanilla::block::IRON_BLOCK)
            .hardness(2.0)
            .texture("testmod:block/machine")
            .behavior(Dial),
    )?;
    ctx.blocks().states(
        "dial",
        BlockStatesSpec {
            properties: vec![
                PropertySpec {
                    name: "facing".to_owned(),
                    values: ["north", "east", "south", "west"].map(str::to_owned).to_vec(),
                },
                PropertySpec { name: "power".to_owned(), values: DIAL_POWER.map(str::to_owned).to_vec() },
            ],
            variants: vec![StateVariantSpec {
                when: [("power".to_owned(), "5".to_owned())].into(),
                texture: Some("testmod:block/charged_ore".to_owned()),
                textures: None,
            }],
            ..BlockStatesSpec::default()
        },
    )?;
    // a vanilla block changed through the modify phase, other mods changing the same field are reported
    ctx.blocks().modify(BlockModifySpec {
        target: Identifier::minecraft("glowstone"),
        hardness: Some(0.6),
        resistance: None,
        light: None,
        opacity: None,
        slipperiness: None,
        flammability: None,
        tool: None,
    })?;
    ctx.items().register(
        "spark",
        ItemSpecBuilder::new()
            .texture("testmod:item/spark")
            .maxStack(16)
            .fuel(800)
            .tag("c:dusts/spark")
            .behavior(Spark),
    )?;
    ctx.items().register(
        "spark_pick",
        ItemSpecBuilder::new().texture("minecraft:items/98").tool(ToolClass::Pickaxe, 2, 9.0, 3, 400),
    )?;
    ctx.items().register("spark_snack", ItemSpecBuilder::new().texture("minecraft:items/10").food(5))?;
    ctx.entities().register(
        "glider",
        EntitySpec {
            template: Identifier::parse("minecraft:chicken").ok(),
            texture: Some("testmod:entity/glider".to_owned()),
            health: Some(6),
            spawn: Some(SpawnSpec {
                kind: CreatureKind::Creature,
                weight: 4,
                biomes: vec![Identifier::minecraft("plains")],
            }),
            drops: Some(vec![DropSpec {
                item: Identifier::parse("testmod:spark").map_err(|error| Error(error.to_string()))?,
                count: vec![0, 2],
                meta: 0,
                chance: 1.0,
            }]),
            ..EntitySpec::default()
        },
        Some(std::rc::Rc::new(Glider)),
    )?;
    let spark: ItemRef = ctx.items().get("spark")?;
    ctx.recipes()
        .shaped("machine", ItemStack::of("testmod:machine", 1))
        .pattern(["III", "ISI", "III"])
        .key('I', vanilla::item::IRON_INGOT)
        .key('S', &spark)
        .build()?;
    ctx.recipes().shapeless(
        "spark",
        ItemStack::of(&spark, 4),
        &[ItemStack::of(vanilla::item::REDSTONE, 1), ItemStack::of(vanilla::item::COAL, 1)],
    )?;
    ctx.recipes().smelting("spark_from_ore", "testmod:charged_ore", ItemStack::of(&spark, 2))?;
    ctx.recipes().fuel("spark_snack_fuel", "testmod:spark_snack", 200)?;
    ctx.tags().add(RegistryKind::Item, "c:dusts/spark", &["testmod:spark"])?;
    ctx.worldgen().ore("charged_extra", "charged_ore", 3, 1, [10, 20])?;
    Ok(())
}

fn registerWorld(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.dimensions().biome(
        "realm_plains",
        BiomeSpec {
            name: "Realm Plains".to_owned(),
            color: 0x55AA55,
            temperature: 0.7,
            rainfall: 0.2,
            top_block: Identifier::parse("minecraft:sand").ok(),
            filler_block: Identifier::parse("minecraft:sandstone").ok(),
            weight: 1,
        },
    )?;
    ctx.dimensions().register(
        "test_realm",
        DimensionSpec {
            beds: false,
            music: None,
            name: "Test Realm".to_owned(),
            sky_color: 0x88CCFF,
            fog_color: 0xAADDFF,
            has_sky: true,
            fill_block: Identifier::parse("minecraft:sandstone").ok(),
            ground_height: 40,
            portal_block: None,
            portal_frame: None,
            biome: Identifier::parse("testmod:realm_plains").ok(),
            generator: None,
        },
    )?;
    ctx.dimensions().generator(REALM, |chunk| {
        let world: World = World { dimension: chunk.world };
        if (chunk.x + chunk.z) % 3 == 0
        {
            let base: BlockPos = BlockPos::new(chunk.x * 16 + 8, 40, chunk.z * 16 + 8);
            if let Ok(pedestal) = b173_api::Blocks.id("pedestal")
            {
                for height in 0..4
                {
                    let _ = world.setBlockQuiet(base.offset(0, height, 0), pedestal, 0);
                }
            }
        }
    })?;
    Ok(())
}

fn registerEvents(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.events().define(PULSE)?;
    ctx.events().on::<Setup>(Priority::Normal, |_| log("setup ran"))?;
    ctx.events().on::<LoadComplete>(Priority::Normal, |_| {
        let ores: Vec<Identifier> = tagValues(RegistryKind::Block, "c:ores/charged");
        let names: Vec<String> = ores.iter().map(ToString::to_string).collect();
        log(&format!("load complete, charged ores [{}]", names.join(", ")));
        services::sendInterMod("energy", "hello", &"testmod says hi".to_owned());
    })?;
    ctx.events().on::<LivingDeath>(Priority::Normal, |event| {
        if event.entityKind().is_some_and(|kind| kind.path() == "creeper")
            && event.world().is_some_and(|world| world.chance(0.25))
        {
            event.addDrop(ItemStack::of("testmod:spark", 1));
        }
    })?;
    ctx.events().on::<BlockHarvestDrops>(Priority::Low, |event| {
        if let EventData::Drops { block, .. } = event.data()
            && b173_api::Blocks.id("charged_ore").is_ok_and(|id| id == *block)
        {
            event.addDrop(ItemStack::of("testmod:data_gem", 1));
        }
    })?;
    ctx.events().on::<BlockBreak>(Priority::High, |event| {
        let protected: bool = event.pos().is_some_and(|pos| pos.y == 0);
        if protected
        {
            event.cancel();
            if let Some(player) = event.player()
            {
                player.chat("the bottom layer is protected by testmod");
            }
        }
    })?;
    ctx.events().on::<LivingAttack>(Priority::Normal, |event| {
        if event.entityKind().is_some_and(|kind| kind.path() == "glider")
        {
            event.setAmount(1);
        }
    })?;
    ctx.events().on::<PlayerBreakSpeed>(Priority::Normal, |event| {
        if let EventData::BreakSpeed { speed, .. } = event.data()
        {
            let boosted: f32 = speed * 1.1;
            event.setSpeed(boosted);
        }
    })?;
    ctx.events().on::<PlayerChat>(Priority::Normal, |event| {
        if let EventData::Chat { message, .. } = event.data()
            && message.contains(":spark:")
        {
            let replaced: String = message.replace(":spark:", "*");
            event.setMessage(&replaced);
        }
    })?;
    ctx.events()
        .on::<ChunkPopulate>(Priority::Monitor, |_| CHUNKS_POPULATED.with(|count| count.set(count.get() + 1)))?;
    ctx.events().on::<PlayerLogin>(Priority::Normal, |event| {
        if let Some(player) = event.player()
        {
            let mut visits: Visits = player.data("visits");
            visits.count += 1;
            let _ = player.setData("visits", &visits);
            player.chat(&format!("welcome back, visit {}", visits.count));
        }
    })?;
    ctx.events().on::<TickPost>(Priority::Normal, |_| {
        let mut ticks: u64 = worldData("ticks");
        ticks += 1;
        setWorldData("ticks", &ticks);
    })?;
    ctx.events().on::<ItemCrafted>(Priority::Monitor, |event| {
        if let Some(stack) = event.stack()
        {
            log(&format!("crafted {:?} x{}", stack.item.name(), stack.count));
        }
    })?;
    ctx.events().onCustom(PULSE, Priority::Normal, |event| {
        if let Some((_, data)) = event.custom()
        {
            log(&format!("pulse with {} bytes", data.len()));
        }
    })?;
    ctx.events().on::<ScreenOpen>(Priority::Monitor, |_| {})?;
    ctx.events().on::<KeyInput>(Priority::Monitor, |_| {})?;
    Ok(())
}

fn registerInterface(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.channels().register::<ChargeSync>(CHARGE, Direction::ToClient, |message, _| {
        World::OVERWORLD.markBlockForRender(message.pos);
        log(&format!("charge {} at {:?}", message.charge, message.pos));
    })?;
    ctx.channels().register::<Ping>(PING, Direction::ToServer, |message, player| {
        // serverbound input is validated before use
        if message.note.len() <= 32
        {
            player.chat(&format!("server got {}", message.note));
        }
    })?;
    ctx.screens().register(
        MACHINE_SCREEN,
        ScreenBuilder::new("Test Machine", 176, 166)
            .background("testmod:gui/machine")
            .slot(0, 56, 35)
            .outputSlot(1, 116, 35)
            .playerInventory(8, 84),
        MachineScreen { m_pressed: 0 },
    )?;
    ctx.screens().hud(|draw, _partial, width, _height| {
        if HUD_VISIBLE.with(Cell::get)
        {
            draw.rect(width - 92, 2, 90, 12, 0x80000000).text(
                width - 90,
                4,
                &format!("chunks {}", CHUNKS_POPULATED.with(Cell::get)),
                hudColor(),
            );
        }
    })?;
    ctx.keybinds().register("toggle_hud", HUD_KEY, |pressed| {
        if pressed
        {
            let shown: bool = !HUD_VISIBLE.with(Cell::get);
            HUD_VISIBLE.with(|visible| visible.set(shown));
            if let Err(error) = Config.set(HUD_OPTION, shown)
            {
                log(&error.0);
            }
        }
    })?;
    ctx.commands().registerWithUsage("testmod", Permission::Op, "<give|realm|home|energy|pulse|data>", |command| {
        let player: Player = command.player().ok_or("players only")?;
        match command.arg(0)
        {
            Some("give") =>
            {
                player.give(ItemStack::of("testmod:spark", 16))?;
                player.give(ItemStack::of("testmod:machine", 1))?;
                command.reply("gave sparks and a machine");
            }
            Some("realm") =>
            {
                let realm: i32 = resolve(
                    RegistryKind::Dimension,
                    &Identifier::parse(REALM).map_err(|error| Error(error.to_string()))?,
                )?;
                player.teleport(realm, [0.5, 45.0, 0.5])?;
            }
            Some("home") => player.teleport(0, [0.5, 80.0, 0.5])?,
            Some("energy") => command.reply(&energyReport(command.world(), BlockPos::new(0, 63, 0))),
            Some("pulse") =>
            {
                let canceled: bool = services::postEvent(PULSE, &42u32)?;
                let (power, radius): (i64, f64) = (Config.getInt("pulse.power"), Config.getFloat("pulse.radius"));
                command.reply(&format!("pulse power {power} radius {radius} canceled {canceled}"));
            }
            Some("data") =>
            {
                let ticks: u64 = worldData("ticks");
                let visits: Visits = player.data("visits");
                command.reply(&format!("world ticks {ticks}, your visits {}", visits.count));
            }
            _ => command.reply("usage: /testmod <give|realm|home|energy|pulse|data>"),
        }
        Ok(())
    })?;
    ctx.services().onInterMod(|from, kind, _| log(&format!("message {kind} from {from}")));
    Ok(())
}

#[mod_main]
fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.config().options([
        Setting::bool(HUD_OPTION, true).label("Chunk counter").comment("show the chunk counter hud").client(),
        Setting::choice(HUD_COLOR_OPTION, "yellow", &["yellow", "white", "green"])
            .comment("chunk counter text color")
            .client(),
        Setting::int("pulse.power", 3, 0..=15).comment("strength of the pulse event").server(),
        Setting::float("pulse.radius", 4.0, 0.0..=16.0).comment("reach of the pulse event in blocks").server(),
        Setting::text("greeting", "hello").comment("chat greeting"),
    ])?;
    HUD_VISIBLE.with(|visible| visible.set(ctx.config().getBool(HUD_OPTION)));
    ctx.config().onChange(|key| {
        if key == HUD_OPTION
        {
            HUD_VISIBLE.with(|visible| visible.set(Config.getBool(HUD_OPTION)));
        }
    });
    registerContent(ctx)?;
    registerWorld(ctx)?;
    registerEvents(ctx)?;
    registerInterface(ctx)?;
    log(&format!("{} registered, client {}", ctx.modId(), isClient()));
    Ok(())
}
