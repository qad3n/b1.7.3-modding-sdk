use super::World;
use crate::rt::call;
use crate::{Result, expectFloat, expectUnit};
use mod_protocol::abi::{BlockPos, HostCall};

impl World
{
    /// needs api 1.33, Block.getExplosionResistance of a block id, what an explosion ray loses on it
    pub fn explosionResistance(block: i32) -> f32 { expectFloat(call(HostCall::ExplosionResistance { block })).unwrap_or(0.0) }

    /// needs api 1.33, takes a block out as an explosion does: drops kept with a chance, then its explosion callback,
    /// so explosives set each other off
    pub fn blastBlock(self, pos: BlockPos, chance: f32) -> Result<()>
    {
        expectUnit(call(HostCall::BlastBlock { world: self.dimension, pos, chance }))
    }

    /// needs api 1.33, World.getBlockDensity: the share of rays from a point that reach an entity unblocked
    pub fn blockDensity(self, from: [f64; 3], entity: i32) -> f32
    {
        expectFloat(call(HostCall::BlockDensity { world: self.dimension, from, id: entity })).unwrap_or(0.0)
    }
}
