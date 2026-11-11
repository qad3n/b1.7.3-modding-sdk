use crate::dispatch::{self, Command};
use crate::events::{Event, EventType};
use crate::rt::call;
use crate::{Player, Result, expectUnit, ownId};
use mod_protocol::Identifier;
use mod_protocol::abi::{EventKind, HostCall, Permission, Priority};
use mod_protocol::handshake::ChannelDirection;
use serde::de::DeserializeOwned;
use std::rc::Rc;

pub struct Events;

impl Events
{
    pub fn on<E: EventType>(&self, priority: Priority, handler: impl Fn(&mut Event) + 'static) -> Result<()>
    {
        self.onKind(E::KIND, priority, false, handler)
    }

    pub fn onKind(
        &self,
        kind: EventKind,
        priority: Priority,
        receiveCanceled: bool,
        handler: impl Fn(&mut Event) + 'static,
    ) -> Result<()>
    {
        let handle: u32 = dispatch::addEvent(Rc::new(handler));

        expectUnit(call(HostCall::Subscribe
        {
            kind,
            custom: None,
            priority,
            receive_canceled: receiveCanceled,
            handler: handle,
        }))
    }

    /// a custom event another mod defined, the defining mod must load first
    pub fn onCustom(&self, event: &str, priority: Priority, handler: impl Fn(&mut Event) + 'static) -> Result<()>
    {
        let handle: u32 = dispatch::addEvent(Rc::new(handler));
        let custom: Identifier = ownId(event)?;

        expectUnit(call(HostCall::Subscribe
        {
            kind: EventKind::Custom,
            custom: Some(custom),
            priority,
            receive_canceled: false,
            handler: handle,
        }))
    }

    pub fn define(&self, event: &str) -> Result<()> { expectUnit(call(HostCall::DefineEvent { id: ownId(event)? })) }
}

pub struct Commands;

impl Commands
{
    pub fn register(
        &self,
        name: &str,
        permission: Permission,
        handler: impl Fn(&mut Command) -> Result<()> + 'static,
    ) -> Result<()>
    {
        self.registerWithUsage(name, permission, "", handler)
    }

    pub fn registerWithUsage(
        &self,
        name: &str,
        permission: Permission,
        usage: &str,
        handler: impl Fn(&mut Command) -> Result<()> + 'static,
    ) -> Result<()>
    {
        let handle: u32 = dispatch::addCommand(Rc::new(handler));

        expectUnit(call(HostCall::RegisterCommand
        {
            name: name.to_owned(),
            permission,
            usage: usage.to_owned(),
            handler: handle,
        }))
    }
}

pub struct Keybinds;

impl Keybinds
{
    /// key is an LWJGL key code, the handler runs on press and release
    pub fn register(&self, name: &str, key: i32, handler: impl Fn(bool) + 'static) -> Result<()>
    {
        let handle: u32 = dispatch::addKey(Rc::new(handler));
        expectUnit(call(HostCall::RegisterKeybind { name: name.to_owned(), key, handler: handle }))
    }
}

pub struct Channels;

impl Channels
{
    /// typed payloads with any serde type, handlers run on the main thread and must validate serverbound input
    pub fn register<T: DeserializeOwned + 'static>(
        &self,
        name: &str,
        direction: ChannelDirection,
        handler: impl Fn(T, Player) + 'static,
    ) -> Result<()>
    {
        let handle: u32 =
            dispatch::addChannel(Rc::new(move |player: Player, bytes: &[u8]| match crate::world::decode::<T>(bytes)
            {
                Ok(message) => handler(message, player),
                Err(error) => crate::warn(&format!("bad payload: {error}")),
            }));

        expectUnit(call(HostCall::RegisterChannel
        {
            name: name.to_owned(),
            version: "1".to_owned(),
            direction,
            handler: handle,
        }))
    }

    pub fn registerRaw(
        &self,
        name: &str,
        version: &str,
        direction: ChannelDirection,
        handler: impl Fn(Player, &[u8]) + 'static,
    ) -> Result<()>
    {
        let handle: u32 = dispatch::addChannel(Rc::new(handler));

        expectUnit(call(HostCall::RegisterChannel
        {
            name: name.to_owned(),
            version: version.to_owned(),
            direction,
            handler: handle,
        }))
    }
}
