use b173_api::prelude::*;
use energy_api::{Amount, Changed, methods};

const CAPACITY: i64 = 10000;
const TOTAL_KEY: &str = "network_total";

#[derive(Serialize, Deserialize, Default)]
struct Battery
{
    stored: i64,
}

fn adjust(world: World, pos: BlockPos, delta: i64) -> Result<Amount>
{
    let mut battery: Battery = world.tileData(pos);
    let before: i64 = battery.stored;

    battery.stored = (battery.stored + delta).clamp(0, CAPACITY);
    world.setTileData(pos, &battery)?;

    let total: i64 = worldData::<i64>(TOTAL_KEY) + battery.stored - before;
    setWorldData(TOTAL_KEY, &total);

    let changed: Changed =
        Changed { world: world.dimension, x: pos.x, y: pos.y, z: pos.z, stored: Amount(battery.stored) };
    services::postEvent(energy_api::CHANGED_EVENT, &changed)?;
    Ok(Amount(battery.stored - before))
}

// energy:network answers questions about the whole network
struct Network;

impl Service for Network
{
    fn call(&self, method: &str, _args: &[u8]) -> Result<Vec<u8>>
    {
        match method
        {
            methods::TOTAL => Ok(toBytes(&Amount(worldData::<i64>(TOTAL_KEY)))),
            other => Err(Error(format!("energy:network has no method {other}"))),
        }
    }
}

// energy:storage answers at each battery block
struct BatteryLookup;

impl Lookup for BatteryLookup
{
    fn call(&self, world: i32, pos: BlockPos, _side: i32, method: &str, args: &[u8]) -> Result<Vec<u8>>
    {
        let world: World = World { dimension: world };

        match method
        {
            methods::STORED => Ok(toBytes(&Amount(world.tileData::<Battery>(pos).stored))),
            methods::INSERT => Ok(toBytes(&adjust(world, pos, fromBytes::<Amount>(args)?.0)?)),
            methods::EXTRACT => Ok(toBytes(&adjust(world, pos, -fromBytes::<Amount>(args)?.0)?)),
            other => Err(Error(format!("energy:storage has no method {other}"))),
        }
    }
}

struct BatteryBlock;

#[block_behavior]
impl BlockBehavior for BatteryBlock
{
    fn onActivated(&self, ctx: &mut BlockCtx, player: Player) -> bool
    {
        if ctx.isAuthority()
        {
            let battery: Battery = ctx.tileData();
            player.chat(&format!("battery holds {} of {CAPACITY}", battery.stored));
        }

        true
    }
}

#[mod_main]
fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.blocks().register(
        "battery",
        BlockSpecBuilder::template(vanilla::block::IRON_BLOCK)
            .hardness(3.0)
            .texture("energy:block/battery")
            .tile(Tile::plain().synced())
            .behavior(BatteryBlock),
    )?;
    ctx.events().define(energy_api::CHANGED_EVENT)?;
    ctx.services().provide(energy_api::SERVICE, energy_api::VERSION, Network)?;
    ctx.services().lookup(energy_api::LOOKUP, "battery", BatteryLookup)?;

    ctx.services().onInterMod(|from, kind, data|
    {
        let text: String = fromBytes(data).unwrap_or_default();
        log(&format!("{from} sent {kind}: {text}"));
    });

    ctx.recipes()
        .shaped("battery", ItemStack::of("energy:battery", 1))
        .pattern(["IRI", "IRI", "III"])
        .key('I', vanilla::item::IRON_INGOT)
        .key('R', vanilla::item::REDSTONE)
        .build()?;
    Ok(())
}
