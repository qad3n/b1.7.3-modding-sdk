use crate::behavior::{self, TileBehavior};
use crate::refs::IntoName;
use crate::{Error, Result, dispatch};
use mod_protocol::Identifier;
use mod_protocol::abi::{GameRuleValue, ModifierOperation};
use mod_protocol::spec::
{
    AchievementSpec, AttributeSpec, Behavior, DamageTypeSpec, EffectModifierSpec, EffectSpec, EnergyBufferSpec,
    EnergySpec, EntityModelSpec, FaceTextures, FluidSpec, FluidStyle, GameRuleCategory, GameRuleSpec, ParticleSpec,
    PropertySpec, StatFormat, StatSpec, StateVariantSpec, TankSpec, TileEntitySpec, TileMethod,
};
use std::collections::BTreeMap;
use std::rc::Rc;

/// what register calls take: a builder, or the plain spec it builds
pub trait Build<S>
{
    fn build(self) -> Result<S>;
}

impl<S> Build<S> for S
{
    fn build(self) -> Result<S> { Ok(self) }
}

// a builder keeps the first bad name it was given and register reports it
fn note<T>(problem: &mut Option<Error>, value: Result<T>) -> Option<T>
{
    match value
    {
        Ok(value) => Some(value),
        Err(error) =>
        {
            problem.get_or_insert(error);
            None
        }
    }
}

fn finish<S>(problem: Option<Error>, spec: S) -> Result<S>
{
    match problem
    {
        Some(error) => Err(error),
        None => Ok(spec),
    }
}

macro_rules! build
{
    ($builder:ident => $spec:ident) =>
    {
        impl Build<$spec> for $builder
        {
            fn build(self) -> Result<$spec> { finish(self.m_problem, self.m_spec) }
        }
    };
}

/// a status effect: its colour, whether it helps, attribute changes per level and an optional heal or harm
pub struct Effect
{
    m_spec: EffectSpec,
    m_problem: Option<Error>,
}

impl Effect
{
    pub fn new(color: u32) -> Self { Effect { m_spec: EffectSpec { color, ..EffectSpec::default() }, m_problem: None } }

    pub fn beneficial(mut self) -> Self
    {
        self.m_spec.beneficial = true;
        self
    }

    /// the texture the hud draws for it, a colour square without one
    pub fn icon(mut self, texture: &str) -> Self
    {
        self.m_spec.icon = Some(texture.to_owned());
        self
    }

    /// each level adds the amount once more
    pub fn modifier(mut self, attribute: impl IntoName, amount: f64, operation: ModifierOperation) -> Self
    {
        if let Some(attribute) = note(&mut self.m_problem, attribute.intoName())
        {
            self.m_spec.modifiers.push(EffectModifierSpec { attribute, amount, operation });
        }

        self
    }

    /// every interval ticks heals a positive amount or hurts a negative one
    pub fn every(mut self, interval: i32, amount: i32) -> Self
    {
        self.m_spec.interval = interval;
        self.m_spec.amount = amount;
        self
    }
}

build!(Effect => EffectSpec);

/// a mod fluid: its flowing and still blocks, bucket and flow rates
pub struct Fluid
{
    m_spec: FluidSpec,
    m_problem: Option<Error>,
}

impl Fluid
{
    pub fn new(style: FluidStyle, flowing: impl IntoName, still: impl IntoName) -> Self
    {
        let mut problem: Option<Error> = None;
        let air: Identifier = Identifier::minecraft("air");
        let flowing: Identifier = note(&mut problem, flowing.intoName()).unwrap_or_else(|| air.clone());
        let still: Identifier = note(&mut problem, still.intoName()).unwrap_or(air);
        let spec: FluidSpec = FluidSpec
        {
            style,
            flowing,
            still,
            bucket: None,
            tick_rate: None,
            decay: None,
            nether_decay: None,
            infinite: None,
        };
        Fluid { m_spec: spec, m_problem: problem }
    }

    pub fn bucket(mut self, item: impl IntoName) -> Self
    {
        self.m_spec.bucket = note(&mut self.m_problem, item.intoName());
        self
    }

    setters!
    {
        tickRate(i32) => tick_rate;
        decay(i32) => decay;
        netherDecay(i32) => nether_decay;
        infinite(bool) => infinite;
    }
}

build!(Fluid => FluidSpec);

/// a mod particle: a beta particle as base with the parts it changes
pub struct Particle
{
    m_spec: ParticleSpec,
    m_problem: Option<Error>,
}

impl Particle
{
    pub fn new(base: &str) -> Self
    {
        let spec: ParticleSpec =
            ParticleSpec { base: base.to_owned(), color: None, scale: None, lifetime: None, gravity: None };
        Particle { m_spec: spec, m_problem: None }
    }

    setters!
    {
        color(u32) => color;
        scale(f32) => scale;
        lifetime(i32) => lifetime;
        gravity(f32) => gravity;
    }
}

build!(Particle => ParticleSpec);

/// a mod achievement: its lang keys, icon item and the achievement it follows
pub struct Achievement
{
    m_spec: AchievementSpec,
    m_problem: Option<Error>,
}

impl Achievement
{
    pub fn new(title: &str, description: &str, icon: impl IntoName) -> Self
    {
        let mut problem: Option<Error> = None;
        let icon: Identifier = note(&mut problem, icon.intoName()).unwrap_or_else(|| Identifier::minecraft("air"));
        let spec: AchievementSpec =
            AchievementSpec { title: title.to_owned(), description: description.to_owned(), icon, parent: None };
        Achievement { m_spec: spec, m_problem: problem }
    }

    pub fn after(mut self, parent: impl IntoName) -> Self
    {
        self.m_spec.parent = note(&mut self.m_problem, parent.intoName());
        self
    }
}

build!(Achievement => AchievementSpec);

/// a mod stat: its lang key or title and how the stats screen prints its value
pub struct Stat
{
    m_spec: StatSpec,
    m_problem: Option<Error>,
}

impl Stat
{
    pub fn new(title: &str) -> Self
    {
        Stat { m_spec: StatSpec { title: title.to_owned(), format: StatFormat::Number }, m_problem: None }
    }

    /// the value counts ticks, shown as beta shows time played
    pub fn time(mut self) -> Self
    {
        self.m_spec.format = StatFormat::Time;
        self
    }

    /// the value counts centimetres, shown as beta shows distance walked
    pub fn distance(mut self) -> Self
    {
        self.m_spec.format = StatFormat::Distance;
        self
    }
}

build!(Stat => StatSpec);

/// a mod attribute living entities carry, with its default and range
pub struct Attribute
{
    m_spec: AttributeSpec,
    m_problem: Option<Error>,
}

impl Attribute
{
    pub fn new(default: f64) -> Self
    {
        Attribute { m_spec: AttributeSpec { default, min: f64::MIN, max: f64::MAX }, m_problem: None }
    }

    pub fn range(mut self, min: f64, max: f64) -> Self
    {
        self.m_spec.min = min;
        self.m_spec.max = max;
        self
    }
}

build!(Attribute => AttributeSpec);

/// a game rule under /gamerule <modid>:<name>
pub struct GameRule
{
    m_spec: GameRuleSpec,
    m_problem: Option<Error>,
}

impl GameRule
{
    pub fn flag(default: bool) -> Self { Self::with(GameRuleValue::Bool(default)) }

    pub fn number(default: i32) -> Self { Self::with(GameRuleValue::Int(default)) }

    fn with(default: GameRuleValue) -> Self
    {
        let spec: GameRuleSpec =
            GameRuleSpec { default, category: GameRuleCategory::default(), min: i32::MIN, max: i32::MAX };
        GameRule { m_spec: spec, m_problem: None }
    }

    pub fn category(mut self, category: GameRuleCategory) -> Self
    {
        self.m_spec.category = category;
        self
    }

    pub fn range(mut self, min: i32, max: i32) -> Self
    {
        self.m_spec.min = min;
        self.m_spec.max = max;
        self
    }
}

build!(GameRule => GameRuleSpec);

/// an energy kind with its worth in the loader's shared base unit, so machines of other mods convert it
pub struct Energy
{
    m_spec: EnergySpec,
    m_problem: Option<Error>,
}

impl Energy
{
    pub fn new(label: &str, worth: i64) -> Self
    {
        Energy { m_spec: EnergySpec { worth, label: label.to_owned() }, m_problem: None }
    }
}

build!(Energy => EnergySpec);

/// a damage type for Entity::damageAs, its behaviour comes from damage type tags
pub struct DamageType
{
    m_spec: DamageTypeSpec,
    m_problem: Option<Error>,
}

impl DamageType
{
    pub fn new() -> Self { DamageType { m_spec: DamageTypeSpec::default(), m_problem: None } }

    /// a lang key mods may show for hits of this type
    pub fn message(mut self, key: &str) -> Self
    {
        self.m_spec.message = key.to_owned();
        self
    }
}

impl Default for DamageType
{
    fn default() -> Self { Self::new() }
}

build!(DamageType => DamageTypeSpec);

/// a block property such as powered or facing, its first value is the default
pub struct Property
{
    pub(crate) m_spec: PropertySpec,
}

impl Property
{
    pub fn bool(name: &str) -> Self { Self::values(name, &["false", "true"]) }

    pub fn values(name: &str, values: &[&str]) -> Self
    {
        Property
        {
            m_spec: PropertySpec
            {
                name: name.to_owned(),
                values: values.iter().map(|value| (*value).to_owned()).collect(),
            },
        }
    }

    /// whole numbers from low to high, as crop age
    pub fn range(name: &str, low: i32, high: i32) -> Self
    {
        let values: Vec<String> = (low..=high).map(|value| value.to_string()).collect();
        Property { m_spec: PropertySpec { name: name.to_owned(), values } }
    }
}

/// what a block looks like in the states that match every pair
pub struct Variant
{
    pub(crate) m_spec: StateVariantSpec,
}

impl Variant
{
    pub fn when(pairs: &[(&str, &str)]) -> Self
    {
        let when: BTreeMap<String, String> =
            pairs.iter().map(|(property, value)| ((*property).to_owned(), (*value).to_owned())).collect();
        Variant { m_spec: StateVariantSpec { when, texture: None, textures: None } }
    }

    pub fn texture(mut self, texture: &str) -> Self
    {
        self.m_spec.texture = Some(texture.to_owned());
        self
    }

    pub fn faces(mut self, faces: FaceTextures) -> Self
    {
        self.m_spec.textures = Some(faces);
        self
    }
}

/// one fluid at a time, up to a capacity in thousandths of a bucket
pub struct Tank
{
    pub(crate) m_spec: TankSpec,
}

impl Tank
{
    pub fn new(capacity: i32) -> Self { Tank { m_spec: TankSpec { capacity, ..TankSpec::default() } } }

    /// the fluids the tank takes by name, any fluid when not called
    pub fn accepts(mut self, fluids: &[&str]) -> Self
    {
        self.m_spec.accepts = fluids.iter().map(|fluid| (*fluid).to_owned()).collect();
        self
    }

    /// one bit per block face in face order, set where fluid may come in; every face when not called
    pub fn inputFaces(mut self, faces: u8) -> Self
    {
        self.m_spec.input_faces = faces;
        self
    }

    /// one bit per block face in face order, set where fluid may go out; every face when not called
    pub fn outputFaces(mut self, faces: u8) -> Self
    {
        self.m_spec.output_faces = faces;
        self
    }
}

/// how much of one energy kind a tile holds and how fast it moves in and out
pub struct EnergyBuffer
{
    pub(crate) m_spec: EnergyBufferSpec,
}

impl EnergyBuffer
{
    /// the kind is one this or another mod registered, a bare name is taken in this mod's namespace
    pub fn new(kind: &str, capacity: i64) -> Self
    {
        EnergyBuffer { m_spec: EnergyBufferSpec { kind: kind.to_owned(), capacity, ..EnergyBufferSpec::default() } }
    }

    /// the most one transfer may put in from outside
    pub fn input(mut self, rate: i64) -> Self
    {
        self.m_spec.input = rate;
        self
    }

    /// the most one transfer may take out from outside
    pub fn output(mut self, rate: i64) -> Self
    {
        self.m_spec.output = rate;
        self
    }

    /// one bit per block face in face order, set where energy may come in; every face when not called
    pub fn inputFaces(mut self, faces: u8) -> Self
    {
        self.m_spec.input_faces = faces;
        self
    }

    /// one bit per block face in face order, set where energy may go out; every face when not called
    pub fn outputFaces(mut self, faces: u8) -> Self
    {
        self.m_spec.output_faces = faces;
        self
    }
}

/// a tile entity declared with its block: inventory, ticking, client sync, behavior and model
pub struct Tile
{
    pub(crate) m_spec: TileEntitySpec,
    pub(crate) m_behavior: Option<Rc<dyn TileBehavior>>,
    pub(crate) m_model: Option<EntityModelSpec>,
}

impl Tile
{
    pub fn new(behavior: impl TileBehavior + 'static) -> Self
    {
        Tile { m_spec: TileEntitySpec::default(), m_behavior: Some(Rc::new(behavior)), m_model: None }
    }

    /// a tile with no code of its own, only slots or synced data
    pub fn plain() -> Self { Tile { m_spec: TileEntitySpec::default(), m_behavior: None, m_model: None } }

    pub fn inventory(mut self, slots: i32) -> Self
    {
        self.m_spec.inventory = slots;
        self
    }

    pub fn ticks(mut self) -> Self
    {
        self.m_spec.ticks = true;
        self
    }

    pub fn synced(mut self) -> Self
    {
        self.m_spec.sync = true;
        self
    }

    pub fn model(mut self, model: EntityModelSpec) -> Self
    {
        self.m_model = Some(model);
        self
    }

    /// a fluid tank the loader keeps for the tile: it is saved, sent to clients and open to other mods' pipes with no
    /// code here; read and change it with World::tankLevel, fillFluid, drainFluid and setTank
    pub fn tank(mut self, tank: Tank) -> Self
    {
        self.m_spec.tank = Some(tank.m_spec);
        self
    }

    /// an energy buffer the loader keeps for the tile: it is saved, sent to clients and open to other mods' machines
    /// with no code here; read and change it with World::energyLevel, takeEnergy, sendEnergy and setEnergy
    pub fn energy(mut self, buffer: EnergyBuffer) -> Self
    {
        self.m_spec.energy = Some(buffer.m_spec);
        self
    }

    // the spec the loader takes, the behavior handed to the dispatcher once
    pub(crate) fn spec(&self) -> TileEntitySpec
    {
        let behavior: Option<Behavior<TileMethod>> = self.m_behavior.as_ref().map(|behavior|
        {
            let methods: Vec<TileMethod> = behavior::methods(behavior.overridden(), &behavior::TILE_METHODS);
            Behavior { handle: dispatch::addTile(Rc::clone(behavior)), methods }
        });
        TileEntitySpec { behavior, ..self.m_spec.clone() }
    }
}
