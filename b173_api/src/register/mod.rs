use crate::{Config, Result};
use mod_protocol::Identifier;
use serde::Serialize;
use serde::de::DeserializeOwned;

mod blocks;
mod builders;
mod content;
mod custom;
mod entities;
mod items;
mod machines;
mod net;
mod recipes;
mod services;
mod ui;
mod world;

pub use blocks::{BlockSpecBuilder, Blocks};
pub use builders::
{
    Achievement, Attribute, Build, DamageType, Effect, Energy, EnergyBuffer, Fluid, GameRule, Particle, Property, Stat,
    Tank, Tile, Variant,
};
pub(crate) use content::registered;
pub use content::
{
    Achievements, Attachments, Attributes, Components, ConfigTasks, DamageTypes, Effects, Energies, Fluids, GameRules,
    Loot, LootQuery, Particles, PoolBuilder, Stats, Tags, registryList, tagValues, tagsOf,
};
pub use custom::{DataMap, ModRegistries, ModRegistry};
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

    /// needs api 1.49, registries and data maps of this mod's own
    pub fn registries(&mut self) -> ModRegistries { ModRegistries }

    /// needs api 1.51, item stack component types
    pub fn components(&mut self) -> Components { Components }

    /// needs api 1.51, which attachments clients see
    pub fn attachments(&mut self) -> Attachments { Attachments }

    /// needs api 1.54, steps joining clients run before play
    pub fn configTasks(&mut self) -> ConfigTasks { ConfigTasks }

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

    /// needs api 1.56
    pub fn stats(&mut self) -> Stats { Stats }

    /// needs api 1.27
    pub fn attributes(&mut self) -> Attributes { Attributes }

    /// needs api 1.32
    pub fn gameRules(&mut self) -> GameRules { GameRules }

    /// needs api 1.33
    pub fn energy(&mut self) -> Energies { Energies }

    /// needs api 1.48
    pub fn damageTypes(&mut self) -> DamageTypes { DamageTypes }

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
