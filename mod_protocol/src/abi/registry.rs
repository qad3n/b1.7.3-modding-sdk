use crate::Identifier;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum RegistryKind
{
    Block,
    Item,
    Entity,
    TileEntity,
    Recipe,
    Biome,
    Dimension,
    /// api 1.40, fluid tags only
    Fluid,
    /// api 1.48: named registries without numeric ids, listed by RegistryList and tagged under tags/<singular>
    Effect,
    Attribute,
    Particle,
    Sound,
    GameRule,
    Energy,
    Machine,
    AccessorySlot,
    Feature,
    Structure,
    LootCondition,
    DamageType,
    Achievement,
    Component,
    /// api 1.56
    Stat,
}

/// a registry kind with its data and tag folder name and the singular form paths and logs also accept
pub struct KindRow
{
    pub kind: RegistryKind,
    pub plural: &'static str,
    pub singular: &'static str,
}

impl RegistryKind
{
    pub const ROWS: [KindRow; 23] = [
        KindRow { kind: RegistryKind::Block, plural: "blocks", singular: "block" },
        KindRow { kind: RegistryKind::Item, plural: "items", singular: "item" },
        KindRow { kind: RegistryKind::Entity, plural: "entities", singular: "entity" },
        KindRow { kind: RegistryKind::TileEntity, plural: "tile_entities", singular: "tile_entity" },
        KindRow { kind: RegistryKind::Recipe, plural: "recipes", singular: "recipe" },
        KindRow { kind: RegistryKind::Biome, plural: "biomes", singular: "biome" },
        KindRow { kind: RegistryKind::Dimension, plural: "dimensions", singular: "dimension" },
        KindRow { kind: RegistryKind::Fluid, plural: "fluids", singular: "fluid" },
        KindRow { kind: RegistryKind::Effect, plural: "effects", singular: "effect" },
        KindRow { kind: RegistryKind::Attribute, plural: "attributes", singular: "attribute" },
        KindRow { kind: RegistryKind::Particle, plural: "particles", singular: "particle" },
        KindRow { kind: RegistryKind::Sound, plural: "sounds", singular: "sound" },
        KindRow { kind: RegistryKind::GameRule, plural: "gamerules", singular: "gamerule" },
        KindRow { kind: RegistryKind::Energy, plural: "energy", singular: "energy" },
        KindRow { kind: RegistryKind::Machine, plural: "machines", singular: "machine" },
        KindRow { kind: RegistryKind::AccessorySlot, plural: "accessory_slots", singular: "accessory_slot" },
        KindRow { kind: RegistryKind::Feature, plural: "features", singular: "feature" },
        KindRow { kind: RegistryKind::Structure, plural: "structures", singular: "structure" },
        KindRow { kind: RegistryKind::LootCondition, plural: "loot_conditions", singular: "loot_condition" },
        KindRow { kind: RegistryKind::DamageType, plural: "damage_types", singular: "damage_type" },
        KindRow { kind: RegistryKind::Achievement, plural: "achievements", singular: "achievement" },
        KindRow { kind: RegistryKind::Component, plural: "components", singular: "component" },
        KindRow { kind: RegistryKind::Stat, plural: "stats", singular: "stat" },
    ];

    pub fn byFolder(folder: &str) -> Option<RegistryKind>
    {
        Self::ROWS.iter().find(|row| row.plural == folder || row.singular == folder).map(|row| row.kind)
    }

    pub fn singular(self) -> &'static str
    {
        Self::ROWS.iter().find(|row| row.kind == self).map_or("unknown", |row| row.singular)
    }

    /// kinds whose entries carry numeric ids in saves and on the wire
    pub fn numbered(self) -> bool
    {
        matches!(
            self,
            RegistryKind::Block
                | RegistryKind::Item
                | RegistryKind::Entity
                | RegistryKind::TileEntity
                | RegistryKind::Recipe
                | RegistryKind::Biome
                | RegistryKind::Dimension
        )
    }
}

/// api 1.49: the registry a data map attaches values to
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum MapTarget
{
    Kind(RegistryKind),
    Registry(Identifier),
}
