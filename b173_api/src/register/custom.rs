use crate::refs::{IntoName, Ref, kind};
use crate::rt::call;
use crate::{Error, Result, expectUnit, ownIdentifier};
use mod_protocol::Identifier;
use mod_protocol::abi::{HostCall, HostReply, MapTarget, RegistryKind};
use mod_protocol::spec::{TagEntry, TagRef};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

/// needs api 1.49, registries and data maps mods create for content other mods and data files add to
pub struct ModRegistries;

/// a registry of entries of type T, created by one mod and filled by any mod or data file under
/// data/<namespace>/registries/<owner>/<registry>/<entry>.toml
pub struct ModRegistry<T>
{
    m_id: Identifier,
    m_kind: PhantomData<T>,
}

/// one value of type T per entry of a registry, set by id or #tag from code or from
/// data/<namespace>/data_maps/<registry>/<map>.toml
pub struct DataMap<T>
{
    m_id: Identifier,
    m_kind: PhantomData<T>,
}

// a data map value travels as a TOML document with the value under one key
#[derive(Serialize, Deserialize)]
struct Wrapped<T>
{
    value: T,
}

fn encode<T: Serialize>(value: &T) -> Result<String>
{
    toml::to_string(value).map_err(|error| Error(error.to_string()))
}

fn decode<T: DeserializeOwned>(text: &str) -> Option<T>
{
    toml::from_str(text).map_err(|error| crate::warn(&format!("unreadable registry value: {error}"))).ok()
}

fn entries(reply: HostReply) -> Vec<(Identifier, String)>
{
    match reply
    {
        HostReply::Entries(entries) => entries,
        _ => Vec::new(),
    }
}

impl ModRegistries
{
    /// a registry named <modid>:<name>, synced registries reach clients with the server's entries
    pub fn create<T>(&self, name: &str, synced: bool) -> Result<ModRegistry<T>>
    {
        let id: Identifier = ownIdentifier(name)?;
        expectUnit(call(HostCall::CreateRegistry { name: id.to_string(), synced }))?;
        Ok(ModRegistry::named(id))
    }

    /// a data map named <modid>:<name> over a built in registry kind
    pub fn dataMap<T>(&self, name: &str, registry: RegistryKind, synced: bool) -> Result<DataMap<T>>
    {
        self.createMap(name, MapTarget::Kind(registry), synced)
    }

    /// a data map named <modid>:<name> over a mod registry
    pub fn dataMapOf<T, E>(&self, name: &str, registry: &ModRegistry<E>, synced: bool) -> Result<DataMap<T>>
    {
        self.createMap(name, MapTarget::Registry(registry.m_id.clone()), synced)
    }

    fn createMap<T>(&self, name: &str, target: MapTarget, synced: bool) -> Result<DataMap<T>>
    {
        let id: Identifier = ownIdentifier(name)?;
        expectUnit(call(HostCall::CreateDataMap { name: id.to_string(), target, synced }))?;
        Ok(DataMap::named(id))
    }
}

impl<T> ModRegistry<T>
{
    /// another mod's registry by its full name, usually from that mod's api crate
    pub fn named(id: impl IntoName) -> Self
    {
        let id: Identifier = id.intoName().unwrap_or_else(|error|
        {
            crate::warn(&format!("bad registry name: {error}"));
            Identifier::minecraft("invalid")
        });
        ModRegistry { m_id: id, m_kind: PhantomData }
    }

    pub fn id(&self) -> &Identifier { &self.m_id }
}

impl<T: Serialize + DeserializeOwned> ModRegistry<T>
{
    /// an entry named <modid>:<name>, while mods load
    pub fn register(&self, name: &str, value: &T) -> Result<Ref<kind::Registry>>
    {
        let id: Identifier = ownIdentifier(name)?;
        let value: String = encode(value)?;
        expectUnit(call(HostCall::AddRegistryEntry { registry: self.m_id.clone(), name: id.to_string(), value }))?;
        Ok(Ref::new(id))
    }

    pub fn entries(&self) -> Vec<(Identifier, T)>
    {
        entries(call(HostCall::RegistryEntries { registry: self.m_id.clone() }))
            .into_iter()
            .filter_map(|(id, text)| Some((id, decode(&text)?)))
            .collect()
    }

    pub fn get(&self, id: impl IntoName) -> Option<T>
    {
        let id: Identifier = id.intoName().ok()?;
        self.entries().into_iter().find(|(entry, _)| *entry == id).map(|(_, value)| value)
    }

    /// adds entries or #nested tags to a tag of this registry, while mods load
    pub fn tag(&self, tag: &str, members: &[&str]) -> Result<()>
    {
        let entries: Vec<TagEntry> = members
            .iter()
            .map(|member| Ok(TagEntry { target: TagRef::parse(member)?, required: true }))
            .collect::<Result<_>>()?;
        let tag: Identifier = Identifier::parse(tag.trim_start_matches('#'))?;
        expectUnit(call(HostCall::AddRegistryTag { registry: self.m_id.clone(), tag, entries, replace: false }))
    }

    pub fn tagged(&self, tag: &str) -> Vec<Identifier>
    {
        let Ok(tag) = Identifier::parse(tag.trim_start_matches('#'))
        else
        {
            return Vec::new();
        };

        match call(HostCall::RegistryTagValues { registry: self.m_id.clone(), tag })
        {
            HostReply::Ids(ids) => ids,
            _ => Vec::new(),
        }
    }
}

impl<T> DataMap<T>
{
    /// another mod's data map by its full name
    pub fn named(id: impl IntoName) -> Self
    {
        let id: Identifier = id.intoName().unwrap_or_else(|error|
        {
            crate::warn(&format!("bad registry name: {error}"));
            Identifier::minecraft("invalid")
        });
        DataMap { m_id: id, m_kind: PhantomData }
    }

    pub fn id(&self) -> &Identifier { &self.m_id }
}

impl<T: Serialize + DeserializeOwned> DataMap<T>
{
    /// sets the value of an entry or of every member of a #tag, while mods load; later mods win
    pub fn set(&self, key: &str, value: &T) -> Result<()>
    {
        let value: String = encode(&Wrapped { value })?;
        let values: Vec<(TagRef, String)> = vec![(TagRef::parse(key)?, value)];
        expectUnit(call(HostCall::SetDataMapValues
        {
            map: self.m_id.clone(),
            values,
            remove: Vec::new(),
            replace: false,
        }))
    }

    /// clears the value an earlier mod set for an entry or a #tag
    pub fn remove(&self, key: &str) -> Result<()>
    {
        let remove: Vec<TagRef> = vec![TagRef::parse(key)?];
        expectUnit(call(HostCall::SetDataMapValues
        {
            map: self.m_id.clone(),
            values: Vec::new(),
            remove,
            replace: false,
        }))
    }

    pub fn value(&self, id: impl IntoName) -> Option<T>
    {
        let id: Identifier = id.intoName().ok()?;
        let (_, text) = entries(call(HostCall::DataMapValue { map: self.m_id.clone(), id })).into_iter().next()?;
        decode::<Wrapped<T>>(&text).map(|wrapped| wrapped.value)
    }

    pub fn entries(&self) -> Vec<(Identifier, T)>
    {
        entries(call(HostCall::DataMapEntries { map: self.m_id.clone() }))
            .into_iter()
            .filter_map(|(id, text)| Some((id, decode::<Wrapped<T>>(&text)?.value)))
            .collect()
    }
}
