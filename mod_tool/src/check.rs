use anyhow::{Context, Result};
use mod_protocol::data;
use mod_protocol::identifier::isValidModId;
use serde::Deserialize;
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

    let major: String = format!("{}.", mod_protocol::API_MAJOR);
    if !head.api.starts_with(&major)
    {
        problems.push(format!("api '{}' must be {major}x for this loader", head.api));
    }
    for path in files(root, "data")
    {
        let relative: String = path.strip_prefix(root).unwrap_or(&path).to_string_lossy().replace('\\', "/");
        let checked: std::result::Result<(), String> = std::fs::read_to_string(&path)
            .map_err(|error| format!("{relative}: {error}"))
            .and_then(|text| data::parse(Some(&head.id), &relative, &text).map(drop));
        problems.extend(checked.err());
    }

    if let Ok(text) = std::fs::read_to_string(root.join(mod_assets::RECIPE_FILE))
        && let Err(reason) = mod_assets::Recipe::parse(&text)
    {
        problems.push(reason);
    }

    Ok(problems)
}
