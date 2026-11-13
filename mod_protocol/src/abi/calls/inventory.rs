use crate::Identifier;
use crate::abi::{BlockPos, FluidStack, StackData};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum InventoryRequest
{
    /// a player's inventory slot, 0 to 8 the hotbar and 9 to 35 the rest
    InventorySlot
    {
        player: i32, slot: i32
    },

    SetInventorySlot
    {
        player: i32, slot: i32, stack: Option<StackData>
    },

    /// the stack a player carries on the mouse in an open inventory
    CursorStack
    {
        player: i32
    },

    SetCursorStack
    {
        player: i32, stack: Option<StackData>
    },

    TileSlot
    {
        world: i32, pos: BlockPos, slot: i32
    },

    SetTileSlot
    {
        world: i32, pos: BlockPos, slot: i32, stack: Option<StackData>
    },

    /// a slot of the container the player has open, as its client sees it
    OpenSlot
    {
        player: i32, slot: i32
    },

    /// api 1.26: puts a stack into the inventory a block exposes from a face (0 bottom, 1 top, 2 to 5 sides), joining
    /// equal stacks first; a furnace takes ore on top and fuel on the sides; answered with Int, the count left over
    InsertItem
    {
        world: i32, pos: BlockPos, face: i32, stack: StackData, simulate: bool
    },

    /// api 1.26: takes up to max items from the first filled slot a face reaches, a furnace gives its output below;
    /// answered with Stack
    ExtractItem
    {
        world: i32, pos: BlockPos, face: i32, max: i32, simulate: bool
    },

    /// api 1.33: the result CraftingManager.findMatchingRecipe gives for a 3 by 3 grid in row order, answered with
    /// Stack
    CraftResult
    {
        grid: Vec<Option<StackData>>
    },

    /// api 1.33: how many slots the inventory tile at a block has, the slots TileSlot reads; answered with Int, 0 for
    /// a block with none
    InventorySize
    {
        world: i32, pos: BlockPos
    },

    /// api 1.33: how many slots an entity's own inventory has, a minecart's included; answered with Int, 0 for none
    EntityInventorySize
    {
        world: i32, id: i32
    },

    /// api 1.33: one slot of an entity's inventory, answered with Stack
    EntitySlot
    {
        world: i32, id: i32, slot: i32
    },

    /// api 1.33: changes one slot of an entity's inventory, on the side owning the world
    SetEntitySlot
    {
        world: i32, id: i32, slot: i32, stack: Option<StackData>
    },

    /// api 1.33: rolls a chest loot table once with a random seeded from seed, every entry on its own, answered with
    /// Stacks
    RollLoot
    {
        table: Identifier, seed: i64
    },

    /// api 1.40: the first job of a machine, in load order, whose inputs all match distinct given stacks and whose
    /// fluid inputs the given fluids cover, answered with Job
    MatchMachineJob
    {
        machine: Identifier, inputs: Vec<Option<StackData>>, fluids: Vec<FluidStack>
    },

    /// api 1.40: a stack's fuel value for a machine, from the machine's own entries or with furnace from beta's and
    /// mods' furnace fuels when it has none, 0 when it is no fuel there, answered with Int
    MachineFuel
    {
        machine: Identifier, stack: StackData, furnace: bool
    },

    /// api 1.44: the fluid a stack holds, answered with Fluid
    ContainedFluid
    {
        stack: StackData
    },

    /// api 1.44: the full container one item of an empty stack becomes with a fluid, answered with Swap
    FillContainer
    {
        stack: StackData, fluid: Identifier
    },

    /// api 1.44: what one item of a full container leaves and the fluid it gives, answered with Swap
    DrainContainer
    {
        stack: StackData
    },

    /// api 1.44: every fluid container, vanilla buckets first, answered with FluidContainers
    FluidContainers,

    /// api 1.44: as ExtractItem, taking only stacks the filter accepts, answered with Stack
    ExtractMatching
    {
        world: i32,
        pos: BlockPos,
        face: i32,
        filter: crate::spec::ItemFilter,
        max: i32,
        simulate: bool,
    },
}
