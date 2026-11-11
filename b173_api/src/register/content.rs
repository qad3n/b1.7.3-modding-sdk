use crate::rt::call;
use crate::{Result, expectUnit, ownIdentifier};
use mod_protocol::Identifier;
use mod_protocol::abi::{HostCall, HostReply, RegistryKind};
use mod_protocol::spec::{AchievementSpec, AttributeSpec, EffectSpec, FluidSpec, LootSpec, ParticleSpec, TagEntry};

pub struct Tags;

pub struct Loot;

pub struct Fluids;

pub struct Effects;

pub struct Particles;

pub struct Achievements;

pub struct Attributes;

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
