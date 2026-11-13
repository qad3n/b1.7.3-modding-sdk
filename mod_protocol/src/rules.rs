use crate::abi::
{
    ContentRequest, EntityRequest, HostCall, InventoryRequest, MediaRequest, MetaRequest, NetRequest, PlayerRequest,
    RegistryRequest, ScreenRequest, SharedRequest, StoredRequest, ViewerRequest, WorldRequest, WorldgenRequest,
};

/// who may make a host call
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CallSide
{
    /// either side
    Any,
    /// only the side that owns the world: a dedicated server, or the game in single player
    Authority,
}

impl HostCall
{
    pub fn side(&self) -> CallSide
    {
        match self
        {
            HostCall::Content(call) => call.side(),
            HostCall::Registry(call) => call.side(),
            HostCall::Worldgen(call) => call.side(),
            HostCall::World(call) => call.side(),
            HostCall::Entity(call) => call.side(),
            HostCall::Player(call) => call.side(),
            HostCall::Inventory(call) => call.side(),
            HostCall::Stored(call) => call.side(),
            HostCall::Shared(call) => call.side(),
            HostCall::Net(call) => call.side(),
            HostCall::Screen(call) => call.side(),
            HostCall::Viewer(call) => call.side(),
            HostCall::Meta(call) => call.side(),
            HostCall::Media(call) => call.side(),
        }
    }
}

impl ContentRequest
{
    /// one row per call, with no catch-all arm, so a new call cannot go unclassified
    pub fn side(&self) -> CallSide
    {
        match self
        {
            ContentRequest::RegisterBlock { .. }
            | ContentRequest::RegisterItem { .. }
            | ContentRequest::RegisterEntity { .. }
            | ContentRequest::RegisterTileEntity { .. }
            | ContentRequest::RegisterRecipe { .. }
            | ContentRequest::RemoveRecipe { .. }
            | ContentRequest::ReplaceBlock { .. }
            | ContentRequest::ModifyBlock { .. }
            | ContentRequest::ModifySpawns { .. }
            | ContentRequest::RegisterBlockStates { .. }
            | ContentRequest::SetLoot { .. }
            | ContentRequest::TileModel { .. }
            | ContentRequest::RegisterFluid { .. }
            | ContentRequest::RegisterEffect { .. }
            | ContentRequest::RegisterParticle { .. }
            | ContentRequest::RegisterAchievement { .. }
            | ContentRequest::RegisterEffectBehavior { .. }
            | ContentRequest::RegisterAttribute { .. }
            | ContentRequest::RegisterGameRule { .. }
            | ContentRequest::RegisterEnergy { .. }
            | ContentRequest::BlockItemModel { .. }
            | ContentRequest::TileStackLimit { .. }
            | ContentRequest::SetChestLoot { .. }
            | ContentRequest::RegisterMachine { .. }
            | ContentRequest::AddMachineRecipe { .. }
            | ContentRequest::AddWorkstation { .. }
            | ContentRequest::AddMachineFuel { .. }
            | ContentRequest::RegisterAccessorySlot { .. }
            | ContentRequest::RegisterLootCondition { .. }
            | ContentRequest::RegisterLootCount { .. }
            | ContentRequest::AddConditionalPool { .. }
            | ContentRequest::RegisterFluidContainer { .. }
            | ContentRequest::AddMachineJobRules { .. }
            | ContentRequest::SetStateModel { .. }
            | ContentRequest::RegisterDamageType { .. }
            | ContentRequest::ModifyItem { .. }
            | ContentRequest::ModifyMachine { .. }
            | ContentRequest::RegisterComponent { .. }
            | ContentRequest::RegisterStat { .. } => CallSide::Any,
        }
    }
}

impl RegistryRequest
{
    /// one row per call, with no catch-all arm, so a new call cannot go unclassified
    pub fn side(&self) -> CallSide
    {
        match self
        {
            RegistryRequest::TagValues { .. }
            | RegistryRequest::Resolve { .. }
            | RegistryRequest::NameOf { .. }
            | RegistryRequest::BlockInfo { .. }
            | RegistryRequest::DisplayName { .. }
            | RegistryRequest::MaxDamage { .. }
            | RegistryRequest::BlockMapColor { .. }
            | RegistryRequest::AddTagEntries { .. }
            | RegistryRequest::TagsOf { .. }
            | RegistryRequest::ContainerItem { .. }
            | RegistryRequest::FuelTicks { .. }
            | RegistryRequest::Unified { .. }
            | RegistryRequest::ItemInfo { .. }
            | RegistryRequest::ExplosionResistance { .. }
            | RegistryRequest::ResolveState { .. }
            | RegistryRequest::DescribeState { .. }
            | RegistryRequest::RegistryList { .. }
            | RegistryRequest::CreateRegistry { .. }
            | RegistryRequest::AddRegistryEntry { .. }
            | RegistryRequest::RegistryEntries { .. }
            | RegistryRequest::AddRegistryTag { .. }
            | RegistryRequest::RegistryTagValues { .. }
            | RegistryRequest::CreateDataMap { .. }
            | RegistryRequest::SetDataMapValues { .. }
            | RegistryRequest::DataMapValue { .. }
            | RegistryRequest::DataMapEntries { .. } => CallSide::Any,
        }
    }
}

impl WorldgenRequest
{
    /// one row per call, with no catch-all arm, so a new call cannot go unclassified
    pub fn side(&self) -> CallSide
    {
        match self
        {
            WorldgenRequest::AddStructure { .. }
            | WorldgenRequest::SetBiome { .. }
            | WorldgenRequest::PlaceFeature { .. }
            | WorldgenRequest::PlaceTemplate { .. } => CallSide::Authority,

            WorldgenRequest::RegisterWorldgen { .. }
            | WorldgenRequest::RegisterGenerator { .. }
            | WorldgenRequest::RegisterBiome { .. }
            | WorldgenRequest::RegisterDimension { .. }
            | WorldgenRequest::RegisterTerrain { .. }
            | WorldgenRequest::RegisterBiomeSource { .. }
            | WorldgenRequest::WorldSeed { .. }
            | WorldgenRequest::Colorizer { .. }
            | WorldgenRequest::DimensionHeight { .. }
            | WorldgenRequest::RegisterSpawns { .. }
            | WorldgenRequest::RegisterStructure { .. }
            | WorldgenRequest::FindStructures { .. }
            | WorldgenRequest::LocateStructure { .. }
            | WorldgenRequest::DimensionSeed { .. }
            | WorldgenRequest::BiomeAt { .. }
            | WorldgenRequest::RegisterFeature { .. }
            | WorldgenRequest::Features
            | WorldgenRequest::BiomeSpawns { .. }
            | WorldgenRequest::RegisterTemplate { .. } => CallSide::Any,
        }
    }
}

impl WorldRequest
{
    /// one row per call, with no catch-all arm, so a new call cannot go unclassified
    pub fn side(&self) -> CallSide
    {
        match self
        {
            WorldRequest::SetBlock { .. }
            | WorldRequest::SetMeta { .. }
            | WorldRequest::Lightning { .. }
            | WorldRequest::BlockEffect { .. }
            | WorldRequest::NotifyNeighbors { .. }
            | WorldRequest::ScheduleTick { .. }
            | WorldRequest::Explode { .. }
            | WorldRequest::SetWorldTime { .. }
            | WorldRequest::DropBlockItems { .. }
            | WorldRequest::SetBlockState { .. }
            | WorldRequest::SetSpawner { .. }
            | WorldRequest::RandomTick { .. }
            | WorldRequest::Fertilize { .. }
            | WorldRequest::SwapBlock { .. }
            | WorldRequest::Weather { .. }
            | WorldRequest::SetGameRule { .. }
            | WorldRequest::PlaceBlock { .. }
            | WorldRequest::BlastBlock { .. }
            | WorldRequest::ExplodeWith { .. }
            | WorldRequest::CheckLeafDecay { .. }
            | WorldRequest::BlockEvent { .. } => CallSide::Authority,

            WorldRequest::IsAuthority { .. }
            | WorldRequest::GetBlock { .. }
            | WorldRequest::GetMeta { .. }
            | WorldRequest::MarkForRender { .. }
            | WorldRequest::Light { .. }
            | WorldRequest::IsPowered { .. }
            | WorldRequest::WorldTime { .. }
            | WorldRequest::WorldRandom { .. }
            | WorldRequest::PlaySound { .. }
            | WorldRequest::Particle { .. }
            | WorldRequest::Height { .. }
            | WorldRequest::RayTrace { .. }
            | WorldRequest::SetSkyTime { .. }
            | WorldRequest::BlockState { .. }
            | WorldRequest::GameRule { .. }
            | WorldRequest::FluidAt { .. }
            | WorldRequest::BlockDrops { .. }
            | WorldRequest::BlockDensity { .. }
            | WorldRequest::BlockDropStack { .. } => CallSide::Any,
        }
    }
}

impl EntityRequest
{
    /// one row per call, with no catch-all arm, so a new call cannot go unclassified
    pub fn side(&self) -> CallSide
    {
        match self
        {
            EntityRequest::SpawnEntity { .. }
            | EntityRequest::SpawnItem { .. }
            | EntityRequest::KillEntity { .. }
            | EntityRequest::MoveTowards { .. }
            | EntityRequest::Mount { .. }
            | EntityRequest::DamageEntityAs { .. }
            | EntityRequest::SetModifier { .. }
            | EntityRequest::RemoveModifier { .. }
            | EntityRequest::AddEffect { .. }
            | EntityRequest::RemoveEffect { .. }
            | EntityRequest::SetItemStack { .. }
            | EntityRequest::DropItem { .. }
            | EntityRequest::SetAccessory { .. } => CallSide::Authority,

            EntityRequest::Entity { .. }
            | EntityRequest::EntitiesNear { .. }
            | EntityRequest::MoveEntity { .. }
            | EntityRequest::EntityField { .. }
            | EntityRequest::EntitiesAlong { .. }
            | EntityRequest::SetEntityField { .. }
            | EntityRequest::ItemEntityStack { .. }
            | EntityRequest::DamageType
            | EntityRequest::EffectLevel { .. }
            | EntityRequest::Accelerate { .. }
            | EntityRequest::Step { .. }
            | EntityRequest::AnimateWalk { .. }
            | EntityRequest::AttributeValue { .. }
            | EntityRequest::MobList
            | EntityRequest::EntitiesIn { .. }
            | EntityRequest::Accessories { .. } => CallSide::Any,
        }
    }
}

impl PlayerRequest
{
    /// one row per call, with no catch-all arm, so a new call cannot go unclassified
    pub fn side(&self) -> CallSide
    {
        match self
        {
            PlayerRequest::Give { .. }
            | PlayerRequest::SetHeldStack { .. }
            | PlayerRequest::Teleport { .. }
            | PlayerRequest::DamageHeld { .. }
            | PlayerRequest::UnlockAchievement { .. }
            | PlayerRequest::AddStat { .. }
            | PlayerRequest::SwapAccessory { .. } => CallSide::Authority,

            PlayerRequest::Players { .. }
            | PlayerRequest::Chat { .. }
            | PlayerRequest::HeldStack { .. }
            | PlayerRequest::LocalPlayer
            | PlayerRequest::IsOp { .. }
            | PlayerRequest::DigSpeed { .. }
            | PlayerRequest::SetItemInUse { .. }
            | PlayerRequest::StopUsingItem { .. }
            | PlayerRequest::HasAchievement { .. }
            | PlayerRequest::StatValue { .. } => CallSide::Any,
        }
    }
}

impl InventoryRequest
{
    /// one row per call, with no catch-all arm, so a new call cannot go unclassified
    pub fn side(&self) -> CallSide
    {
        match self
        {
            InventoryRequest::SetInventorySlot { .. }
            | InventoryRequest::SetCursorStack { .. }
            | InventoryRequest::SetTileSlot { .. }
            | InventoryRequest::InsertItem { .. }
            | InventoryRequest::ExtractItem { .. }
            | InventoryRequest::SetEntitySlot { .. }
            | InventoryRequest::ExtractMatching { .. } => CallSide::Authority,

            InventoryRequest::InventorySlot { .. }
            | InventoryRequest::CursorStack { .. }
            | InventoryRequest::TileSlot { .. }
            | InventoryRequest::OpenSlot { .. }
            | InventoryRequest::CraftResult { .. }
            | InventoryRequest::InventorySize { .. }
            | InventoryRequest::EntityInventorySize { .. }
            | InventoryRequest::EntitySlot { .. }
            | InventoryRequest::RollLoot { .. }
            | InventoryRequest::MatchMachineJob { .. }
            | InventoryRequest::MachineFuel { .. }
            | InventoryRequest::ContainedFluid { .. }
            | InventoryRequest::FillContainer { .. }
            | InventoryRequest::DrainContainer { .. }
            | InventoryRequest::FluidContainers => CallSide::Any,
        }
    }
}

impl StoredRequest
{
    /// one row per call, with no catch-all arm, so a new call cannot go unclassified
    pub fn side(&self) -> CallSide
    {
        match self
        {
            StoredRequest::SetTileData { .. }
            | StoredRequest::SetWorldData { .. }
            | StoredRequest::SetPlayerData { .. }
            | StoredRequest::SetSyncedData { .. } => CallSide::Authority,

            StoredRequest::TileData { .. }
            | StoredRequest::EntityData { .. }
            | StoredRequest::SetEntityData { .. }
            | StoredRequest::WorldData { .. }
            | StoredRequest::PlayerData { .. }
            | StoredRequest::SyncedData { .. }
            | StoredRequest::StackComponent { .. }
            | StoredRequest::SetStackComponent { .. }
            | StoredRequest::SharedData { .. }
            | StoredRequest::SetSharedData { .. }
            | StoredRequest::EntityValue { .. }
            | StoredRequest::SetEntityValue { .. }
            | StoredRequest::Declare { .. }
            | StoredRequest::Fetch { .. }
            | StoredRequest::Store { .. } => CallSide::Any,
        }
    }
}

impl SharedRequest
{
    /// one row per call, with no catch-all arm, so a new call cannot go unclassified
    pub fn side(&self) -> CallSide
    {
        match self
        {
            SharedRequest::SendEnergy { .. }
            | SharedRequest::FillFluid { .. }
            | SharedRequest::DrainFluid { .. }
            | SharedRequest::TakeEnergy { .. }
            | SharedRequest::SetEnergy { .. }
            | SharedRequest::SetTank { .. } => CallSide::Authority,

            SharedRequest::ProvideService { .. }
            | SharedRequest::CallService { .. }
            | SharedRequest::HasService { .. }
            | SharedRequest::RegisterLookup { .. }
            | SharedRequest::CallLookup { .. }
            | SharedRequest::SendInterMod { .. }
            | SharedRequest::Capabilities { .. }
            | SharedRequest::EnergyDemand { .. }
            | SharedRequest::EnergyLevel { .. }
            | SharedRequest::TankLevel { .. } => CallSide::Any,
        }
    }
}

impl NetRequest
{
    /// one row per call, with no catch-all arm, so a new call cannot go unclassified
    pub fn side(&self) -> CallSide
    {
        match self
        {
            NetRequest::RegisterChannel { .. }
            | NetRequest::Send { .. }
            | NetRequest::CanSend { .. }
            | NetRequest::RegisterConfigTask { .. } => CallSide::Any,
        }
    }
}

impl ScreenRequest
{
    /// one row per call, with no catch-all arm, so a new call cannot go unclassified
    pub fn side(&self) -> CallSide
    {
        match self
        {
            ScreenRequest::RegisterKeybind { .. }
            | ScreenRequest::RegisterScreen { .. }
            | ScreenRequest::RegisterHud { .. }
            | ScreenRequest::RegisterOverlay { .. }
            | ScreenRequest::RegisterPlayerLayer { .. }
            | ScreenRequest::TextWidth { .. }
            | ScreenRequest::OpenScreen { .. }
            | ScreenRequest::CloseScreen { .. }
            | ScreenRequest::ScreenSize
            | ScreenRequest::Clipboard
            | ScreenRequest::RegisterTooltip { .. }
            | ScreenRequest::SetDynamicTexture { .. }
            | ScreenRequest::RegisterWorldRender { .. }
            | ScreenRequest::TerrainSlot { .. }
            | ScreenRequest::TerrainTiles { .. }
            | ScreenRequest::ItemSlot { .. }
            | ScreenRequest::ScreenCraftSlot { .. }
            | ScreenRequest::PaintSprite { .. }
            | ScreenRequest::SpritePixels { .. } => CallSide::Any,
        }
    }
}

impl ViewerRequest
{
    /// one row per call, with no catch-all arm, so a new call cannot go unclassified
    pub fn side(&self) -> CallSide
    {
        match self
        {
            ViewerRequest::ItemCatalog
            | ViewerRequest::RecipeList
            | ViewerRequest::ContentGeneration
            | ViewerRequest::LootList
            | ViewerRequest::MachineList => CallSide::Any,
        }
    }
}

impl MetaRequest
{
    /// one row per call, with no catch-all arm, so a new call cannot go unclassified
    pub fn side(&self) -> CallSide
    {
        match self
        {
            MetaRequest::Subscribe { .. }
            | MetaRequest::DefineEvent { .. }
            | MetaRequest::PostEvent { .. }
            | MetaRequest::RegisterCommand { .. }
            | MetaRequest::ConfigValue { .. }
            | MetaRequest::Translate { .. }
            | MetaRequest::IsClient
            | MetaRequest::IsLoaded { .. }
            | MetaRequest::DefineConfigOptions { .. }
            | MetaRequest::SetConfigValue { .. }
            | MetaRequest::ModName { .. } => CallSide::Any,
        }
    }
}

impl MediaRequest
{
    pub fn side(&self) -> CallSide
    {
        match self
        {
            MediaRequest::Import { .. } | MediaRequest::Status { .. } | MediaRequest::InsertHeld { .. } =>
            {
                CallSide::Authority
            }
            MediaRequest::Play { .. } => CallSide::Any,
        }
    }
}
