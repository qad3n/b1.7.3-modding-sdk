use crate::rt::{call, fire};
use crate::world::{decode, encode};
use crate::{Error, Result, expectBool, expectBytes};
use mod_protocol::Identifier;
use mod_protocol::abi::{BlockPos, HostReply, MetaRequest, SharedRequest};
use serde::Serialize;
use serde::de::DeserializeOwned;

/// a provider answers named methods with bytes its own api crate defines
pub trait Service
{
    fn call(&self, method: &str, args: &[u8]) -> Result<Vec<u8>>;
}

/// answers an api at a block position, such as energy storage at a machine
pub trait Lookup
{
    fn call(&self, world: i32, pos: BlockPos, side: i32, method: &str, args: &[u8]) -> Result<Vec<u8>>;
}

pub fn has(name: &str, version: u32) -> bool
{
    Identifier::parse(name)
        .is_ok_and(|name| expectBool(call(SharedRequest::HasService { name, version })).unwrap_or(false))
}

pub fn callService<A: Serialize, R: DeserializeOwned>(name: &str, version: u32, method: &str, args: &A) -> Result<R>
{
    let bytes: Vec<u8> = expectBytes(call(SharedRequest::CallService
    {
        name: Identifier::parse(name)?,
        version,
        method: method.to_owned(),
        args: encode(args),
    }))?;
    decode(&bytes)
}

/// None when no mod answers that api at the block
pub fn lookup<A: Serialize, R: DeserializeOwned>(
    api: &str,
    world: i32,
    pos: BlockPos,
    side: i32,
    method: &str,
    args: &A,
) -> Result<Option<R>>
{
    match call(SharedRequest::CallLookup
    {
        api: Identifier::parse(api)?,
        world,
        pos,
        side,
        method: method.to_owned(),
        args: encode(args),
    })
    {
        HostReply::Bytes(bytes) => decode(&bytes).map(Some),
        HostReply::Unit => Ok(None),
        HostReply::Error(reason) => Err(Error(reason)),
        other => Err(Error(format!("unexpected reply {other:?}"))),
    }
}

pub fn sendInterMod<T: Serialize>(to: &str, kind: &str, data: &T)
{
    fire(SharedRequest::SendInterMod { to: to.to_owned(), kind: kind.to_owned(), data: encode(data) });
}

pub fn postEvent<T: Serialize>(event: &str, data: &T) -> Result<bool>
{
    expectBool(call(MetaRequest::PostEvent { id: Identifier::parse(event)?, data: encode(data) }))
}
