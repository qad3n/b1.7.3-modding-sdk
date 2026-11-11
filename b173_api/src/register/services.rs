use crate::rt::call;
use crate::services::{Lookup, Service};
use crate::{Result, dispatch, expectUnit, ownId};
use mod_protocol::Identifier;
use mod_protocol::abi::HostCall;
use std::rc::Rc;

pub struct Services;

impl Services
{
    pub fn provide(&self, name: &str, version: u32, service: impl Service + 'static) -> Result<()>
    {
        let handle: u32 = dispatch::addService(Rc::new(service));
        expectUnit(call(HostCall::ProvideService { name: Identifier::parse(name)?, version, handler: handle }))
    }

    pub fn lookup(&self, api: &str, block: &str, lookup: impl Lookup + 'static) -> Result<()>
    {
        let handle: u32 = dispatch::addLookup(Rc::new(lookup));
        let api: Identifier = Identifier::parse(api)?;
        expectUnit(call(HostCall::RegisterLookup { api, block: ownId(block)?, handler: handle }))
    }

    pub fn onInterMod(&self, handler: impl Fn(&str, &str, &[u8]) + 'static) { dispatch::setInterMod(Rc::new(handler)); }
}
