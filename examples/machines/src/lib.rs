use b173_api::prelude::*;
use energy_api::{Amount, Changed, methods};

const OUTPUT: i64 = 50;
const PERIOD: u64 = 40;
const SIDES: [(i32, i32, i32); 6] = [(1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)];

#[derive(Serialize, Deserialize, Default)]
struct Generator
{
    ticks: u64,
    produced: i64,
}

// every few seconds the generator pushes energy into each battery next to it, found through the lookup
struct GeneratorTile;

#[tile_behavior]
impl TileBehavior for GeneratorTile
{
    fn onTick(&self, ctx: &mut BlockCtx)
    {
        if !ctx.isAuthority()
        {
            return;
        }
        let mut state: Generator = ctx.tileData();
        state.ticks += 1;
        if state.ticks.is_multiple_of(PERIOD)
        {
            for (dx, dy, dz) in SIDES
            {
                let target: BlockPos = ctx.pos().offset(dx, dy, dz);
                let inserted: Result<Option<Amount>> = services::lookup(
                    energy_api::LOOKUP,
                    ctx.world().dimension,
                    target,
                    0,
                    methods::INSERT,
                    &Amount(OUTPUT),
                );
                if let Ok(Some(amount)) = inserted
                {
                    state.produced += amount.0;
                }
            }
        }
        let _ = ctx.setTileData(&state);
    }
}

struct GeneratorBlock;

#[block_behavior]
impl BlockBehavior for GeneratorBlock
{
    fn onActivated(&self, ctx: &mut BlockCtx, player: Player) -> bool
    {
        if ctx.isAuthority()
        {
            let state: Generator = ctx.tileData();
            let total: Result<Amount> =
                services::callService(energy_api::SERVICE, energy_api::VERSION, methods::TOTAL, &());
            player.chat(&format!(
                "produced {}, network holds {}",
                state.produced,
                total.map(|amount| amount.0).unwrap_or(0)
            ));
        }
        true
    }
}

#[mod_main]
fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    if !services::has(energy_api::SERVICE, energy_api::VERSION)
    {
        warn("the energy service is missing, generators will idle");
    }
    ctx.tileEntities().register(
        "generator",
        TileEntitySpec { inventory: 0, ticks: true, sync: false, behavior: None },
        Some(std::rc::Rc::new(GeneratorTile)),
    )?;
    ctx.blocks().register(
        "generator",
        BlockSpecBuilder::template(vanilla::block::COBBLESTONE)
            .hardness(3.5)
            .texture("machines:block/generator")
            .light(0.4)
            .tileEntity("generator")
            .behavior(GeneratorBlock),
    )?;
    ctx.events().onCustom(energy_api::CHANGED_EVENT, Priority::Monitor, |event| {
        if let Some((_, data)) = event.custom()
            && let Ok(changed) = fromBytes::<Changed>(data)
            && changed.stored.0 >= 10_000
        {
            broadcast(&format!("a battery at {} {} {} is full", changed.x, changed.y, changed.z));
        }
    })?;
    ctx.recipes()
        .shaped("generator", ItemStack::of("machines:generator", 1))
        .pattern(["CCC", "CFC", "CRC"])
        .key('C', vanilla::block::COBBLESTONE)
        .key('F', "minecraft:furnace")
        .key('R', vanilla::item::REDSTONE)
        .build()?;
    services::sendInterMod("energy", "hello", &"machines are online".to_owned());
    Ok(())
}
