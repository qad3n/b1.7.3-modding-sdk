use crate::{Error, Located, Result, StackData, StateData, StructureStart};
use mod_protocol::abi::{EntityInfo, HostReply};

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
