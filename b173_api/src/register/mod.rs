use crate::{Config, Result};
use mod_protocol::Identifier;
use serde::Serialize;
use serde::de::DeserializeOwned;

mod blocks;
mod content;
mod entities;
mod items;
mod machines;
mod net;
mod recipes;
mod services;
mod ui;
mod world;

pub use blocks::{BlockSpecBuilder, Blocks};
pub use content::
{
    Achievements, Attributes, Effects, Energies, Fluids, GameRules, Loot, Particles, PoolBuilder, Tags, tagValues,
    tagsOf,
};
pub use entities::{Entities, TileEntities};
pub use items::{ItemSpecBuilder, Items};
pub use machines::{MachineRecipeBuilder, Machines};
pub use net::{Channels, Commands, Events, Keybinds};
pub use recipes::{Recipes, ShapedBuilder};
pub use services::Services;
pub use ui::{ScreenBuilder, Screens};
pub use world::{Dimension, Dimensions, SpawnQuery, Structure, Structures, Worldgen, spawnEntry};

/// the context handed to #[mod_main], registration only works while the loader is in its register phase
pub struct RegisterCtx
{
    pub(crate) m_modId: String,
}

impl RegisterCtx
{
    pub fn modId(&self) -> &str { &self.m_modId }

    pub fn blocks(&mut self) -> Blocks { Blocks }

    pub fn items(&mut self) -> Items { Items }

    pub fn entities(&mut self) -> Entities { Entities }

    pub fn tileEntities(&mut self) -> TileEntities { TileEntities }

    pub fn recipes(&mut self) -> Recipes { Recipes }

    pub fn events(&mut self) -> Events { Events }

    pub fn commands(&mut self) -> Commands { Commands }

    pub fn keybinds(&mut self) -> Keybinds { Keybinds }

    pub fn channels(&mut self) -> Channels { Channels }

    pub fn screens(&mut self) -> Screens { Screens }

    pub fn config(&mut self) -> Config { Config }

    pub fn services(&mut self) -> Services { Services }

    pub fn tags(&mut self) -> Tags { Tags }

    /// needs api 1.19
    pub fn loot(&mut self) -> Loot { Loot }

    /// needs api 1.21
    pub fn fluids(&mut self) -> Fluids { Fluids }

    /// needs api 1.22
    pub fn effects(&mut self) -> Effects { Effects }

    /// needs api 1.24
    pub fn particles(&mut self) -> Particles { Particles }

    /// needs api 1.25
    pub fn achievements(&mut self) -> Achievements { Achievements }

    /// needs api 1.27
    pub fn attributes(&mut self) -> Attributes { Attributes }

    /// needs api 1.32
    pub fn gameRules(&mut self) -> GameRules { GameRules }

    /// needs api 1.33
    pub fn energy(&mut self) -> Energies { Energies }

    /// needs api 1.35
    pub fn machines(&mut self) -> Machines { Machines }

    /// needs api 1.40
    pub fn accessories(&mut self) -> crate::Accessories { crate::Accessories }

    pub fn worldgen(&mut self) -> Worldgen { Worldgen }

    pub fn dimensions(&mut self) -> Dimensions { Dimensions }

    pub fn structures(&mut self) -> Structures { Structures }

    pub fn lang(&mut self) -> Lang { Lang }
}

pub struct Lang;

impl Lang
{
    pub fn translate(&self, key: &str) -> String { crate::world::translate(key) }
}

// a builder step cannot fail, a name that does not parse is logged and the step is skipped
fn named<E: std::fmt::Display>(parsed: std::result::Result<Identifier, E>, what: &str) -> Option<Identifier>
{
    parsed.map_err(|error| crate::warn(&format!("{what}: {error}"))).ok()
}

pub fn ownIdentifier(name: &str) -> Result<Identifier> { Ok(Identifier::parseIn(name, &crate::modId())?) }

#[deprecated(note = "use toBytes")]
pub fn bytes<T: Serialize>(value: &T) -> Vec<u8> { toBytes(value) }

pub fn fromBytes<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> { crate::world::decode(bytes) }

pub fn toBytes<T: Serialize>(value: &T) -> Vec<u8> { crate::world::encode(value) }
