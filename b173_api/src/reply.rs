use crate::{Error, Located, Result, StackData, StateData, StructureStart};
use mod_protocol::Identifier;
use mod_protocol::abi::{EnergyDemand, EntityInfo, FluidBlock, FluidStack, GameRuleValue, HostReply, ItemInfo};
use serde::de::DeserializeOwned;

// host replies the SDK expects, anything else becomes an Error carrying the host's reason
fn unexpected(reply: HostReply) -> Error
{
    match reply
    {
        HostReply::Error(reason) => Error(reason),
        other => Error(format!("unexpected host reply {other:?}")),
    }
}

pub(crate) fn expectUnit(reply: HostReply) -> Result<()>
{
    match reply
    {
        HostReply::Unit => Ok(()),
        other => Err(unexpected(other)),
    }
}

pub(crate) fn expectBool(reply: HostReply) -> Result<bool>
{
    match reply
    {
        HostReply::Bool(value) => Ok(value),
        other => Err(unexpected(other)),
    }
}

pub(crate) fn expectInt(reply: HostReply) -> Result<i32>
{
    match reply
    {
        HostReply::Int(value) => Ok(value),
        other => Err(unexpected(other)),
    }
}

pub(crate) fn expectBytes(reply: HostReply) -> Result<Vec<u8>>
{
    match reply
    {
        HostReply::Bytes(bytes) => Ok(bytes),
        HostReply::Unit => Ok(Vec::new()),
        other => Err(unexpected(other)),
    }
}

pub(crate) fn expectState(reply: HostReply) -> Result<Option<StateData>>
{
    match reply
    {
        HostReply::State(state) => Ok(state),
        other => Err(unexpected(other)),
    }
}

pub(crate) fn expectStack(reply: HostReply) -> Result<Option<StackData>>
{
    match reply
    {
        HostReply::Stack(stack) => Ok(stack),
        other => Err(unexpected(other)),
    }
}

pub(crate) fn expectStarts(reply: HostReply) -> Result<Vec<StructureStart>>
{
    match reply
    {
        HostReply::Starts(starts) => Ok(starts),
        other => Err(unexpected(other)),
    }
}

pub(crate) fn expectRule(reply: HostReply) -> Result<GameRuleValue>
{
    match reply
    {
        HostReply::Bool(value) => Ok(GameRuleValue::Bool(value)),
        HostReply::Int(value) => Ok(GameRuleValue::Int(value)),
        other => Err(unexpected(other)),
    }
}

pub(crate) fn expectDouble(reply: HostReply) -> Result<f64>
{
    match reply
    {
        HostReply::Double(value) => Ok(value),
        other => Err(unexpected(other)),
    }
}

pub(crate) fn expectLocated(reply: HostReply) -> Result<Option<Located>>
{
    match reply
    {
        HostReply::Located(found) => Ok(found),
        other => Err(unexpected(other)),
    }
}

pub(crate) fn expectEntity(reply: HostReply) -> Option<EntityInfo>
{
    match reply
    {
        HostReply::Entity(entity) => entity,
        _ => None,
    }
}

pub(crate) fn expectEntities(reply: HostReply) -> Vec<EntityInfo>
{
    match reply
    {
        HostReply::Entities(entities) => entities,
        _ => Vec::new(),
    }
}

pub(crate) fn expectText(reply: HostReply) -> Result<String>
{
    match reply
    {
        HostReply::Text(text) => Ok(text),
        other => Err(unexpected(other)),
    }
}

pub(crate) fn expectLong(reply: HostReply) -> Result<i64>
{
    match reply
    {
        HostReply::Long(value) => Ok(value),
        other => Err(unexpected(other)),
    }
}

pub(crate) fn expectFloat(reply: HostReply) -> Result<f32>
{
    match reply
    {
        HostReply::Float(value) => Ok(value),
        other => Err(unexpected(other)),
    }
}

pub(crate) fn expectId(reply: HostReply) -> Option<Identifier>
{
    match reply
    {
        HostReply::Id(id) => id,
        _ => None,
    }
}

// saved mod data, the default when nothing was stored, and a warning plus the default when it no longer decodes
pub(crate) fn fetchData<T: DeserializeOwned + Default>(reply: HostReply) -> T
{
    let bytes: Vec<u8> = expectBytes(reply).unwrap_or_default();
    if bytes.is_empty()
    {
        return T::default();
    }

    match mod_protocol::wire::decodeStored(&bytes)
    {
        Ok(value) => value,

        Err(error) =>
        {
            crate::warn(&format!("stored data no longer decodes, using the default: {error}"));
            T::default()
        }
    }
}

pub(crate) fn expectDemand(reply: HostReply) -> Option<EnergyDemand>
{
    match reply
    {
        HostReply::Demand(demand) => demand,
        _ => None,
    }
}

pub(crate) fn expectFluid(reply: HostReply) -> Option<FluidStack>
{
    match reply
    {
        HostReply::Fluid(stack) => stack,
        _ => None,
    }
}

pub(crate) fn expectFluidBlock(reply: HostReply) -> Option<FluidBlock>
{
    match reply
    {
        HostReply::FluidBlock(block) => block,
        _ => None,
    }
}

pub(crate) fn expectItemInfo(reply: HostReply) -> Option<ItemInfo>
{
    match reply
    {
        HostReply::ItemInfo(info) => info,
        _ => None,
    }
}
