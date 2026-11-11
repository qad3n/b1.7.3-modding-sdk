use anyhow::{Context, Result};
use mod_protocol::Identifier;
use mod_protocol::identifier::isValidModId;
use mod_protocol::spec::
{
    AchievementSpec, BlockModifySpec, BlockSpec, BlockStatesSpec, EffectSpec, FluidSpec, ItemSpec, LootSpec,
    ParticleSpec, RecipeSpec, SpawnModifySpec, TagSpec, WorldgenSpec,
};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
pub(crate) struct ManifestHead
{
    pub(crate) id: String,
    pub(crate) version: String,
    pub(crate) api: String,
}

pub(crate) fn manifest(root: &Path) -> Result<ManifestHead>
{
    let text: String =
        std::fs::read_to_string(root.join("mod.toml")).with_context(|| format!("no mod.toml in {}", root.display()))?;
    toml::from_str(&text).context("mod.toml")
}

pub(crate) fn files(root: &Path, under: &str) -> Vec<PathBuf>
{
    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![root.join(under)];
    while let Some(directory) = pending.pop()
    {
        let Ok(entries) = std::fs::read_dir(&directory)
        else
        {
            continue;
        };

        for entry in entries.flatten()
        {
            let path: PathBuf = entry.path();
            if path.is_dir()
            {
                pending.push(path)
            }
            else
            {
                found.push(path)
            }
        }
    }

    found.sort();
    found
}

fn parse<T: DeserializeOwned>(path: &Path) -> std::result::Result<T, String>
{
    let text: String = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    toml::from_str(&text).map_err(|error| error.to_string())
}

// the same rules the loader applies when it reads data/, reported all at once
pub(crate) fn run(root: &Path) -> Result<Vec<String>>
{
    let head: ManifestHead = manifest(root)?;
    let mut problems: Vec<String> = Vec::new();

    if !isValidModId(&head.id)
    {
        problems.push(format!("mod id '{}' must match [a-z0-9_]{{2,64}}", head.id));
    }

    let semverParts: usize = 3;
    if head.version.split('.').count() != semverParts
        || head.version.split('.').any(|part| part.parse::<u64>().is_err())
    {
        problems.push(format!("version '{}' must be major.minor.patch", head.version));
    }

    if !head.api.starts_with("1.")
    {
        problems.push(format!("api '{}' must be 1.x for this loader", head.api));
    }
    // data/<namespace>/<kind>/<path>
    let depth: usize = 4;

    for path in files(root, "data")
    {
        let relative: String = path.strip_prefix(root).unwrap_or(&path).to_string_lossy().replace('\\', "/");
        let parts: Vec<&str> = relative.trim_end_matches(".toml").splitn(depth, '/').collect();
        let [_, namespace, kind, name] = parts.as_slice()
        else
        {
            problems.push(format!("{relative}: expected data/<namespace>/<kind>/<path>.toml"));
            continue;
        };
        if *kind != "tags" && *namespace != head.id
        {
            problems.push(format!("{relative}: namespace {namespace} is not the mod id {}", head.id));
        }

        if let Err(error) = Identifier::new(namespace, name)
        {
            problems.push(format!("{relative}: {error}"));
        }

        let result: std::result::Result<(), String> = match *kind
        {
            "blocks" => parse::<BlockSpec>(&path).map(drop),
            "items" => parse::<ItemSpec>(&path).map(drop),
            "recipes" => parse::<RecipeSpec>(&path).map(drop),
            "worldgen" => parse::<WorldgenSpec>(&path).map(drop),
            "states" => parse::<BlockStatesSpec>(&path).map(drop),
            "modify" => match name.split_once('/')
            {
                Some(("blocks", _)) => parse::<BlockModifySpec>(&path).map(drop),
                Some(("spawns", _)) => parse::<SpawnModifySpec>(&path).map(drop),
                _ => Err("modify files go in modify/blocks/ or modify/spawns/".to_owned()),
            },
            "loot" => parse::<LootSpec>(&path).map(drop),
            "fluids" => parse::<FluidSpec>(&path).map(drop),
            "effects" => parse::<EffectSpec>(&path).map(drop),
            "particles" => parse::<ParticleSpec>(&path).map(drop),
            "achievements" => parse::<AchievementSpec>(&path).map(drop),
            "tags" => parse::<TagSpec>(&path).and_then(|spec|
            {
                spec.values.iter().try_for_each(|value| value.entry().map(drop).map_err(|error| error.to_string()))
            }),
            other => Err(format!("unknown data kind {other}")),
        };
        if let Err(reason) = result
        {
            problems.push(format!("{relative}: {reason}"));
        }
    }

    Ok(problems)
}
