use serde::Deserialize;
use std::collections::BTreeMap;

pub const RECIPE_FILE: &str = "assets.toml";

// a file the player downloads themselves and drops in mods/<modid>_assets
#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Download
{
    pub name: String,
    #[serde(default)]
    pub url: String,
    // accepted versions, any file with this name when empty
    #[serde(default)]
    pub sha256: Vec<String>,
}

// one file, or every file below a folder when from ends with a slash
#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Copy
{
    #[serde(default)]
    pub source: Option<String>,
    pub from: String,
    pub to: String,
}

// a terrain style sheet cut into one png per tile, tiles counted left to right then down
#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Slice
{
    #[serde(default)]
    pub source: Option<String>,
    pub from: String,
    pub to: String,
    pub tile: u32,
    pub tiles: BTreeMap<String, String>,
}

// keys starting with from become to plus the rest, with cut removed from the end and add appended
#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Rule
{
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub cut: String,
    #[serde(default)]
    pub add: String,
}

// a key=value or json lang file, keys no rule matches are dropped
#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Lang
{
    #[serde(default)]
    pub source: Option<String>,
    pub from: String,
    pub rules: Vec<Rule>,
}

#[derive(Deserialize, Clone, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Recipe
{
    pub download: Vec<Download>,
    pub copy: Vec<Copy>,
    pub slice: Vec<Slice>,
    pub lang: Vec<Lang>,
}

impl Recipe
{
    pub fn parse(text: &str) -> Result<Self, String>
    {
        let recipe: Self = toml::from_str(text).map_err(|error| format!("{RECIPE_FILE}: {error}"))?;
        recipe.check()?;
        Ok(recipe)
    }

    // index of the download a step reads, the first one when it names none
    pub fn download(&self, name: Option<&str>) -> Result<usize, String>
    {
        match name
        {
            None if self.download.is_empty() => Err(format!("{RECIPE_FILE} has no [[download]]")),
            None => Ok(0),
            Some(name) => self
                .download
                .iter()
                .position(|download| download.name == name)
                .ok_or_else(|| format!("{RECIPE_FILE} names no download called {name}")),
        }
    }

    fn check(&self) -> Result<(), String>
    {
        let sources = self
            .copy
            .iter()
            .map(|copy| &copy.source)
            .chain(self.slice.iter().map(|slice| &slice.source))
            .chain(self.lang.iter().map(|lang| &lang.source));

        for source in sources
        {
            self.download(source.as_deref())?;
        }

        let targets = self
            .copy
            .iter()
            .map(|copy| copy.to.as_str())
            .chain(self.slice.iter().map(|slice| slice.to.as_str()))
            .chain(self.slice.iter().flat_map(|slice| slice.tiles.values().map(String::as_str)));

        for target in targets
        {
            if !inside(target)
            {
                return Err(format!("{RECIPE_FILE}: {target} leaves the mod's assets folder"));
            }
        }

        for copy in &self.copy
        {
            if copy.from.ends_with('/') != copy.to.ends_with('/')
            {
                return Err(format!(
                    "{RECIPE_FILE}: copy {} to {} must both be folders or both files",
                    copy.from, copy.to
                ));
            }
        }

        for slice in &self.slice
        {
            if slice.tile == 0
            {
                return Err(format!("{RECIPE_FILE}: slice {} has a zero tile size", slice.from));
            }

            if let Some(bad) = slice.tiles.keys().find(|key| key.parse::<u32>().is_err())
            {
                return Err(format!("{RECIPE_FILE}: slice {} tile {bad} is not a number", slice.from));
            }
        }

        Ok(())
    }
}

// a relative path that stays below the folder it is joined to
pub fn inside(path: &str) -> bool
{
    !path.is_empty() && !path.starts_with('/') && !path.contains('\\') && path.split('/').all(|part| part != "..")
}
