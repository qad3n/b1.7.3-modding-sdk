use serde::Deserialize;
use std::collections::BTreeMap;

pub const RECIPE_FILE: &str = "assets.toml";
// widest canvas or output a compose step may ask for so a recipe cannot exhaust memory
const COMPOSE_LIMIT: u32 = 4096;

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

// rectangles of one image pasted onto a canvas, later parts over earlier ones, then scaled to size by nearest pixel
#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Compose
{
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub from: Option<String>,
    pub to: String,
    pub canvas: [u32; 2],
    #[serde(default)]
    pub size: Option<[u32; 2]>,
    // rrggbbaa the canvas starts as, clear when empty
    #[serde(default)]
    pub fill: String,
    #[serde(default)]
    pub parts: Vec<Part>,
}

// left top width height of the image, pasted with its corner at at
#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Part
{
    pub rect: [u32; 4],
    pub at: [u32; 2],
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
    pub compose: Vec<Compose>,
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
            .chain(self.compose.iter().map(|compose| &compose.source))
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
            .chain(self.slice.iter().flat_map(|slice| slice.tiles.values().map(String::as_str)))
            .chain(self.compose.iter().map(|compose| compose.to.as_str()));

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

        for compose in &self.compose
        {
            compose.check()?;
        }

        Ok(())
    }
}

impl Compose
{
    // rgba bytes of fill, clear when it is empty
    pub(crate) fn color(&self) -> Result<[u8; 4], String>
    {
        if self.fill.is_empty()
        {
            return Ok([0; 4]);
        }

        let invalid = || format!("{RECIPE_FILE}: compose {} fill {} is not rrggbbaa", self.to, self.fill);
        if self.fill.len() != 8
        {
            return Err(invalid());
        }

        let mut color: [u8; 4] = [0; 4];
        for (i, channel) in color.iter_mut().enumerate()
        {
            *channel =
                self.fill.get(i * 2..i * 2 + 2).and_then(|hex| u8::from_str_radix(hex, 16).ok()).ok_or_else(invalid)?;
        }

        Ok(color)
    }

    fn check(&self) -> Result<(), String>
    {
        self.color()?;

        let [width, height]: [u32; 2] = self.canvas;
        let [outWidth, outHeight]: [u32; 2] = self.size.unwrap_or(self.canvas);
        let sides: [u32; 4] = [width, height, outWidth, outHeight];
        if sides.iter().any(|&side| side == 0 || side > COMPOSE_LIMIT)
        {
            return Err(format!("{RECIPE_FILE}: compose {} canvas or size is not 1 to {COMPOSE_LIMIT}", self.to));
        }

        if !self.parts.is_empty() && self.from.is_none()
        {
            return Err(format!("{RECIPE_FILE}: compose {} has parts but no from image", self.to));
        }

        for part in &self.parts
        {
            let [_, _, partWidth, partHeight]: [u32; 4] = part.rect;
            let [x, y]: [u32; 2] = part.at;
            let fits: bool = x.checked_add(partWidth).is_some_and(|right| right <= width)
                && y.checked_add(partHeight).is_some_and(|bottom| bottom <= height);
            if !fits
            {
                return Err(format!("{RECIPE_FILE}: compose {} part {:?} leaves the canvas", self.to, part.rect));
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
