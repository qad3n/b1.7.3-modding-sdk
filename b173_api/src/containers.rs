use crate::rt::call;
use crate::{ItemStack, Result, ownIdentifier};
use mod_protocol::Identifier;
use mod_protocol::abi::{ContainerSwap, ContentRequest, FluidStack, HostReply, InventoryRequest, StackData};
use mod_protocol::spec::FluidContainerSpec;

fn swap(call: InventoryRequest) -> Option<ContainerSwap>
{
    match crate::rt::call(call)
    {
        HostReply::Swap(swap) => swap,
        _ => None,
    }
}

/// needs api 1.44, the fluid a stack holds, a water bucket, an oil cell or any mod's container
pub fn containedFluid(stack: &ItemStack) -> Option<FluidStack>
{
    let stack: StackData = stack.raw().ok()?;
    crate::expectFluid(call(InventoryRequest::ContainedFluid { stack }))
}

/// needs api 1.44, the full container one item of an empty stack becomes with fluid and the millibuckets it takes
pub fn fill(empty: &ItemStack, fluid: impl crate::IntoName) -> Option<(ItemStack, i32)>
{
    let stack: StackData = empty.raw().ok()?;
    let fluid: Identifier = fluid.intoName().ok()?;
    let filled: ContainerSwap = swap(InventoryRequest::FillContainer { stack, fluid })?;
    Some((ItemStack::fromRaw(filled.stack?), filled.fluid.amount))
}

/// needs api 1.44, what one item of a full container leaves, none when it is used up, and the fluid it gives
pub fn drainItem(full: &ItemStack) -> Option<(Option<ItemStack>, FluidStack)>
{
    let stack: StackData = full.raw().ok()?;
    let drained: ContainerSwap = swap(InventoryRequest::DrainContainer { stack })?;
    Some((drained.stack.map(ItemStack::fromRaw), drained.fluid))
}

/// needs api 1.44, every fluid container any mod registered, vanilla buckets first
pub fn fluidContainers() -> Vec<FluidContainerSpec>
{
    match call(InventoryRequest::FluidContainers)
    {
        HostReply::FluidContainers(containers) => containers,
        _ => Vec::new(),
    }
}

pub(crate) fn register(full: &str, empty: Option<&str>, fluid: &str, amount: i32) -> Result<()>
{
    let spec: FluidContainerSpec = FluidContainerSpec
    {
        full: ownIdentifier(full)?,
        full_meta: 0,
        empty: empty.map(ownIdentifier).transpose()?,
        fluid: ownIdentifier(fluid)?,
        amount,
    };
    crate::expectUnit(call(ContentRequest::RegisterFluidContainer { spec }))
}
