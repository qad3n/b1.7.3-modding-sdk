use super::builders::Build;
use crate::behavior::{self, EffectBehavior};
use crate::refs::
{
    AchievementRef, AttributeRef, DamageRef, EffectRef, EnergyRef, FluidRef, GameRuleRef, ParticleRef, Ref,
};
use crate::rt::call;
use crate::{Result, dispatch, expectUnit, ownIdentifier};
use mod_protocol::Identifier;
use mod_protocol::abi::{HostCall, HostReply, LootTarget, RegistryKind};
use mod_protocol::spec::
{
    AchievementSpec, AttributeSpec, Behavior, ConditionalLootSpec, ConditionalPoolSpec, DamageTypeSpec, EffectMethod,
    EffectSpec, EnergySpec, FluidSpec, GameRuleSpec, LootSpec, ParticleSpec, TagEntry,
};
use std::rc::Rc;

pub struct Tags;

pub struct Loot;

pub struct Fluids;

pub struct Effects;

pub struct Particles;

pub struct Achievements;

pub struct Attributes;

pub struct GameRules;

pub struct Energies;

pub struct DamageTypes;

pub struct Components;

pub struct ConfigTasks;

impl ConfigTasks
{
    /// needs api 1.54, a step every joining client with this mod runs before play: send runs on the server and gives
    /// the bytes for the named player, empty to skip; receive runs on the client and refuses the join with an error
    pub fn register(
        &self,
        name: &str,
        send: impl Fn(&str) -> Vec<u8> + 'static,
        receive: impl Fn(&[u8]) -> Result<()> + 'static,
    ) -> Result<()>
    {
        let task: dispatch::ConfigTask = dispatch::ConfigTask { m_send: Rc::new(send), m_receive: Rc::new(receive) };
        let handler: u32 = dispatch::addConfigTask(task);
        expectUnit(call(HostCall::RegisterConfigTask { name: ownIdentifier(name)?.to_string(), handler }))
    }
}

pub struct Attachments;

impl Components
{
    /// needs api 1.51, an item stack component type named <modid>:<name>; synced values reach clients with every
    /// stack the server sends, so tooltips and models can read them
    pub fn register(&self, name: &str, synced: bool) -> Result<crate::refs::ComponentRef>
    {
        registered(name, |id| HostCall::RegisterComponent { name: id.to_string(), synced })
    }
}

impl Attachments
{
    /// needs api 1.51, this mod's attachments of the holder kind reach the clients that see the holder, on every
    /// change and when they first see it; world data reaches every client
    pub fn sync(&self, holder: mod_protocol::abi::AttachmentHolder) -> Result<()>
    {
        expectUnit(call(HostCall::SyncAttachments { holder }))
    }
}

impl DamageTypes
{
    /// needs api 1.48, a damage type named <modid>:<name> for Entity::damageAs; its behaviour comes from damage type
    /// tags such as minecraft:bypasses_armor or minecraft:is_fire
    pub fn register(&self, name: &str, spec: impl Build<DamageTypeSpec>) -> Result<DamageRef>
    {
        let spec: DamageTypeSpec = spec.build()?;
        registered(name, |id| HostCall::RegisterDamageType { name: id.to_string(), spec })
    }
}

impl Energies
{
    /// needs api 1.33, an energy kind named <modid>:<name> and its worth in the loader's shared base unit, so machines
    /// of other mods convert to and from it
    pub fn register(&self, name: &str, spec: impl Build<EnergySpec>) -> Result<EnergyRef>
    {
        let spec: EnergySpec = spec.build()?;
        registered(name, |id| HostCall::RegisterEnergy { name: id.to_string(), spec })
    }
}

impl GameRules
{
    /// read it with world::gameRule and change it with world::setGameRule or /gamerule under <modid>:<name>
    pub fn register(&self, name: &str, spec: impl Build<GameRuleSpec>) -> Result<GameRuleRef>
    {
        let spec: GameRuleSpec = spec.build()?;
        registered(name, |id| HostCall::RegisterGameRule { name: id.to_string(), spec })
    }
}

impl Attributes
{
    /// read it with Entity::attribute and change it with Entity::setModifier
    pub fn register(&self, name: &str, spec: impl Build<AttributeSpec>) -> Result<AttributeRef>
    {
        let spec: AttributeSpec = spec.build()?;
        registered(name, |id| HostCall::RegisterAttribute { name: id.to_string(), spec })
    }
}

impl Achievements
{
    pub fn register(&self, name: &str, spec: impl Build<AchievementSpec>) -> Result<AchievementRef>
    {
        let spec: AchievementSpec = spec.build()?;
        registered(name, |id| HostCall::RegisterAchievement { name: id.to_string(), spec })
    }
}

impl Particles
{
    pub fn register(&self, name: &str, spec: impl Build<ParticleSpec>) -> Result<ParticleRef>
    {
        let spec: ParticleSpec = spec.build()?;
        registered(name, |id| HostCall::RegisterParticle { name: id.to_string(), spec })
    }
}

impl Effects
{
    pub fn register(&self, name: &str, spec: impl Build<EffectSpec>) -> Result<EffectRef>
    {
        let spec: EffectSpec = spec.build()?;
        registered(name, |id| HostCall::RegisterEffect { name: id.to_string(), spec })
    }

    /// needs api 1.29, the effect also runs the behavior's onAdded, onTick and onRemoved
    pub fn registerWith(
        &self,
        name: &str,
        spec: impl Build<EffectSpec>,
        behavior: Rc<dyn EffectBehavior>,
    ) -> Result<EffectRef>
    {
        let spec: EffectSpec = spec.build()?;
        let methods: Vec<EffectMethod> = behavior::methods(behavior.overridden(), &behavior::EFFECT_METHODS);
        let behavior: Behavior<EffectMethod> = Behavior { handle: dispatch::addEffect(behavior), methods };
        registered(name, |id| HostCall::RegisterEffectBehavior { name: id.to_string(), spec, behavior })
    }
}

impl Fluids
{
    pub fn register(&self, name: &str, spec: impl Build<FluidSpec>) -> Result<FluidRef>
    {
        let spec: FluidSpec = spec.build()?;
        registered(name, |id| HostCall::RegisterFluid { name: id.to_string(), spec })
    }

    /// needs api 1.44, full holds amount millibuckets of fluid and drains to empty, none when it is used up; a fluid's
    /// own bucket is registered with the fluid
    pub fn container(&self, full: &str, empty: Option<&str>, fluid: &str, amount: i32) -> Result<()>
    {
        crate::containers::register(full, empty, fluid, amount)
    }
}

impl Loot
{
    /// adds drops to a block, replace drops its beta ones
    pub fn block(&self, block: &str, spec: LootSpec) -> Result<()> { setLoot(RegistryKind::Block, block, spec) }

    pub fn entity(&self, entity: &str, spec: LootSpec) -> Result<()> { setLoot(RegistryKind::Entity, entity, spec) }

    /// needs api 1.33, a chest loot table structures roll with rollLoot and recipe viewers list
    pub fn chest(&self, table: &str, spec: LootSpec) -> Result<()>
    {
        expectUnit(call(HostCall::SetChestLoot { table: ownIdentifier(table)?, spec }))
    }

    /// needs api 1.40, a weighted pool for a block, entity or chest table of any mod, each roll picking one entry;
    /// a pool naming loot conditions or count rules needs api 1.42
    pub fn pool(&self, target: LootTarget, id: &str, pool: impl Into<ConditionalPoolSpec>) -> Result<()>
    {
        let id: Identifier = ownIdentifier(id)?;
        let pool: ConditionalPoolSpec = pool.into();

        match pool.plain()
        {
            Some(plain) => expectUnit(call(HostCall::AddLootPool { target, id, pool: plain })),
            None => expectUnit(call(HostCall::AddConditionalPool { target, id, pool })),
        }
    }

    /// needs api 1.42, a loot condition any mod's pools name, label is how recipe viewers show it
    pub fn condition(&self, name: &str, label: &str, handler: impl Fn(&LootQuery) -> bool + 'static) -> Result<()>
    {
        let handler: u32 = dispatch::addLootCondition(Rc::new(handler));
        expectUnit(call(HostCall::RegisterLootCondition { name: name.to_owned(), label: label.to_owned(), handler }))
    }

    /// needs api 1.42, a rule giving the count of a drop, for pools of any mod
    pub fn count(&self, name: &str, label: &str, handler: impl Fn(&LootQuery) -> i32 + 'static) -> Result<()>
    {
        let handler: u32 = dispatch::addLootCount(Rc::new(handler));
        expectUnit(call(HostCall::RegisterLootCount { name: name.to_owned(), label: label.to_owned(), handler }))
    }
}

/// needs api 1.42, one loot roll as conditions and count rules see it
pub struct LootQuery
{
    pub world: crate::World,
    pub pos: Option<crate::BlockPos>,
    pub state: Option<crate::StateData>,
    pub entity: Option<crate::Entity>,
    pub killer: Option<crate::Entity>,
    pub tool: Option<crate::ItemStack>,
    pub random: crate::terrain::Random,
}

impl LootQuery
{
    pub(crate) fn new(context: mod_protocol::abi::LootContext) -> Self
    {
        let world: crate::World = crate::World { dimension: context.world };
        Self
        {
            world,
            pos: context.pos,
            state: context.state,
            entity: context.entity.map(|id| crate::Entity { world, id }),
            killer: context.killer.map(|id| crate::Entity { world, id }),
            tool: context.tool.map(crate::ItemStack::fromRaw),
            random: crate::terrain::Random::withSeed(context.seed),
        }
    }
}

/// needs api 1.40, a weighted loot pool built up entry by entry
pub struct PoolBuilder
{
    pub m_spec: ConditionalPoolSpec,
}

impl PoolBuilder
{
    /// rolled from rolls to most times
    pub fn new(rolls: i32, most: i32) -> Self
    {
        Self { m_spec: ConditionalPoolSpec { rolls, max_rolls: Some(most), entries: Vec::new() } }
    }

    /// an item picked with weight, from count to most of it
    pub fn entry(mut self, item: &str, weight: i32, count: i32, most: i32) -> Result<Self>
    {
        let item: Identifier = ownIdentifier(item)?;
        self.m_spec.entries.push(Self::pick(Some(item), weight, count, Some(most)));
        Ok(self)
    }

    fn pick(item: Option<Identifier>, weight: i32, count: i32, most: Option<i32>) -> ConditionalLootSpec
    {
        ConditionalLootSpec
        {
            item,
            meta: 0,
            count,
            max: most,
            weight,
            player: false,
            block_meta: None,
            when: Vec::new(),
            unless: Vec::new(),
            count_by: None,
        }
    }

    /// a pick that drops nothing
    pub fn nothing(mut self, weight: i32) -> Self
    {
        self.m_spec.entries.push(Self::pick(None, weight, 1, None));
        self
    }

    /// the entry added last only takes part when a player made the kill
    pub fn byPlayer(mut self) -> Self
    {
        if let Some(last) = self.m_spec.entries.last_mut()
        {
            last.player = true;
        }
        self
    }

    /// the entry added last only takes part for a block broken with this metadata
    pub fn blockMeta(mut self, meta: i32) -> Self
    {
        if let Some(last) = self.m_spec.entries.last_mut()
        {
            last.block_meta = Some(meta);
        }
        self
    }

    /// the entry added last drops its item with this damage value
    pub fn damage(mut self, damage: i32) -> Self
    {
        if let Some(last) = self.m_spec.entries.last_mut()
        {
            last.meta = damage;
        }
        self
    }

    /// needs api 1.42, the entry added last only takes part when the named condition passes
    pub fn when(mut self, condition: &str) -> Result<Self>
    {
        let condition: Identifier = ownIdentifier(condition)?;
        if let Some(last) = self.m_spec.entries.last_mut()
        {
            last.when.push(condition);
        }
        Ok(self)
    }

    /// needs api 1.42, the entry added last sits out when the named condition passes
    pub fn unless(mut self, condition: &str) -> Result<Self>
    {
        let condition: Identifier = ownIdentifier(condition)?;
        if let Some(last) = self.m_spec.entries.last_mut()
        {
            last.unless.push(condition);
        }
        Ok(self)
    }

    /// needs api 1.42, the entry added last drops as many as the named count rule gives
    pub fn countBy(mut self, rule: &str) -> Result<Self>
    {
        let rule: Identifier = ownIdentifier(rule)?;
        if let Some(last) = self.m_spec.entries.last_mut()
        {
            last.count_by = Some(rule);
        }
        Ok(self)
    }
}

fn setLoot(registry: RegistryKind, target: &str, spec: LootSpec) -> Result<()>
{
    expectUnit(call(HostCall::SetLoot { registry, target: ownIdentifier(target)?, spec }))
}

impl Tags
{
    pub fn add(&self, registry: RegistryKind, tag: &str, values: &[&str]) -> Result<()>
    {
        let values: Vec<Identifier> = values.iter().map(|value| ownIdentifier(value)).collect::<Result<_>>()?;
        expectUnit(call(HostCall::AddTag { registry, tag: Identifier::parse(tag)?, values }))
    }

    /// entries may name other tags as #ns:path or be optional, replace also drops the vanilla members (api 1.15)
    pub fn addEntries(&self, registry: RegistryKind, tag: &str, entries: Vec<TagEntry>, replace: bool) -> Result<()>
    {
        expectUnit(call(HostCall::AddTagEntries { registry, tag: Identifier::parse(tag)?, entries, replace }))
    }

    pub fn values(&self, registry: RegistryKind, tag: &str) -> Vec<Identifier> { tagValues(registry, tag) }

    /// needs api 1.30
    pub fn of(&self, registry: RegistryKind, id: &Identifier) -> Vec<Identifier> { tagsOf(registry, id) }
}

pub fn tagValues(registry: RegistryKind, tag: &str) -> Vec<Identifier>
{
    let Ok(tag) = Identifier::parse(tag)
    else
    {
        return Vec::new();
    };

    match call(HostCall::TagValues { registry, tag })
    {
        HostReply::Ids(ids) => ids,
        _ => Vec::new(),
    }
}

/// needs api 1.30, the tags that hold the id, resolved through nested tags
pub fn tagsOf(registry: RegistryKind, id: &Identifier) -> Vec<Identifier>
{
    match call(HostCall::TagsOf { registry, id: id.clone() })
    {
        HostReply::Ids(ids) => ids,
        _ => Vec::new(),
    }
}

/// needs api 1.48, every entry of a registry, vanilla first then mods in name order
pub fn registryList(registry: RegistryKind) -> Vec<Identifier>
{
    match call(HostCall::RegistryList { registry })
    {
        HostReply::Ids(ids) => ids,
        _ => Vec::new(),
    }
}

// sends one registration under this mod's namespace and hands back its handle
pub(crate) fn registered<K>(name: &str, send: impl FnOnce(&Identifier) -> HostCall) -> Result<Ref<K>>
{
    let id: Identifier = ownIdentifier(name)?;
    expectUnit(call(send(&id)))?;
    Ok(Ref::new(id))
}
