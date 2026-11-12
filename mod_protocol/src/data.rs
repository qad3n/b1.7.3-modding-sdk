use crate::Identifier;
use crate::abi::{BiomeSpec, LootTarget, MapTarget, RegistryKind, StructureSpec};
use crate::spec::
{
    AccessorySlotSpec, AchievementSpec, AttributeSpec, BlockModifySpec, BlockSpec, BlockStatesSpec,
    ConditionalPoolSpec, DamageTypeSpec, EffectSpec, EnergySpec, EntitySpec, FeatureModifySpec, FeatureSpec,
    FluidContainerSpec, FluidSpec, GameRuleSpec, ItemModifySpec, ItemSpec, LoadConditions, LootSpec, MachineFuelSpec,
    MachineJobSpec, MachineModifySpec, MachineRecipeSpec, MachineSpec, ParticleSpec, RecipeModifySpec, RecipeSpec,
    SpawnModifySpec, TagEntry, TagRef, TagSpec, TagValue, TileEntitySpec, WorldgenSpec,
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
const WHEN: &str = "when";
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
    ModifyItem(ItemModifySpec),
    ModifyRecipes(RecipeModifySpec),
    ModifyMachine(MachineModifySpec),
    Entity(Identifier, Box<EntitySpec>),
    TileEntity(Identifier, TileEntitySpec),
    Machine(Identifier, MachineSpec),
    MachineJob
    {
        machine: Identifier,
        job: Box<MachineJobSpec>,
    },
    MachineFuel
    {
        machine: Identifier,
        fuel: MachineFuelSpec,
    },
    MachineRecipe
    {
        machine: Identifier,
        recipe: MachineRecipeSpec,
    },
    AccessorySlot(Identifier, AccessorySlotSpec),
    Energy(Identifier, EnergySpec),
    FluidContainer(FluidContainerSpec),
    LootPool
    {
        target: LootTarget,
        id: Identifier,
        pool: ConditionalPoolSpec,
    },
    ChestLoot(Identifier, LootSpec),
    Structure(Identifier, StructureSpec),
    Biome(Identifier, BiomeSpec),
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

type Document = toml::Table;

fn read<T: DeserializeOwned>(document: &Document) -> Result<T, String>
{
    toml::Value::Table(document.clone()).try_into().map_err(|error: toml::de::Error| error.to_string())
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

/// a data file with the conditions its [when] table sets
pub struct Parsed
{
    pub file: DataFile,
    pub when: Option<LoadConditions>,
}

/// parses data/<namespace>/<kind>/<path>.toml of the mod owner, errors name the file; an owner of None is the
/// override folder, which may write any namespace
pub fn parse(owner: Option<&str>, name: &str, text: &str) -> Result<Parsed, String>
{
    let (namespace, kind, path) =
        split(name).ok_or_else(|| format!("{name} is not data/<namespace>/<kind>/<path>.toml"))?;
    if let Some(owner) = owner
        && kind != TAGS
        && kind != DATA_MAPS
        && namespace != owner
    {
        return Err(format!("{name} registers into namespace {namespace}, a mod only owns {owner}"));
    }

    let mut document: Document = toml::from_str(text).map_err(|error| format!("{name}: {error}"))?;
    let when: Option<LoadConditions> = document
        .remove(WHEN)
        .map(|table| table.try_into().map_err(|error: toml::de::Error| format!("{name} [when]: {error}")))
        .transpose()?;
    let file: DataFile = file(namespace, kind, path, &document).map_err(|reason| format!("{name}: {reason}"))?;
    Ok(Parsed { file, when })
}

fn file(namespace: &str, kind: &str, path: &str, document: &Document) -> Result<DataFile, String>
{
    let id = || identifier(namespace, path);

    match kind
    {
        "blocks" => Ok(DataFile::Block(id()?, Box::new(read(document)?))),
        "items" => Ok(DataFile::Item(id()?, Box::new(read(document)?))),
        "recipes" => Ok(DataFile::Recipe(id()?, read(document)?)),
        "states" => Ok(DataFile::States(id()?, read(document)?)),
        "achievements" => Ok(DataFile::Achievement(id()?, read(document)?)),
        "particles" => Ok(DataFile::Particle(id()?, read(document)?)),
        "effects" => Ok(DataFile::Effect(id()?, read(document)?)),
        "attributes" => Ok(DataFile::Attribute(id()?, read(document)?)),
        "gamerules" => Ok(DataFile::GameRule(id()?, read(document)?)),
        "fluids" => Ok(DataFile::Fluid(id()?, read(document)?)),
        "damage_types" => Ok(DataFile::DamageType(id()?, read(document)?)),
        "entities" => Ok(DataFile::Entity(id()?, Box::new(read(document)?))),
        "tile_entities" => Ok(DataFile::TileEntity(id()?, read(document)?)),
        "machines" => Ok(DataFile::Machine(id()?, read(document)?)),
        "machine_jobs" => Ok(DataFile::MachineJob { machine: machine(path)?, job: Box::new(read(document)?) }),
        "machine_fuels" => Ok(DataFile::MachineFuel { machine: machine(path)?, fuel: read(document)? }),
        "machine_recipes" => Ok(DataFile::MachineRecipe { machine: machine(path)?, recipe: read(document)? }),
        "accessory_slots" => Ok(DataFile::AccessorySlot(id()?, read(document)?)),
        "energy" => Ok(DataFile::Energy(id()?, read(document)?)),
        "fluid_containers" => Ok(DataFile::FluidContainer(read(document)?)),
        "loot_pools" => lootPool(path, document),
        "structures" => Ok(DataFile::Structure(id()?, read(document)?)),
        "biomes" => Ok(DataFile::Biome(id()?, read(document)?)),
        "loot" => loot(path, document),
        "worldgen" => match path.strip_prefix(FEATURES)
        {
            Some(feature) => Ok(DataFile::Feature(identifier(namespace, feature)?, Box::new(read(document)?))),
            None => Ok(DataFile::Worldgen(id()?, read(document)?)),
        },
        "modify" => modify(path, document),
        TAGS => tag(namespace, path, document),
        REGISTRIES => entry(namespace, path, document),
        DATA_MAPS => dataMap(namespace, path, document),
        other => Err(format!("unknown data kind {other}")),
    }
}

// loot/blocks/minecraft/gravel names the block minecraft:gravel, loot/chests/<namespace>/<table> a chest table
fn loot(path: &str, document: &Document) -> Result<DataFile, String>
{
    let usage = || "needs loot/<blocks|entities|chests>/<namespace>/<path>.toml".to_owned();
    let (folder, rest) = path.split_once('/').ok_or_else(usage)?;
    let (namespace, target) = rest.split_once('/').ok_or_else(usage)?;
    let target: Identifier = identifier(namespace, target)?;
    let registry: RegistryKind = match folder
    {
        "blocks" => RegistryKind::Block,
        "entities" => RegistryKind::Entity,
        "chests" => return Ok(DataFile::ChestLoot(target, read(document)?)),
        _ => return Err(usage()),
    };

    Ok(DataFile::Loot { registry, target, spec: read(document)? })
}

// loot_pools/<blocks|entities|chests>/<namespace>/<target>/<pool>.toml, the last part only names the file
fn lootPool(path: &str, document: &Document) -> Result<DataFile, String>
{
    let usage = || "needs loot_pools/<blocks|entities|chests>/<namespace>/<target>/<pool>.toml".to_owned();
    let (folder, rest) = path.split_once('/').ok_or_else(usage)?;
    let target: LootTarget = match folder
    {
        "blocks" => LootTarget::Block,
        "entities" => LootTarget::Entity,
        "chests" => LootTarget::Chest,
        _ => return Err(usage()),
    };

    let (namespace, rest) = rest.split_once('/').ok_or_else(usage)?;
    let (targetPath, _) = rest.rsplit_once('/').ok_or_else(usage)?;
    Ok(DataFile::LootPool { target, id: identifier(namespace, targetPath)?, pool: read(document)? })
}

// machine_<part>/<owner>/<machine>/<name>.toml adds to the machine <owner>:<machine>
fn machine(path: &str) -> Result<Identifier, String>
{
    modRegistry(path).map(|(machine, _)| machine).ok_or_else(|| "needs <owner>/<machine>/<name>.toml".to_owned())
}

fn modify(path: &str, document: &Document) -> Result<DataFile, String>
{
    match path.split_once('/')
    {
        Some(("blocks", _)) => Ok(DataFile::ModifyBlock(Box::new(read(document)?))),
        Some(("spawns", _)) => Ok(DataFile::ModifySpawn(read(document)?)),
        Some(("features", _)) => Ok(DataFile::ModifyFeature(read(document)?)),
        Some(("items", _)) => Ok(DataFile::ModifyItem(read(document)?)),
        Some(("recipes", _)) => Ok(DataFile::ModifyRecipes(read(document)?)),
        Some(("machines", _)) => Ok(DataFile::ModifyMachine(read(document)?)),
        _ => Err("needs modify/<blocks|items|recipes|machines|spawns|features>/<name>.toml".to_owned()),
    }
}

fn tag(namespace: &str, path: &str, document: &Document) -> Result<DataFile, String>
{
    let (folder, tagPath) = path.split_once('/').ok_or("needs tags/<registry>/<tag>.toml")?;
    if folder == REGISTRIES
    {
        let (registry, tagPath) = modRegistry(tagPath).ok_or("needs tags/registries/<owner>/<registry>/<tag>.toml")?;
        let (entries, replace) = tagEntries(document)?;
        return Ok(DataFile::RegistryTag { registry, tag: identifier(namespace, tagPath)?, entries, replace });
    }

    let registry: RegistryKind =
        RegistryKind::byFolder(folder).ok_or_else(|| format!("unknown tag registry {folder}"))?;
    let (entries, replace) = tagEntries(document)?;
    Ok(DataFile::Tag { registry, tag: identifier(namespace, tagPath)?, entries, replace })
}

fn tagEntries(document: &Document) -> Result<(Vec<TagEntry>, bool), String>
{
    let spec: TagSpec = read(document)?;
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
fn entry(namespace: &str, path: &str, document: &Document) -> Result<DataFile, String>
{
    let (registry, entry) = modRegistry(path).ok_or("needs registries/<owner>/<registry>/<entry>.toml")?;
    let value: String = toml::to_string(document).map_err(|error| error.to_string())?;
    Ok(DataFile::RegistryEntry { registry, id: identifier(namespace, entry)?, value })
}

// data_maps/<kind>/<map>.toml or data_maps/registries/<owner>/<registry>/<map>.toml for the map <namespace>:<map>
fn dataMap(namespace: &str, path: &str, document: &Document) -> Result<DataFile, String>
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

    let file: DataMapFile = read(document)?;
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
