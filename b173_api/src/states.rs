use crate::{BlockCtx, BlockPos, Entity, EntityField, Identifier, Result, StateData, World, epoch, modId, rt};
use mod_protocol::abi::{HostReply, RegistryRequest};
use std::cell::{Cell, Ref, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

// each entry keeps the epoch it was read in
type StateCache = HashMap<(i32, i32), (u32, Option<Rc<StateData>>)>;

thread_local!
{
    static STATES: RefCell<StateCache> = RefCell::new(HashMap::new());
}

/// needs api 1.39, the named state a numeric id and metadata hold, cached per epoch so hot code reads properties
/// without a host call or an allocation
pub fn stateOf(block: i32, meta: i32) -> Option<Rc<StateData>>
{
    let epoch: u32 = epoch();
    let cached: Option<Option<Rc<StateData>>> = STATES.with(|cache|
    {
        let cache: Ref<StateCache> = cache.borrow();
        let (at, state): &(u32, Option<Rc<StateData>>) = cache.get(&(block, meta))?;
        (*at == epoch).then(|| state.clone())
    });
    if let Some(state) = cached
    {
        return state;
    }

    let state: Option<Rc<StateData>> =
        crate::reply::expectState(rt::call(RegistryRequest::DescribeState { block, meta })).ok().flatten().map(Rc::new);
    STATES.with(|cache| cache.borrow_mut().insert((block, meta), (epoch, state.clone())));
    state
}

/// needs api 1.39, a block state kept with the numeric id and metadata it is stored as, resolved again after every
/// remap, so generators and hot block code name each state once
pub struct StateHandle
{
    m_state: StateData,
    m_resolved: Cell<Option<(u32, i32, i32)>>,
}

impl StateHandle
{
    pub fn new(state: StateData) -> StateHandle { StateHandle { m_state: state, m_resolved: Cell::new(None) } }

    /// a block's default state, a bare name means this mod's
    pub fn named(name: &str) -> Result<StateHandle>
    {
        Ok(StateHandle::new(StateData::of(Identifier::parseIn(name, &modId())?)))
    }

    pub fn state(&self) -> &StateData { &self.m_state }

    /// the id and metadata the state is stored as until the next remap, air when no block has the name
    pub fn numeric(&self) -> (i32, i32)
    {
        let epoch: u32 = epoch();
        if let Some((at, block, meta)) = self.m_resolved.get()
            && at == epoch
        {
            return (block, meta);
        }

        let (block, meta): (i32, i32) = match rt::call(RegistryRequest::ResolveState { state: self.m_state.clone() })
        {
            HostReply::Size(block, meta) => (block, meta),
            other =>
            {
                crate::warn(&format!("state of {} does not resolve: {other:?}", self.m_state.block));
                (0, 0)
            }
        };
        self.m_resolved.set(Some((epoch, block, meta)));
        (block, meta)
    }

    /// a block and metadata read back from the world hold exactly this state
    pub fn matches(&self, block: i32, meta: i32) -> bool { self.numeric() == (block, meta) }

    /// the block alone, for code that asks whether a spot holds any state of it
    pub fn block(&self) -> i32 { self.numeric().0 }

    /// sets the state at a spot, without neighbour updates when notify is false as world generation places blocks
    pub fn place(&self, world: World, pos: BlockPos, notify: bool) -> Result<bool>
    {
        let (block, meta): (i32, i32) = self.numeric();
        if notify
        {
            return world.setBlock(pos, block, meta);
        }

        world.setBlockQuiet(pos, block, meta)
    }
}

impl World
{
    /// needs api 1.39, rewrites one property of the block at a spot and keeps the others, a block with no named state
    /// there fails
    pub fn setProperty(self, pos: BlockPos, property: &str, value: impl ToString) -> Result<bool>
    {
        let state: Rc<StateData> = self.stateAt(pos).ok_or("no block state to change")?;
        self.setBlockState(pos, StateData::clone(&state).with(property, value))
    }
}

impl World
{
    /// needs api 1.39, swaps the block at a spot for a state and keeps its tile entity, as a furnace swaps its idle and
    /// lit blocks
    pub fn swapState(self, pos: BlockPos, state: &StateHandle) -> Result<bool>
    {
        let (block, meta): (i32, i32) = state.numeric();
        self.swapBlock(pos, block, meta)
    }

    /// needs api 1.39, bits of a block flying off as its digging particle, blockcrack_<id>_<meta> by state
    pub fn crack(self, state: &StateHandle, at: [f64; 3], motion: [f64; 3])
    {
        let (block, meta): (i32, i32) = state.numeric();
        self.particle(&format!("blockcrack_{block}_{meta}"), at, motion);
    }
}

impl Entity
{
    /// needs api 1.39, the block a moving_block or falling entity draws, by state instead of its stored numbers
    pub fn setCarried(self, state: &StateHandle)
    {
        let (block, meta): (i32, i32) = state.numeric();
        self.setField(EntityField::Block, f64::from(block));
        self.setField(EntityField::BlockMeta, f64::from(meta));
    }
}

impl BlockCtx
{
    /// needs api 1.39, this block's usual drops at its spot, as a harvest callback dropping them again needs
    pub fn dropItems(&self) -> Result<()>
    {
        self.world().dropBlockItems(self.pos(), self.block(), i32::from(self.meta()))
    }

    /// needs api 1.39, this block's breaking particles and sound at its spot
    pub fn breakEffect(&self) -> Result<()>
    {
        self.world().breakEffect(self.pos(), self.block(), i32::from(self.meta()))
    }
}
