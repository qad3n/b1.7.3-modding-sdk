use super::{World, send};
use crate::Result;
use mod_protocol::abi::{BlockCtx as RawCtx, BlockPos, StateData, Target};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::rc::Rc;

/// what a block callback knows: where it runs and whether this side owns the world
#[derive(Clone, Copy, Debug)]
pub struct BlockCtx
{
    pub raw: RawCtx,
}

impl BlockCtx
{
    pub fn world(&self) -> World { World { dimension: self.raw.world } }

    pub fn pos(&self) -> BlockPos { self.raw.pos }

    pub fn block(&self) -> i32 { self.raw.block }

    /// this block's named state, the way to read variants of a block with more than 16 states
    pub fn state(&self) -> Option<StateData> { self.world().blockState(self.raw.pos) }

    pub fn setState(&self, state: StateData) -> Result<bool> { self.world().setBlockState(self.raw.pos, state) }

    /// needs api 1.39, this block's named state from the per epoch cache, still the removed one's in removal callbacks
    pub fn stateAt(&self) -> Option<Rc<StateData>> { crate::stateOf(self.raw.block, self.raw.meta) }

    /// needs api 1.37, this block is the named one, a bare name means this mod's
    pub fn is(&self, name: &str) -> bool { crate::blockIs(self.raw.block, name) }

    /// needs api 1.37, this block sits in the tag
    pub fn tagged(&self, tag: &str) -> bool { crate::blockTagged(self.raw.block, tag) }

    pub fn meta(&self) -> u8 { self.raw.meta as u8 }

    pub fn isAuthority(&self) -> bool { self.raw.authority }

    pub fn setMeta(&mut self, meta: u8)
    {
        if self.world().setMeta(self.raw.pos, meta as i32).is_ok()
        {
            self.raw.meta = meta as i32;
        }
    }

    pub fn tileData<T: DeserializeOwned + Default>(&self) -> T { self.world().tileData(self.raw.pos) }

    pub fn setTileData<T: Serialize>(&self, value: &T) -> Result<()> { self.world().setTileData(self.raw.pos, value) }

    pub fn sendTracking<T: Serialize>(&self, channel: &str, message: &T) -> Result<()>
    {
        send(channel, Target::Tracking(self.raw.world, self.raw.pos), message)
    }
}
