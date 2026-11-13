use crate::events::Event;
use crate::rt::call;
use crate::{BlockCtx, Entity, Player, Result, World, dispatch, expectUnit, ownIdentifier};
use mod_protocol::abi::
{
    AttachmentChange, BlockPos, EventKind, Holder, HostReply, MetaRequest, Priority, StoredRequest, attachmentEvent,
};
use mod_protocol::spec::AttachmentSpec;
use mod_protocol::wire;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::marker::PhantomData;
use std::rc::Rc;

/// a named value this mod keeps on block entities, chunks, entities, players or the save, declared once while the
/// mod loads; the loader saves it, sends it to clients and keeps it through death as its options say
///
/// `static CHARGE: Attachment<i32> = Attachment::new("charge").saved().synced();`
pub struct Attachment<T>
{
    m_name: &'static str,
    m_spec: AttachmentSpec,
    m_value: PhantomData<fn() -> T>,
}

impl<T> Attachment<T>
{
    /// lives until its holder is unloaded unless an option below says more
    pub const fn new(name: &'static str) -> Self
    {
        Self { m_name: name, m_spec: AttachmentSpec { saved: false, synced: false, kept: false }, m_value: PhantomData }
    }

    /// written with its holder and read back with it
    pub const fn saved(mut self) -> Self
    {
        self.m_spec.saved = true;
        self
    }

    /// every client that sees the holder gets the value, when it changes and when the client first sees the holder
    pub const fn synced(mut self) -> Self
    {
        self.m_spec.synced = true;
        self
    }

    /// on a player, stays through death
    pub const fn kept(mut self) -> Self
    {
        self.m_spec.kept = true;
        self
    }
}

impl<T: Serialize + DeserializeOwned> Attachment<T>
{
    pub(crate) fn declare(&self) -> Result<()>
    {
        expectUnit(call(StoredRequest::Declare { name: self.m_name.to_owned(), spec: self.m_spec }))
    }

    pub(crate) fn watch(&self, handler: impl Fn(Holder, Option<T>) + 'static) -> Result<()>
    where T: 'static
    {
        let name: &'static str = self.m_name;
        let custom: mod_protocol::Identifier = attachmentEvent(&ownIdentifier(name)?);
        let handle: u32 = dispatch::addEvent(Rc::new(move |event: &mut Event|
        {
            let Some((_, bytes)) = event.custom()
            else
            {
                return;
            };

            match wire::decode::<AttachmentChange>(bytes)
            {
                Ok(change) => handler(change.holder, change.data.and_then(|data| wire::decodeStored(&data).ok())),
                Err(error) => crate::warn(&format!("attachment {name} sent a change that does not decode: {error}")),
            }
        }));

        expectUnit(call(MetaRequest::Subscribe
        {
            kind: EventKind::Custom,
            custom: Some(custom),
            priority: Priority::Normal,
            receive_canceled: false,
            handler: handle,
        }))
    }

    /// none when the holder has no value, or when the stored one can no longer be read
    pub fn get(&self, on: impl Attachable) -> Option<T>
    {
        let name: mod_protocol::Identifier = ownIdentifier(self.m_name).ok()?;
        let bytes: Vec<u8> = match call(StoredRequest::Fetch { holder: on.holder(), name })
        {
            HostReply::Stored(bytes) => bytes?,

            HostReply::Error(reason) =>
            {
                crate::warn(&reason);
                return None;
            }

            _ => return None,
        };

        match wire::decodeStored(&bytes)
        {
            Ok(value) => Some(value),

            Err(error) =>
            {
                crate::warn(&format!("attachment {} no longer decodes: {error}", self.m_name));
                None
            }
        }
    }

    /// the stored value, or the type's default when there is none
    pub fn getOr(&self, on: impl Attachable) -> T
    where T: Default
    {
        self.get(on).unwrap_or_default()
    }

    /// a saved or synced attachment changes only where the world is owned, so on a multiplayer client this is an error
    pub fn set(&self, on: impl Attachable, value: &T) -> Result<()> { self.store(on, Some(wire::encodeStored(value))) }

    pub fn clear(&self, on: impl Attachable) -> Result<()> { self.store(on, None) }

    fn store(&self, on: impl Attachable, data: Option<Vec<u8>>) -> Result<()>
    {
        let name: mod_protocol::Identifier = ownIdentifier(self.m_name)?;
        expectUnit(call(StoredRequest::Store { holder: on.holder(), name, data }))
    }
}

/// what an attachment can sit on
pub trait Attachable
{
    fn holder(&self) -> Holder;
}

/// the save as a whole, one value for every dimension
pub struct Save;

/// a chunk column of a world
pub struct ChunkAt(pub World, pub i32, pub i32);

impl Attachable for Save
{
    fn holder(&self) -> Holder { Holder::World }
}

impl Attachable for ChunkAt
{
    fn holder(&self) -> Holder { Holder::Chunk { world: self.0.dimension, x: self.1, z: self.2 } }
}

impl Attachable for (World, BlockPos)
{
    fn holder(&self) -> Holder { Holder::Tile { world: self.0.dimension, pos: self.1 } }
}

impl Attachable for &BlockCtx
{
    fn holder(&self) -> Holder { Holder::Tile { world: self.world().dimension, pos: self.pos() } }
}

impl Attachable for Entity
{
    fn holder(&self) -> Holder { Holder::Entity { world: self.world.dimension, id: self.id } }
}

impl Attachable for Player
{
    // the loader finds a player in whichever world it is
    fn holder(&self) -> Holder { Holder::Entity { world: World::OVERWORLD.dimension, id: self.id } }
}
