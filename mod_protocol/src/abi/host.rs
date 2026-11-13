use crate::Identifier;
use crate::abi::
{
    BlockInfo, ContainerSwap, EnergyDemand, EnergyLevel, EntityInfo, FeatureInfo, FluidBlock, FluidStack, ItemInfo,
    JobMatch, Located, LootView, MachineView, RayHit, RecipeView, SpawnEntry, StackData, StateData, StructureStart,
    TankLevel, Worn,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum HostReply
{
    Unit,
    Bool(bool),
    Int(i32),
    Long(i64),
    Float(f32),
    Text(String),
    Bytes(Vec<u8>),
    Id(Option<Identifier>),
    Ids(Vec<Identifier>),
    Stack(Option<StackData>),
    Entity(Option<EntityInfo>),
    Entities(Vec<EntityInfo>),
    Size(i32, i32),
    Error(String),
    BlockInfo(Option<BlockInfo>),
    Hit(Option<RayHit>),
    /// abi 3
    State(Option<StateData>),
    /// api 1.2
    Stacks(Vec<StackData>),
    Recipes(Vec<RecipeView>),
    /// api 1.10
    Starts(Vec<StructureStart>),
    Located(Option<Located>),
    /// api 1.27
    Double(f64),
    /// api 1.33
    Demand(Option<EnergyDemand>),
    /// api 1.33
    Fluid(Option<FluidStack>),
    /// api 1.33
    FluidBlock(Option<FluidBlock>),
    /// api 1.33
    ItemInfo(Option<ItemInfo>),
    /// api 1.33
    Loot(Vec<LootView>),
    /// api 1.35
    Machines(Vec<MachineView>),
    /// api 1.35: square argb pixels of a sprite, size 0 when there is none
    Pixels
    {
        size: u32,
        pixels: Vec<u32>,
    },
    /// api 1.40
    Job(Option<JobMatch>),
    /// api 1.40
    Accessories(Vec<Worn>),
    /// api 1.41
    Features(Vec<FeatureInfo>),
    /// api 1.43
    SpawnEntries(Vec<SpawnEntry>),
    /// api 1.44
    Swap(Option<ContainerSwap>),
    /// api 1.44
    FluidContainers(Vec<crate::spec::FluidContainerSpec>),
    /// api 1.49: registry or data map entries with their TOML text
    Entries(Vec<(Identifier, String)>),
    /// api 2.0: an attachment value, none when it was never set
    Stored(Option<Vec<u8>>),
    /// api 2.0
    Energy(Option<EnergyLevel>),
    /// api 2.0
    Tank(Option<TankLevel>),
    /// api 2.1
    Media(super::MediaStatus),
}
