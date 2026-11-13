use super::StackData;
use crate::Identifier;
use serde::{Deserialize, Serialize};

/// api 1.35: a machine category as a recipe viewer shows it
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct MachineView
{
    pub id: Identifier,
    pub title: String,
    pub workstations: Vec<StackData>,
    pub recipes: Vec<MachineRecipeView>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct MachineRecipeView
{
    pub inputs: Vec<MachineSlot>,
    pub outputs: Vec<MachineSlot>,
    pub notes: Vec<String>,
}

/// one slot of a machine recipe, several stacks where a tag fills it, damage -1 takes any
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct MachineSlot
{
    pub stacks: Vec<StackData>,
    pub chance: f32,
}

/// api 1.33: one drop table as a recipe viewer shows it
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct LootView
{
    pub source: LootSource,
    /// times the table is rolled for one source, as the 8 picks of a beta dungeon chest
    pub rolls: i32,
    /// chances and counts measured by rolling the block's drop code, not read from a table
    pub sampled: bool,
    pub drops: Vec<LootDrop>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum LootSource
{
    /// a block broken with this metadata
    Block
    {
        block: i32, meta: i32
    },

    Entity
    {
        entity: Identifier
    },

    Chest
    {
        table: Identifier
    },
}

/// one drop of a table, stack.count is the least it drops and a damage of -1 any damage
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct LootDrop
{
    pub stack: StackData,
    pub max: i32,
    pub chance: f32,
    /// only for an entity a player killed
    pub player: bool,
    /// any other condition in words, as a creeper's disc needing a skeleton's arrow
    pub note: Option<String>,
}

/// api 1.2: a recipe as a viewer shows it, an ingredient with damage -1 takes any damage
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct RecipeView
{
    /// the recipe registry name, minecraft:crafting/<output> for vanilla or the mod's own, none if added outside the loader
    pub id: Option<Identifier>,
    pub output: StackData,
    pub shape: RecipeShape,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum RecipeShape
{
    /// ingredients row by row, width per row
    Shaped
    {
        width: i32, inputs: Vec<Option<StackData>>
    },

    Shapeless
    {
        inputs: Vec<StackData>
    },

    Smelting
    {
        input: StackData
    },
}
