use crate::{Result, modId};
use mod_protocol::Identifier;
use mod_protocol::identifier::MINECRAFT;
use std::fmt;
use std::marker::PhantomData;

/// the registry kinds a Ref can point into, only used as type markers
pub mod kind
{
    pub struct Effect;
    pub struct Fluid;
    pub struct Particle;
    pub struct Sound;
    pub struct Attribute;
    pub struct GameRule;
    pub struct Energy;
    pub struct Machine;
    pub struct AccessorySlot;
    pub struct Feature;
    pub struct Structure;
    pub struct LootRule;
    pub struct DamageType;
    pub struct Achievement;
    pub struct Entity;
    pub struct Tile;
    pub struct Screen;
    pub struct Registry;
    pub struct Component;
}

/// a registered entry by name, returned by every register call and taken wherever a name is asked for; it holds
/// no numeric id, so it stays valid across worlds, servers and remaps
pub struct Ref<K>
{
    m_id: Identifier,
    m_kind: PhantomData<K>,
}

pub type EffectRef = Ref<kind::Effect>;
pub type FluidRef = Ref<kind::Fluid>;
pub type ParticleRef = Ref<kind::Particle>;
pub type SoundRef = Ref<kind::Sound>;
pub type AttributeRef = Ref<kind::Attribute>;
pub type GameRuleRef = Ref<kind::GameRule>;
pub type EnergyRef = Ref<kind::Energy>;
pub type MachineRef = Ref<kind::Machine>;
pub type SlotRef = Ref<kind::AccessorySlot>;
pub type FeatureRef = Ref<kind::Feature>;
pub type LootRuleRef = Ref<kind::LootRule>;
pub type DamageRef = Ref<kind::DamageType>;
pub type AchievementRef = Ref<kind::Achievement>;
pub type EntityRef = Ref<kind::Entity>;
pub type TileRef = Ref<kind::Tile>;
pub type ScreenRef = Ref<kind::Screen>;
pub type ComponentRef = Ref<kind::Component>;

impl<K> Ref<K>
{
    pub fn new(id: Identifier) -> Self { Ref { m_id: id, m_kind: PhantomData } }

    /// a name of another mod or of vanilla, a bare path is read in this mod's namespace
    pub fn named(name: &str) -> Result<Self> { Ok(Self::new(Identifier::parseIn(name, &modId())?)) }

    pub fn id(&self) -> &Identifier { &self.m_id }

    pub fn is(&self, name: &str) -> bool { Identifier::parseIn(name, &modId()).is_ok_and(|id| id == self.m_id) }
}

impl<K> Clone for Ref<K>
{
    fn clone(&self) -> Self { Self::new(self.m_id.clone()) }
}

impl<K> PartialEq for Ref<K>
{
    fn eq(&self, other: &Self) -> bool { self.m_id == other.m_id }
}

impl<K> Eq for Ref<K> {}

impl<K> std::hash::Hash for Ref<K>
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.m_id.hash(state); }
}

impl<K> fmt::Debug for Ref<K>
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result { write!(formatter, "Ref({})", self.m_id) }
}

impl<K> fmt::Display for Ref<K>
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result { self.m_id.fmt(formatter) }
}

impl<K> From<Ref<K>> for Identifier
{
    fn from(value: Ref<K>) -> Identifier { value.m_id }
}

/// anything that names a registry entry: a Ref, an Identifier, or a name; a bare name without a namespace means
/// this mod, except where a call says it means vanilla
pub trait IntoName: Sized
{
    fn nameIn(self, namespace: &str) -> Result<Identifier>;

    fn intoName(self) -> Result<Identifier> { self.nameIn(&modId()) }

    /// a bare name means minecraft, for calls that always took vanilla names such as minecraft:fall
    fn vanillaName(self) -> Result<Identifier> { self.nameIn(MINECRAFT) }
}

impl IntoName for &str
{
    fn nameIn(self, namespace: &str) -> Result<Identifier> { Ok(Identifier::parseIn(self, namespace)?) }
}

impl IntoName for &String
{
    fn nameIn(self, namespace: &str) -> Result<Identifier> { self.as_str().nameIn(namespace) }
}

impl IntoName for String
{
    fn nameIn(self, namespace: &str) -> Result<Identifier> { self.as_str().nameIn(namespace) }
}

impl IntoName for Identifier
{
    fn nameIn(self, _namespace: &str) -> Result<Identifier> { Ok(self) }
}

impl IntoName for &Identifier
{
    fn nameIn(self, _namespace: &str) -> Result<Identifier> { Ok(self.clone()) }
}

impl<K> IntoName for Ref<K>
{
    fn nameIn(self, _namespace: &str) -> Result<Identifier> { Ok(self.m_id) }
}

impl IntoName for &crate::ItemRef
{
    fn nameIn(self, _namespace: &str) -> Result<Identifier>
    {
        self.name().ok_or_else(|| crate::Error::from("the item has no registry name"))
    }
}

impl<K> IntoName for &Ref<K>
{
    fn nameIn(self, _namespace: &str) -> Result<Identifier> { Ok(self.m_id.clone()) }
}
