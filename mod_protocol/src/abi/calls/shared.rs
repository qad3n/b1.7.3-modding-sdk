use crate::Identifier;
use crate::abi::{BlockPos, FluidStack, Handle};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum SharedRequest
{
    ProvideService
    {
        name: Identifier, version: u32, handler: Handle
    },

    CallService
    {
        name: Identifier, version: u32, method: String, args: Vec<u8>
    },

    HasService
    {
        name: Identifier, version: u32
    },

    RegisterLookup
    {
        api: Identifier, block: Identifier, handler: Handle
    },

    CallLookup
    {
        api: Identifier, world: i32, pos: BlockPos, side: i32, method: String, args: Vec<u8>
    },

    SendInterMod
    {
        to: String, kind: String, data: Vec<u8>
    },

    /// api 1.33: which kinds of transfer a block takes from a face, the bits of capability, answered with Int; an
    /// inventory tile counts for items, a mod tile for each transfer its behavior answers
    Capabilities
    {
        world: i32, pos: BlockPos, face: i32
    },

    /// api 1.33: the packet sizes a block takes from a face now, converted into the asker's energy kind through the
    /// energy registry, answered with Demand; None when it takes none or its kind does not convert
    EnergyDemand
    {
        world: i32, pos: BlockPos, face: i32, kind: Identifier
    },

    /// api 1.33: pushes energy of a kind into a block from a face, converted into the kind the block takes; answered
    /// with Long, the amount of the sender's kind it took
    SendEnergy
    {
        world: i32, pos: BlockPos, face: i32, kind: Identifier, amount: i64, simulate: bool
    },

    /// api 1.33: pours fluid into a block from a face, answered with Int, the amount it took
    FillFluid
    {
        world: i32, pos: BlockPos, face: i32, stack: FluidStack, simulate: bool
    },

    /// api 1.33: takes up to max fluid out of a block from a face, answered with Fluid
    DrainFluid
    {
        world: i32, pos: BlockPos, face: i32, max: i32, simulate: bool
    },

    /// api 2.0: what a block holds of an energy kind seen from a face, answered with Energy in the asker's kind,
    /// none when the block keeps no energy the loader can read
    EnergyLevel
    {
        world: i32, pos: BlockPos, face: i32, kind: Identifier
    },

    /// api 2.0: draws energy out of a block from a face in the asker's kind, answered with the Long amount taken
    TakeEnergy
    {
        world: i32, pos: BlockPos, face: i32, kind: Identifier, amount: i64, simulate: bool
    },

    /// api 2.0: a mod sets what its own tile's loader-kept energy buffer holds, in the buffer's kind
    SetEnergy
    {
        world: i32, pos: BlockPos, amount: i64
    },

    /// api 2.0: what a block's loader-kept tank holds, answered with Tank, none when it has no such tank
    TankLevel
    {
        world: i32, pos: BlockPos
    },

    /// api 2.0: a mod sets what its own tile's loader-kept tank holds, none empties it
    SetTank
    {
        world: i32, pos: BlockPos, stack: Option<FluidStack>
    },
}
