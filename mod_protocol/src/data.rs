use crate::Identifier;
use crate::abi::{MapTarget, RegistryKind};
use crate::spec::
{
    AchievementSpec, AttributeSpec, BlockModifySpec, BlockSpec, BlockStatesSpec, DamageTypeSpec, EffectSpec,
    FeatureModifySpec, FeatureSpec, FluidSpec, GameRuleSpec, ItemSpec, LootSpec, ParticleSpec, RecipeSpec,
    SpawnModifySpec, TagEntry, TagRef, TagSpec, TagValue, WorldgenSpec,
};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;

pub const DATA: &str = "data/";
const TOML: &str = ".toml";
const FEATURES: &str = "features/";
const TAGS: &str = "tags";
const DATA_MAPS: &str = "data_maps";
const REGISTRIES: &str = "registries";
// the key a data map value travels under, so scalars and tables both make a TOML document
pub const VALUE_KEY: &str = "value";

/// one data file of a mod, read the same way by the loader and by cargo b173 check
pub enum DataFile
{
    Block(Identifier, Box<BlockSpec>),
    Item(Identifier, Box<ItemSpec>),
    Recipe(Identifier, RecipeSpec),
    States(Identifier, BlockStatesSpec),
    Achievement(Identifier, AchievementSpec),
    Particle(Identifier, ParticleSpec),
    Effect(Identifier, EffectSpec),
    Attribute(Identifier, AttributeSpec),
    GameRule(Identifier, GameRuleSpec),
    Fluid(Identifier, FluidSpec),
    DamageType(Identifier, DamageTypeSpec),
    Loot
    {
        registry: RegistryKind,
        target: Identifier,
        spec: LootSpec,
    },
    Feature(Identifier, Box<FeatureSpec>),
    Worldgen(Identifier, WorldgenSpec),
    ModifyBlock(Box<BlockModifySpec>),
    ModifySpawn(SpawnModifySpec),
    ModifyFeature(FeatureModifySpec),
    Tag
    {
        registry: RegistryKind,
        tag: Identifier,
        entries: Vec<TagEntry>,
        replace: bool,
    },
    RegistryTag
    {
        registry: Identifier,
        tag: Identifier,
        entries: Vec<TagEntry>,
        replace: bool,
    },
    RegistryEntry
    {
        registry: Identifier,
        id: Identifier,
        value: String,
    },
    DataMap
    {
        map: Identifier,
        target: MapTarget,
        values: Vec<(TagRef, String)>,
        remove: Vec<TagRef>,
        replace: bool,
    },
}

/// data/<namespace>/data_maps/<registry>/<map>.toml, any mod may add values to any map
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DataMapFile
{
    #[serde(default)]
    replace: bool,
    #[serde(default)]
    values: BTreeMap<String, toml::Value>,
    #[serde(default)]
    remove: Vec<String>,
}

pub fn isData(name: &str) -> bool { name.starts_with(DATA) && name.ends_with(TOML) }

fn read<T: DeserializeOwned>(text: &str) -> Result<T, String>
{
    toml::from_str(text).map_err(|error| error.to_string())
}

fn identifier(namespace: &str, path: &str) -> Result<Identifier, String>
{
    Identifier::new(namespace, path).map_err(|error| error.to_string())
}

// data/<namespace>/<kind>/<path>.toml, only tags may name a namespace other than the owning mod
fn split(name: &str) -> Option<(&str, &str, &str)>
{
    let rest: &str = name.strip_prefix(DATA)?.strip_suffix(TOML)?;
    let (namespace, rest) = rest.split_once('/')?;
    let (kind, path) = rest.split_once('/')?;
    Some((namespace, kind, path))
}

/// parses data/<namespace>/<kind>/<path>.toml of the mod owner, errors name the file
pub fn parse(owner: &str, name: &str, text: &str) -> Result<DataFile, String>
{
    let (namespace, kind, path) =
        split(name).ok_or_else(|| format!("{name} is not data/<namespace>/<kind>/<path>.toml"))?;
    if kind != TAGS && kind != DATA_MAPS && namespace != owner
    {
        return Err(format!("{name} registers into namespace {namespace}, a mod only owns {owner}"));
    }

    file(namespace, kind, path, text).map_err(|reason| format!("{name}: {reason}"))
}

fn file(namespace: &str, kind: &str, path: &str, text: &str) -> Result<DataFile, String>
{
    let id = || identifier(namespace, path);

    match kind
    {
        "blocks" => Ok(DataFile::Block(id()?, Box::new(read(text)?))),
        "items" => Ok(DataFile::Item(id()?, Box::new(read(text)?))),
        "recipes" => Ok(DataFile::Recipe(id()?, read(text)?)),
        "states" => Ok(DataFile::States(id()?, read(text)?)),
        "achievements" => Ok(DataFile::Achievement(id()?, read(text)?)),
        "particles" => Ok(DataFile::Particle(id()?, read(text)?)),
        "effects" => Ok(DataFile::Effect(id()?, read(text)?)),
        "attributes" => Ok(DataFile::Attribute(id()?, read(text)?)),
        "gamerules" => Ok(DataFile::GameRule(id()?, read(text)?)),
        "fluids" => Ok(DataFile::Fluid(id()?, read(text)?)),
        "damage_types" => Ok(DataFile::DamageType(id()?, read(text)?)),
        "loot" => loot(path, text),
        "worldgen" => match path.strip_prefix(FEATURES)
        {
            Some(feature) => Ok(DataFile::Feature(identifier(namespace, feature)?, Box::new(read(text)?))),
            None => Ok(DataFile::Worldgen(id()?, read(text)?)),
        },
        "modify" => modify(path, text),
        TAGS => tag(namespace, path, text),
        REGISTRIES => entry(namespace, path, text),
        DATA_MAPS => dataMap(namespace, path, text),
        other => Err(format!("unknown data kind {other}")),
    }
}

// loot/blocks/minecraft/gravel names the block minecraft:gravel
fn loot(path: &str, text: &str) -> Result<DataFile, String>
{
    let usage = || "needs loot/<blocks|entities>/<namespace>/<path>.toml".to_owned();
    let (folder, rest) = path.split_once('/').ok_or_else(usage)?;
    let registry: RegistryKind = match folder
    {
        "blocks" => RegistryKind::Block,
        "entities" => RegistryKind::Entity,
        _ => return Err(usage()),
    };

    let (namespace, target) = rest.split_once('/').ok_or_else(usage)?;
    Ok(DataFile::Loot { registry, target: identifier(namespace, target)?, spec: read(text)? })
}

fn modify(path: &str, text: &str) -> Result<DataFile, String>
{
    match path.split_once('/')
    {
        Some(("blocks", _)) => Ok(DataFile::ModifyBlock(Box::new(read(text)?))),
        Some(("spawns", _)) => Ok(DataFile::ModifySpawn(read(text)?)),
        Some(("features", _)) => Ok(DataFile::ModifyFeature(read(text)?)),
        _ => Err("needs modify/<blocks|spawns|features>/<name>.toml".to_owned()),
    }
}

fn tag(namespace: &str, path: &str, text: &str) -> Result<DataFile, String>
{
    let (folder, tagPath) = path.split_once('/').ok_or("needs tags/<registry>/<tag>.toml")?;
    if folder == REGISTRIES
    {
        let (registry, tagPath) = modRegistry(tagPath).ok_or("needs tags/registries/<owner>/<registry>/<tag>.toml")?;
        let (entries, replace) = tagEntries(text)?;
        return Ok(DataFile::RegistryTag { registry, tag: identifier(namespace, tagPath)?, entries, replace });
    }

    let registry: RegistryKind =
        RegistryKind::byFolder(folder).ok_or_else(|| format!("unknown tag registry {folder}"))?;
    let (entries, replace) = tagEntries(text)?;
    Ok(DataFile::Tag { registry, tag: identifier(namespace, tagPath)?, entries, replace })
}

fn tagEntries(text: &str) -> Result<(Vec<TagEntry>, bool), String>
{
    let spec: TagSpec = read(text)?;
    let entries: Vec<TagEntry> =
        spec.values.iter().map(TagValue::entry).collect::<Result<_, _>>().map_err(|error| error.to_string())?;
    Ok((entries, spec.replace))
}

// <owner>/<registry>/<rest> names the mod registry <owner>:<registry> and what follows it
fn modRegistry(path: &str) -> Option<(Identifier, &str)>
{
    let (owner, rest) = path.split_once('/')?;
    let (registry, rest) = rest.split_once('/')?;
    Some((Identifier::new(owner, registry).ok()?, rest))
}

// registries/<owner>/<registry>/<entry>.toml adds the entry <namespace>:<entry>
fn entry(namespace: &str, path: &str, text: &str) -> Result<DataFile, String>
{
    let (registry, entry) = modRegistry(path).ok_or("needs registries/<owner>/<registry>/<entry>.toml")?;
    let _: toml::Table = read(text)?;
    Ok(DataFile::RegistryEntry { registry, id: identifier(namespace, entry)?, value: text.to_owned() })
}

// data_maps/<kind>/<map>.toml or data_maps/registries/<owner>/<registry>/<map>.toml for the map <namespace>:<map>
fn dataMap(namespace: &str, path: &str, text: &str) -> Result<DataFile, String>
{
    let usage = || "needs data_maps/<registry>/<map>.toml or data_maps/registries/<owner>/<registry>/<map>.toml";
    let (folder, rest) = path.split_once('/').ok_or_else(usage)?;
    let (target, mapPath): (MapTarget, &str) = match folder
    {
        REGISTRIES =>
        {
            modRegistry(rest).map(|(registry, map)| (MapTarget::Registry(registry), map)).ok_or_else(usage)?
        }
        _ =>
        {
            (MapTarget::Kind(RegistryKind::byFolder(folder).ok_or_else(|| format!("unknown registry {folder}"))?), rest)
        }
    };

    let file: DataMapFile = read(text)?;
    let values: Vec<(TagRef, String)> = file
        .values
        .into_iter()
        .map(|(key, value)| Ok((TagRef::parse(&key).map_err(|error| error.to_string())?, mapValue(value)?)))
        .collect::<Result<_, String>>()?;
    let remove: Vec<TagRef> = file
        .remove
        .iter()
        .map(|key| TagRef::parse(key))
        .collect::<Result<_, _>>()
        .map_err(|error| error.to_string())?;
    Ok(DataFile::DataMap { map: identifier(namespace, mapPath)?, target, values, remove, replace: file.replace })
}

/// a data map value as the TOML document it travels as
pub fn mapValue(value: toml::Value) -> Result<String, String>
{
    let mut document: toml::Table = toml::Table::new();
    document.insert(VALUE_KEY.to_owned(), value);
    toml::to_string(&document).map_err(|error| error.to_string())
}
