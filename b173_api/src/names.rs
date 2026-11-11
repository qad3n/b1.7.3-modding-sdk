use crate::{ItemRef, RegistryKind, epoch, modId, rt};
use mod_protocol::Identifier;
use mod_protocol::abi::HostCall;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

// each entry keeps the epoch it was read in
type NameCache = HashMap<(RegistryKind, i32), (u32, Option<Identifier>)>;
type TagCache = HashMap<(RegistryKind, Identifier), (u32, Rc<HashSet<Identifier>>)>;

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
        let name: Option<Identifier> = crate::reply::expectId(rt::call(HostCall::NameOf { registry, numeric }));
        NAMES.with(|cache| cache.borrow_mut().insert((registry, numeric), (epoch, name)));
    }

    NAMES.with(|cache| read(cache.borrow().get(&(registry, numeric)).and_then(|(_, name)| name.as_ref())))
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
    let Ok(tag) = Identifier::parseIn(tag.trim_start_matches('#'), &modId())
    else
    {
        return Rc::default();
    };

    let epoch: u32 = epoch();
    if let Some((cached, members)) = TAGS.with(|cache| cache.borrow().get(&(registry, tag.clone())).cloned())
        && cached == epoch
    {
        return members;
    }

    let members: Rc<HashSet<Identifier>> = Rc::new(crate::tagValues(registry, &tag.to_string()).into_iter().collect());
    TAGS.with(|cache| cache.borrow_mut().insert((registry, tag), (epoch, Rc::clone(&members))));
    members
}

/// needs api 1.37, whether the content behind a numeric id is the named one, a bare name means this mod's
pub fn named(registry: RegistryKind, numeric: i32, name: &str) -> bool
{
    let Ok(name) = Identifier::parseIn(name, &modId())
    else
    {
        return false;
    };

    withName(registry, numeric, |own| own == Some(&name))
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
