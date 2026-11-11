mod convert;
mod recipe;

pub use convert::{Outcome, install};
pub use recipe::{Copy, Download, Lang, RECIPE_FILE, Recipe, Rule, Slice};

// where a player drops the downloads a mod's recipe reads, beside the mod in the mods folder
pub fn inputDir(mods: &std::path::Path, modId: &str) -> std::path::PathBuf { mods.join(format!("{modId}_assets")) }
