use crate::Identifier;
use crate::abi::RegistryKind;
use crate::spec::
{
    AchievementSpec, AttributeSpec, BlockModifySpec, BlockSpec, BlockStatesSpec, DamageTypeSpec, EffectSpec,
    FeatureModifySpec, FeatureSpec, FluidSpec, GameRuleSpec, ItemSpec, LootSpec, ParticleSpec, RecipeSpec,
    SpawnModifySpec, TagEntry, TagSpec, TagValue, WorldgenSpec,
};
use serde::de::DeserializeOwned;

pub const DATA: &str = "data/";
const TOML: &str = ".toml";
const FEATURES: &str = "features/";
const TAGS: &str = "tags";

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
    if kind != TAGS && namespace != owner
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
    let registry: RegistryKind =
        RegistryKind::byFolder(folder).ok_or_else(|| format!("unknown tag registry {folder}"))?;
    let spec: TagSpec = read(text)?;
    let entries: Vec<TagEntry> =
        spec.values.iter().map(TagValue::entry).collect::<Result<_, _>>().map_err(|error| error.to_string())?;
    Ok(DataFile::Tag { registry, tag: identifier(namespace, tagPath)?, entries, replace: spec.replace })
}
