use crate::Identifier;
use crate::abi::{BlockPos, GameRuleValue, StackData, StateData};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum WorldRequest
{
    IsAuthority
    {
        world: i32
    },

    GetBlock
    {
        world: i32, pos: BlockPos
    },

    GetMeta
    {
        world: i32, pos: BlockPos
    },

    SetBlock
    {
        world: i32, pos: BlockPos, block: i32, meta: i32, notify: bool
    },

    SetMeta
    {
        world: i32, pos: BlockPos, meta: i32
    },

    /// a lightning bolt as weather strikes, seen by every player in range
    Lightning
    {
        world: i32, pos: [f64; 3]
    },

    /// the vanilla block break effect, cracking particles and the dig sound, seen by every nearby player
    BlockEffect
    {
        world: i32, pos: BlockPos, block: i32, meta: i32
    },

    NotifyNeighbors
    {
        world: i32, pos: BlockPos, block: i32
    },

    ScheduleTick
    {
        world: i32, pos: BlockPos, block: i32, delay: i32
    },

    MarkForRender
    {
        world: i32, pos: BlockPos
    },

    Light
    {
        world: i32, pos: BlockPos
    },

    IsPowered
    {
        world: i32, pos: BlockPos
    },

    WorldTime
    {
        world: i32
    },

    WorldRandom
    {
        world: i32, bound: i32
    },

    PlaySound
    {
        world: i32, pos: [f64; 3], sound: String, volume: f32, pitch: f32
    },

    Particle
    {
        world: i32, name: String, pos: [f64; 3], motion: [f64; 3]
    },

    Explode
    {
        world: i32, pos: [f64; 3], power: f32
    },

    /// y of the first block above the highest light blocking block in the column
    Height
    {
        world: i32, x: i32, z: i32
    },

    /// World.rayTraceBlocks between two points, answers Hit
    RayTrace
    {
        world: i32, from: [f64; 3], to: [f64; 3], liquids: bool
    },

    /// the time of every loaded world, as the time command sets it
    SetWorldTime
    {
        time: i64
    },

    /// the time a mod dimension's sun, sky and daylight show on this side, None follows the world time
    SetSkyTime
    {
        world: i32, time: Option<i64>
    },

    /// Block.dropBlockAsItem: a block's usual drops at a spot, as a harvest callback dropping them twice needs
    DropBlockItems
    {
        world: i32, pos: BlockPos, block: i32, meta: i32
    }, // abi 3: changes properties of any block, vanilla or another mod's, applied in load order after freeze

    /// abi 3: the block and its named state at a spot, answered with HostReply::State
    BlockState
    {
        world: i32, pos: BlockPos
    },

    SetBlockState
    {
        world: i32, pos: BlockPos, state: StateData, notify: bool
    },

    /// api 1.7: makes the block a mob spawner that spawns this entity, vanilla names or mod identifiers
    SetSpawner
    {
        world: i32, pos: BlockPos, entity: String
    },

    /// api 1.7: runs the block's random tick now, as a world tick would when it picks that block
    RandomTick
    {
        world: i32, pos: BlockPos
    },

    /// api 1.7: bone meal on the block, a sapling grows, crops ripen and grass sprouts plants
    Fertilize
    {
        world: i32, pos: BlockPos
    },

    /// api 1.11: replaces the block and keeps its tile entity with inventory and open screens, as beta's furnace swaps
    /// its idle and lit blocks; only the side that owns the world swaps, answered with whether the block changed
    SwapBlock
    {
        world: i32, pos: BlockPos, block: i32, meta: i32
    },

    /// api 1.12: starts rain, and thunder with it when asked, for a number of ticks in a world, or clears both
    Weather
    {
        world: i32, raining: bool, thundering: bool, ticks: i32
    },

    /// api 1.32: a rule's value in the save, answered with Bool or Int, vanilla rules are minecraft:<name> such
    /// as minecraft:keep_inventory; a multiplayer client only knows the defaults
    GameRule
    {
        rule: Identifier
    },

    /// api 1.32: changes a rule for every dimension of the save, the value must match the rule's type and range
    SetGameRule
    {
        rule: Identifier, value: GameRuleValue
    },

    /// api 1.33: the registered fluid a block holds where it stands, vanilla water and lava included, answered with
    /// FluidBlock
    FluidAt
    {
        world: i32, pos: BlockPos
    },

    /// api 1.33: the stacks a block would drop if a machine mined it, as Block.dropBlockAsItem rolls them, without
    /// removing it or spawning anything, answered with Stacks
    BlockDrops
    {
        world: i32, pos: BlockPos
    },

    /// api 1.33: places the block a stack holds at pos as ItemBlock.onItemUse does with no player, as if clicked on
    /// the face of the block behind it, answered with Bool; the caller takes the item from its stack
    PlaceBlock
    {
        world: i32, pos: BlockPos, face: i32, stack: StackData
    },

    /// api 1.33: an explosion takes one block out as Explosion.doExplosionB does: its drops each kept with a chance,
    /// the block gone, then its onBlockDestroyedByExplosion, so explosives set each other off
    BlastBlock
    {
        world: i32, pos: BlockPos, chance: f32
    },

    /// api 1.33: World.getBlockDensity, the share of rays from a point that reach an entity's box unblocked; answered
    /// with Float
    BlockDensity
    {
        world: i32, from: [f64; 3], id: i32
    },

    /// api 1.33: the one stack old mods take from a mined block, idDropped then quantityDropped rolled once with the
    /// world's random and damageDropped, as ModLoader era machines read a block; answered with Stack
    BlockDropStack
    {
        world: i32, pos: BlockPos
    },

    /// api 1.34: an explosion the host runs whole as Explosion does, blocks dropped with chance drop, entity damage
    /// scaled by damage and fire lit where fire is set, so a blast of any size fits in one guest call's time budget
    ExplodeWith
    {
        world: i32, at: [f64; 3], power: f32, drop: f32, damage: f32, fire: bool
    },

    /// api 1.46: a leaf of any mod starts checking whether a log in minecraft:logs still holds it, as BlockLeaves'
    /// decay flag, false when the block is no leaf or already checks, authority only, answered with Bool
    CheckLeafDecay
    {
        world: i32, pos: BlockPos
    },

    /// api 1.53: a block event as pistons and note blocks send, run on the block here and on every client near it,
    /// authority only
    BlockEvent
    {
        world: i32, pos: BlockPos, kind: i32, data: i32
    },
}
