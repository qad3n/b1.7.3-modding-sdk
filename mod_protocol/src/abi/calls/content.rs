use crate::Identifier;
use crate::abi::{Handle, ItemModelSpec, LootTarget, RegistryKind};
use crate::spec::
{
    BlockModifySpec, BlockSpec, BlockStatesSpec, EntitySpec, ItemSpec, LootSpec, MachineRecipeSpec, MachineSpec,
    RecipeSpec, SpawnModifySpec, TileEntitySpec,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum ContentRequest
{
    RegisterBlock
    {
        name: String,
        #[serde(with = "crate::wire::named")]
        spec: BlockSpec,
    },

    RegisterItem
    {
        name: String,
        #[serde(with = "crate::wire::named")]
        spec: ItemSpec,
    },

    RegisterEntity
    {
        name: String,
        #[serde(with = "crate::wire::named")]
        spec: EntitySpec,
    },

    RegisterTileEntity
    {
        name: String,
        #[serde(with = "crate::wire::named")]
        spec: TileEntitySpec,
    },

    RegisterRecipe
    {
        name: String,
        #[serde(with = "crate::wire::named")]
        spec: RecipeSpec,
    },

    RemoveRecipe
    {
        id: Identifier
    },

    ReplaceBlock
    {
        id: Identifier,
        #[serde(with = "crate::wire::named")]
        spec: BlockSpec,
    },

    ModifyBlock
    {
        #[serde(with = "crate::wire::named")]
        spec: BlockModifySpec,
    },

    /// abi 3: adds or removes biome spawn entries for any entity, applied in load order after vanilla spawns
    ModifySpawns
    {
        #[serde(with = "crate::wire::named")]
        spec: SpawnModifySpec,
    },

    /// abi 3: named states for a block this mod registered
    RegisterBlockStates
    {
        block: Identifier,
        #[serde(with = "crate::wire::named")]
        spec: BlockStatesSpec,
    },

    /// api 1.19: loot for a block or entity of any mod or of vanilla, merged in load order with data/<modid>/loot files
    SetLoot
    {
        registry: RegistryKind,
        target: Identifier,
        #[serde(with = "crate::wire::named")]
        spec: LootSpec,
    },

    /// api 1.20: the model a tile entity type draws with, posed every frame by its behavior's pose, before any world
    TileModel
    {
        tile: Identifier,
        #[serde(with = "crate::wire::named")]
        model: crate::spec::EntityModelSpec,
    },

    /// api 1.21: a fluid named <modid>:<name>, register it before the blocks that use it as their material
    RegisterFluid
    {
        name: String,
        #[serde(with = "crate::wire::named")]
        spec: crate::spec::FluidSpec,
    },

    /// api 1.22: a status effect named <modid>:<name>
    RegisterEffect
    {
        name: String,
        #[serde(with = "crate::wire::named")]
        spec: crate::spec::EffectSpec,
    },

    /// api 1.24: a particle spawned by the name <modid>:<name> on any world, built on a beta particle
    RegisterParticle
    {
        name: String,
        #[serde(with = "crate::wire::named")]
        spec: crate::spec::ParticleSpec,
    },

    /// api 1.25: an achievement named <modid>:<name>, kept per player apart from beta's achievements
    RegisterAchievement
    {
        name: String,
        #[serde(with = "crate::wire::named")]
        spec: crate::spec::AchievementSpec,
    },

    /// api 1.29: a status effect whose behavior the mod runs for the methods listed
    RegisterEffectBehavior
    {
        name: String,
        #[serde(with = "crate::wire::named")]
        spec: crate::spec::EffectSpec,
        behavior: crate::spec::Behavior<crate::spec::EffectMethod>,
    },

    /// api 1.27: an attribute named <modid>:<name> that every living entity carries
    RegisterAttribute
    {
        name: String,
        #[serde(with = "crate::wire::named")]
        spec: crate::spec::AttributeSpec,
    },

    /// api 1.32: a game rule named <modid>:<name>, vanilla and mod rules share /gamerule
    RegisterGameRule
    {
        name: String,
        #[serde(with = "crate::wire::named")]
        spec: crate::spec::GameRuleSpec,
    },

    /// api 1.33: an energy kind named <modid>:<name>, register phase; kinds with a known worth convert into each other
    RegisterEnergy
    {
        name: String,
        #[serde(with = "crate::wire::named")]
        spec: crate::spec::EnergySpec,
    },

    /// api 1.33: draws a block's item in 3d from cuboids and its tile's model instead of a flat sprite; register
    /// phase, only the block's own mod
    BlockItemModel
    {
        block: Identifier,
        #[serde(with = "crate::wire::named")]
        model: ItemModelSpec,
    },

    /// api 1.33: caps how many items each slot of a tile entity type holds, as IInventory.getInventoryStackLimit;
    /// register phase, only the type's own mod
    TileStackLimit
    {
        tile: Identifier, limit: i32
    },

    /// api 1.33: entries for a chest loot table named <modid>:<name> that structures roll with RollLoot, merged in
    /// load order, replace drops what came before; beta's own chest tables are fixed
    SetChestLoot
    {
        table: Identifier,
        #[serde(with = "crate::wire::named")]
        spec: LootSpec,
    },

    /// api 1.35: a machine named <modid>:<name> recipe viewers show as a category
    RegisterMachine
    {
        name: Identifier,
        #[serde(with = "crate::wire::named")]
        spec: MachineSpec,
    },

    /// api 1.35: a recipe for any mod's registered machine
    AddMachineRecipe
    {
        machine: Identifier,
        #[serde(with = "crate::wire::named")]
        recipe: MachineRecipeSpec,
    },

    /// api 1.35: an item that runs a machine, minecraft:crafting and minecraft:smelting take workstations too
    AddWorkstation
    {
        machine: Identifier, item: Identifier
    },

    /// api 1.40: a fuel for a machine of any mod by its full name, registration only
    AddMachineFuel
    {
        machine: Identifier,
        #[serde(with = "crate::wire::named")]
        fuel: crate::spec::MachineFuelSpec,
    },

    /// api 1.40: an accessory slot named <modid>:<name>, registration only
    RegisterAccessorySlot
    {
        name: Identifier,
        #[serde(with = "crate::wire::named")]
        spec: crate::spec::AccessorySlotSpec,
    },

    /// api 1.42: a named loot condition the handler answers, label is how viewers name it, registration only
    RegisterLootCondition
    {
        name: String, label: String, handler: Handle
    },

    /// api 1.42: a named rule the handler answers with a drop's count, registration only
    RegisterLootCount
    {
        name: String, label: String, handler: Handle
    },

    /// api 1.42: a pool with conditional entries added to a block's, entity's or chest table's loot
    AddConditionalPool
    {
        target: LootTarget,
        id: Identifier,
        #[serde(with = "crate::wire::named")]
        pool: crate::spec::ConditionalPoolSpec,
    },

    /// api 1.44: an item that holds a fluid, registration only
    RegisterFluidContainer
    {
        #[serde(with = "crate::wire::named")]
        spec: crate::spec::FluidContainerSpec,
    },

    /// api 1.44: a machine job with rules for some of its inputs, registration only
    AddMachineJobRules
    {
        machine: Identifier,
        #[serde(with = "crate::wire::named")]
        job: crate::spec::MachineJobSpec,
        #[serde(with = "crate::wire::named")]
        rules: Vec<crate::spec::JobInputRule>,
    },

    /// api 1.45: a block model whose cuboids follow named states, registration only
    SetStateModel
    {
        block: Identifier,
        #[serde(with = "crate::wire::named")]
        model: crate::spec::StateModelSpec,
    },

    /// api 1.48: a damage type hits can carry, its behaviour comes from damage type tags such as
    /// minecraft:bypasses_armor, registration only
    RegisterDamageType
    {
        name: String,
        #[serde(with = "crate::wire::named")]
        spec: crate::spec::DamageTypeSpec,
    },

    /// api 1.50: changes chosen fields of any item, other mods changing the same field are reported, registration only
    ModifyItem
    {
        #[serde(with = "crate::wire::named")]
        spec: crate::spec::ItemModifySpec,
    },

    /// api 1.50: drops jobs of any mod's machine by what they make, while mods load
    ModifyMachine
    {
        #[serde(with = "crate::wire::named")]
        spec: crate::spec::MachineModifySpec,
    },

    /// api 1.51: an item stack component type; synced values ride along whenever the server sends the stack,
    /// registration only
    RegisterComponent
    {
        name: String, synced: bool
    },

    /// api 1.56: a stat named <modid>:<name> the stats screen lists with its title and format, registration only
    RegisterStat
    {
        name: String,
        #[serde(with = "crate::wire::named")]
        spec: crate::spec::StatSpec,
    },
}
