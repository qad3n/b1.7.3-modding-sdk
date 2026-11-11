use crate::behavior::{self, EffectBehavior};
use crate::rt::call;
use crate::{Result, dispatch, expectUnit, ownIdentifier};
use mod_protocol::Identifier;
use mod_protocol::abi::{HostCall, HostReply, LootTarget, RegistryKind};
use mod_protocol::spec::
{
    AchievementSpec, AttributeSpec, Behavior, EffectMethod, EffectSpec, EnergySpec, FluidSpec, GameRuleSpec,
    LootPoolSpec, LootSpec, ParticleSpec, TagEntry, WeightedLootSpec,
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

impl Energies
{
    /// needs api 1.33, an energy kind named <modid>:<name> and its worth in the loader's shared base unit, so machines
    /// of other mods convert to and from it
    pub fn register(&self, name: &str, spec: EnergySpec) -> Result<()>
    {
        expectUnit(call(HostCall::RegisterEnergy { name: ownIdentifier(name)?.to_string(), spec }))
    }
}

impl GameRules
{
    /// read it with world::gameRule and change it with world::setGameRule or /gamerule under <modid>:<name>
    pub fn register(&self, name: &str, spec: GameRuleSpec) -> Result<()>
    {
        expectUnit(call(HostCall::RegisterGameRule { name: name.to_owned(), spec }))
    }
}

impl Attributes
{
    /// read it with Entity::attribute and change it with Entity::setModifier under its full name <modid>:<name>
    pub fn register(&self, name: &str, spec: AttributeSpec) -> Result<()>
    {
        expectUnit(call(HostCall::RegisterAttribute { name: name.to_owned(), spec }))
    }
}

impl Achievements
{
    pub fn register(&self, name: &str, spec: AchievementSpec) -> Result<()>
    {
        expectUnit(call(HostCall::RegisterAchievement { name: name.to_owned(), spec }))
    }
}

impl Particles
{
    /// spawn it later with its full name <modid>:<name>
    pub fn register(&self, name: &str, spec: ParticleSpec) -> Result<()>
    {
        expectUnit(call(HostCall::RegisterParticle { name: name.to_owned(), spec }))
    }
}

impl Effects
{
    pub fn register(&self, name: &str, spec: EffectSpec) -> Result<()>
    {
        expectUnit(call(HostCall::RegisterEffect { name: name.to_owned(), spec }))
    }

    /// needs api 1.29, the effect also runs the behavior's onAdded, onTick and onRemoved
    pub fn registerWith(&self, name: &str, spec: EffectSpec, behavior: Rc<dyn EffectBehavior>) -> Result<()>
    {
        let methods: Vec<EffectMethod> = behavior::methods(behavior.overridden(), &behavior::EFFECT_METHODS);
        let behavior: Behavior<EffectMethod> = Behavior { handle: dispatch::addEffect(behavior), methods };
        expectUnit(call(HostCall::RegisterEffectBehavior { name: name.to_owned(), spec, behavior }))
    }
}

impl Fluids
{
    pub fn register(&self, name: &str, spec: FluidSpec) -> Result<()>
    {
        expectUnit(call(HostCall::RegisterFluid { name: name.to_owned(), spec }))
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

    /// needs api 1.40, a weighted pool for a block, entity or chest table of any mod, each roll picking one entry
    pub fn pool(&self, target: LootTarget, id: &str, pool: LootPoolSpec) -> Result<()>
    {
        expectUnit(call(HostCall::AddLootPool { target, id: ownIdentifier(id)?, pool }))
    }
}

/// needs api 1.40, a weighted loot pool built up entry by entry
pub struct PoolBuilder
{
    pub m_spec: LootPoolSpec,
}

impl PoolBuilder
{
    /// rolled from rolls to most times
    pub fn new(rolls: i32, most: i32) -> Self
    {
        Self { m_spec: LootPoolSpec { rolls, max_rolls: Some(most), entries: Vec::new() } }
    }

    /// an item picked with weight, from count to most of it
    pub fn entry(mut self, item: &str, weight: i32, count: i32, most: i32) -> Result<Self>
    {
        let item: Identifier = ownIdentifier(item)?;
        self.m_spec.entries.push(WeightedLootSpec
        {
            item: Some(item),
            meta: 0,
            count,
            max: Some(most),
            weight,
            player: false,
            block_meta: None,
        });
        Ok(self)
    }

    /// a pick that drops nothing
    pub fn nothing(mut self, weight: i32) -> Self
    {
        self.m_spec.entries.push(WeightedLootSpec
        {
            item: None,
            meta: 0,
            count: 1,
            max: None,
            weight,
            player: false,
            block_meta: None,
        });
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
