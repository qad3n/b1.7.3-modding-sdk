use crate::{ItemRef, RegistryKind, epoch, modId, rt};
use mod_protocol::Identifier;
use mod_protocol::abi::RegistryRequest;
use std::cell::{Ref, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

// each entry keeps the epoch it was read in
type NameCache = HashMap<(RegistryKind, i32), (u32, Option<Identifier>)>;
// keyed by the tag as callers spell it so a hit needs no parsing
type TagCache = HashMap<RegistryKind, HashMap<String, (u32, Rc<HashSet<Identifier>>)>>;

thread_local!
{
    static NAMES: RefCell<NameCache> = RefCell::new(HashMap::new());
    static TAGS: RefCell<TagCache> = RefCell::new(HashMap::new());
}

// hot paths ask per tick so a cache hit borrows the name instead of cloning it
fn withName<R>(registry: RegistryKind, numeric: i32, read: impl FnOnce(Option<&Identifier>) -> R) -> R
{
    let epoch: u32 = epoch();
    let fresh: bool = NAMES.with(|cache| cache.borrow().get(&(registry, numeric)).is_some_and(|(at, _)| *at == epoch));
    if !fresh
    {
        let name: Option<Identifier> = crate::reply::expectId(rt::call(RegistryRequest::NameOf { registry, numeric }));
        NAMES.with(|cache| cache.borrow_mut().insert((registry, numeric), (epoch, name)));
    }

    NAMES.with(|cache| read(cache.borrow().get(&(registry, numeric)).and_then(|(_, name)| name.as_ref())))
}

/// needs api 1.37, reads the registry name behind a numeric id without cloning it, cached like nameOf, so a hot path
/// can look the name up in its own tables
pub fn nameWith<R>(registry: RegistryKind, numeric: i32, read: impl FnOnce(Option<&Identifier>) -> R) -> R
{
    withName(registry, numeric, read)
}

/// the registry name behind a numeric id, cached until the id map or the tags change; a hidden state page id
/// answers with the block it extends
pub fn nameOf(registry: RegistryKind, numeric: i32) -> Option<Identifier>
{
    withName(registry, numeric, |name| name.cloned())
}

/// needs api 1.37, every member of a tag with nested tags resolved, cached like nameOf
pub fn tagMembers(registry: RegistryKind, tag: &str) -> Rc<HashSet<Identifier>>
{
    let text: &str = tag.trim_start_matches('#');
    let epoch: u32 = epoch();
    let cached: Option<Rc<HashSet<Identifier>>> = TAGS.with(|cache|
    {
        let cache: Ref<TagCache> = cache.borrow();
        let (at, members): &(u32, Rc<HashSet<Identifier>>) = cache.get(&registry)?.get(text)?;
        (*at == epoch).then(|| Rc::clone(members))
    });
    if let Some(members) = cached
    {
        return members;
    }

    let members: Rc<HashSet<Identifier>> = match Identifier::parseIn(text, &modId())
    {
        Ok(tag) => Rc::new(crate::tagValues(registry, &tag.to_string()).into_iter().collect()),
        Err(_) => Rc::default(),
    };
    TAGS.with(|cache|
    {
        cache.borrow_mut().entry(registry).or_default().insert(text.to_owned(), (epoch, Rc::clone(&members)))
    });
    members
}

// a bare name means this mod's namespace, compared as text so a cache hit allocates nothing
fn matches(id: &Identifier, name: &str) -> bool
{
    match name.split_once(':')
    {
        Some((namespace, path)) => id.namespace() == namespace && id.path() == path,
        None => id.path() == name && crate::MOD_ID.with(|own| id.namespace() == own.borrow().as_str()),
    }
}

/// needs api 1.37, whether the content behind a numeric id is the named one, a bare name means this mod's
pub fn named(registry: RegistryKind, numeric: i32, name: &str) -> bool
{
    withName(registry, numeric, |own| own.is_some_and(|own| matches(own, name)))
}

/// needs api 1.37, whether the content behind a numeric id sits in the tag
pub fn tagged(registry: RegistryKind, numeric: i32, tag: &str) -> bool
{
    let members: Rc<HashSet<Identifier>> = tagMembers(registry, tag);
    withName(registry, numeric, |name| name.is_some_and(|name| members.contains(name)))
}

/// needs api 1.37, the block behind a numeric id is the named one
pub fn blockIs(block: i32, name: &str) -> bool { named(RegistryKind::Block, block, name) }

/// needs api 1.37, the block behind a numeric id sits in the tag
pub fn blockTagged(block: i32, tag: &str) -> bool { tagged(RegistryKind::Block, block, tag) }

impl ItemRef
{
    /// needs api 1.37, whether the item sits in the tag
    pub fn tagged(&self, tag: &str) -> bool
    {
        match self
        {
            ItemRef::Named(name) => tagMembers(RegistryKind::Item, tag).contains(name),
            ItemRef::Numeric(numeric) => tagged(RegistryKind::Item, *numeric, tag),
        }
    }
}

/// needs api 1.47, a block by name or by the numeric handle a callback gave, for the world calls that take a block
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum BlockRef
{
    Named(Identifier),
    Numeric(i32),
}

impl BlockRef
{
    /// the numeric handle under this session's id map, cached per content epoch
    pub fn id(&self) -> crate::Result<i32>
    {
        match self
        {
            BlockRef::Named(id) => crate::resolve(RegistryKind::Block, id),
            BlockRef::Numeric(numeric) => Ok(*numeric),
        }
    }
}

impl From<i32> for BlockRef
{
    fn from(numeric: i32) -> BlockRef { BlockRef::Numeric(numeric) }
}

impl From<&str> for BlockRef
{
    fn from(name: &str) -> BlockRef
    {
        match Identifier::parseIn(name, &modId())
        {
            Ok(id) => BlockRef::Named(id),

            Err(error) =>
            {
                crate::warn(&format!("block name '{name}' is not an identifier, using air: {error}"));
                BlockRef::Named(Identifier::minecraft("air"))
            }
        }
    }
}

impl From<Identifier> for BlockRef
{
    fn from(id: Identifier) -> BlockRef { BlockRef::Named(id) }
}

impl From<&crate::StateHandle> for BlockRef
{
    fn from(state: &crate::StateHandle) -> BlockRef { BlockRef::Numeric(state.numeric().0) }
}
