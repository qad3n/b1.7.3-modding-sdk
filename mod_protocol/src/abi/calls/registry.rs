use crate::Identifier;
use crate::abi::{MapTarget, RegistryKind, StackData, StateData};
use crate::spec::TagEntry;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum RegistryRequest
{
    TagValues
    {
        registry: RegistryKind, tag: Identifier
    },

    Resolve
    {
        registry: RegistryKind, id: Identifier
    },

    NameOf
    {
        registry: RegistryKind, numeric: i32
    },

    /// static properties of a block id, answers BlockInfo
    BlockInfo
    {
        block: i32
    },

    /// the translated name an item shows, vanilla items included
    DisplayName
    {
        stack: StackData
    },

    /// api 1.7: the uses a damageable item has, zero for anything that does not wear
    MaxDamage
    {
        item: i32
    },

    /// api 1.8: the rgb a vanilla map draws a block in, its material's map color, answered with Int and 0 for air
    BlockMapColor
    {
        block: i32
    },

    /// api 1.15: tag entries that may name other tags with #ns:path or be optional, replace drops earlier members
    /// including the vanilla ones
    AddTagEntries
    {
        registry: RegistryKind,
        tag: Identifier,
        #[serde(with = "crate::wire::named")]
        entries: Vec<TagEntry>,
        replace: bool,
    },

    /// api 1.30: every tag of a registry that holds the id, answered with Ids
    TagsOf
    {
        registry: RegistryKind, id: Identifier
    },

    /// api 1.33: Item.getContainerItem, the stack a crafting or burning use of the item leaves behind, answered with
    /// Stack
    ContainerItem
    {
        stack: StackData
    },

    /// api 1.33: how many ticks a stack burns in a furnace, mod fuels included, answered with Int
    FuelTicks
    {
        stack: StackData
    },

    /// api 1.33: the member of a merged material a stack turns into, the stack itself when it is in no merged group;
    /// answered with Stack. The loader merges the same material added by several mods through shared c: tags
    Unified
    {
        stack: StackData
    },

    /// api 1.33: what an item is, answered with ItemInfo
    ItemInfo
    {
        stack: StackData
    },

    /// api 1.33: Block.getExplosionResistance of a block id with no exploder, what an explosion ray loses on it;
    /// answered with Float
    ExplosionResistance
    {
        block: i32
    },

    /// api 1.39: the numeric id and metadata a block state is stored as until the next remap, answered with Size, so
    /// code placing many blocks resolves each state once instead of sending its name every time
    ResolveState
    {
        state: StateData
    },

    /// api 1.39: the named state a numeric id and metadata hold, answered with State, the same answer BlockState
    /// gives for a block in the world
    DescribeState
    {
        block: i32, meta: i32
    },

    /// api 1.48: every entry of a registry, vanilla first then mods in name order, answered with Ids
    RegistryList
    {
        registry: RegistryKind
    },

    /// api 1.49: a registry of this mod's own that other mods and data files add entries to, each entry a TOML
    /// document only the owner's types read, sent to clients when synced, registration only
    CreateRegistry
    {
        name: String, synced: bool
    },

    /// api 1.49: an entry of a mod registry, name in this mod's namespace, value a TOML document, while mods load
    AddRegistryEntry
    {
        registry: Identifier, name: String, value: String
    },

    /// api 1.49: every entry of a mod registry in name order, answered with Entries
    RegistryEntries
    {
        registry: Identifier
    },

    /// api 1.49: entries or nested tags added to a tag of a mod registry, while mods load
    AddRegistryTag
    {
        registry: Identifier,
        tag: Identifier,
        #[serde(with = "crate::wire::named")]
        entries: Vec<crate::spec::TagEntry>,
        replace: bool,
    },

    /// api 1.49: the members of a tag of a mod registry, answered with Ids
    RegistryTagValues
    {
        registry: Identifier, tag: Identifier
    },

    /// api 1.49: a data map, one value per entry of a registry set by id or tag from code and data files,
    /// registration only
    CreateDataMap
    {
        name: String, target: MapTarget, synced: bool
    },

    /// api 1.49: values of a data map of any mod, each a TOML document under the key value, while mods load;
    /// replace drops what earlier mods set, remove applies after this call's own values
    SetDataMapValues
    {
        map: Identifier,
        #[serde(with = "crate::wire::named")]
        values: Vec<(crate::spec::TagRef, String)>,
        remove: Vec<crate::spec::TagRef>,
        replace: bool,
    },

    /// api 1.49: the value a data map gives one entry, answered with Entries holding it or nothing
    DataMapValue
    {
        map: Identifier, id: Identifier
    },

    /// api 1.49: every entry a data map gives a value, answered with Entries
    DataMapEntries
    {
        map: Identifier
    },
}
