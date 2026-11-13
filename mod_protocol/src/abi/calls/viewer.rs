use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum ViewerRequest
{
    /// api 1.2: every stack a creative item list shows, subtypes included, in numeric id order, each a full stack
    ItemCatalog,

    /// api 1.2: the crafting and smelting recipes the game knows, mod recipes included
    RecipeList,

    /// api 1.2: changes whenever the loader rebuilds ids or recipe tables, answered with HostReply::Int
    ContentGeneration,

    /// api 1.33: every drop table a recipe viewer shows, beta's mob, block and dungeon drops as the loader describes
    /// them and every mod's block, entity and chest loot, answered with Loot
    LootList,

    /// api 1.35: every machine with its workstations and recipes resolved to stacks, answered with Machines
    MachineList,
}
