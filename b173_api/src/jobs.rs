use crate::rt::call;
use crate::{Error, ItemStack, Result, expectUnit, ownIdentifier};
use mod_protocol::Identifier;
use mod_protocol::abi::{FluidStack, HostCall, HostReply, JobMatch, StackData};
use mod_protocol::spec::
{
    JobEnergySpec, JobFluidSpec, JobInputRule, JobInputSpec, JobOutputSpec, MachineFuelSpec, MachineJobSpec,
};
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

const ANY_DAMAGE: i32 = -1;

/// needs api 1.40, a job for a machine of any mod, built up and sent with build
pub struct JobBuilder
{
    m_machine: Result<Identifier>,
    m_spec: MachineJobSpec,
    m_rules: Vec<JobInputRule>,
    m_error: Option<Error>,
}

fn input(item: Option<Identifier>, tag: Option<Identifier>, count: i32, consumed: bool, wear: i32) -> JobInputSpec
{
    JobInputSpec { item, tag, meta: ANY_DAMAGE, count, consumed, wear }
}

impl JobBuilder
{
    pub(crate) fn new(machine: &str, duration: i32) -> Self
    {
        Self
        {
            m_machine: ownIdentifier(machine),
            m_spec: MachineJobSpec { duration, ..MachineJobSpec::default() },
            m_rules: Vec::new(),
            m_error: None,
        }
    }

    fn named(&mut self, name: &str) -> Option<Identifier>
    {
        match ownIdentifier(name)
        {
            Ok(id) => Some(id),
            Err(error) =>
            {
                self.m_error = Some(error);
                None
            }
        }
    }

    /// count of an item the job uses up, any damage
    pub fn input(mut self, item: &str, count: i32) -> Self
    {
        let item: Option<Identifier> = self.named(item);
        self.m_spec.inputs.push(input(item, None, count, true, 0));
        self
    }

    /// count of any item of a tag the job uses up
    pub fn inputTag(mut self, tag: &str, count: i32) -> Self
    {
        let tag: Option<Identifier> = self.named(tag.trim_start_matches('#'));
        self.m_spec.inputs.push(input(None, tag, count, true, 0));
        self
    }

    /// needs api 1.44, count of any item the job uses up except members of the except tag
    pub fn inputAny(mut self, count: i32, except: Option<&str>) -> Self
    {
        let except: Option<Identifier> = except.and_then(|tag| self.named(tag.trim_start_matches('#')));
        self.m_rules.push(JobInputRule { input: self.m_spec.inputs.len() as u32, except });
        self.m_spec.inputs.push(input(None, None, count, true, 0));
        self
    }

    /// an item the job needs but leaves in place, taking wear damage each job
    pub fn keep(mut self, item: &str, wear: i32) -> Self
    {
        let item: Option<Identifier> = self.named(item);
        self.m_spec.inputs.push(input(item, None, 1, false, wear));
        self
    }

    /// only stacks of this damage fill the input added last
    pub fn damage(mut self, damage: i32) -> Self
    {
        if let Some(last) = self.m_spec.inputs.last_mut()
        {
            last.meta = damage;
        }
        self
    }

    pub fn output(self, item: &str, count: i32) -> Self { self.chance(item, count, 1.0) }

    /// an output given with a chance from 0 to 1
    pub fn chance(mut self, item: &str, count: i32, chance: f32) -> Self
    {
        let item: Option<Identifier> = self.named(item);
        self.m_spec.outputs.push(JobOutputSpec { item, meta: 0, count, chance, from_input: None, damage: None });
        self
    }

    /// hands back the stack that filled an input, its damage set when given, so a repair returns the same item
    pub fn returns(mut self, input: u32, damage: Option<i32>) -> Self
    {
        self.m_spec.outputs.push(JobOutputSpec
        {
            item: None,
            meta: 0,
            count: 1,
            chance: 1.0,
            from_input: Some(input),
            damage,
        });
        self
    }

    /// the job runs as long as the machine fuel value of the stack filling its first input, furnace burn time when
    /// the machine has no entry of its own, and the duration given to job is ignored
    pub fn timedByFuel(mut self) -> Self
    {
        self.m_spec.duration_from_fuel = true;
        self
    }

    pub fn fluidInput(mut self, fluid: &str, amount: i32) -> Self
    {
        let fluid: Option<Identifier> = self.named(fluid);
        self.m_spec.fluid_inputs.push(JobFluidSpec { fluid, tag: None, amount });
        self
    }

    /// millibuckets of any fluid of a fluid tag, such as c:oil
    pub fn fluidInputTag(mut self, tag: &str, amount: i32) -> Self
    {
        let tag: Option<Identifier> = self.named(tag.trim_start_matches('#'));
        self.m_spec.fluid_inputs.push(JobFluidSpec { fluid: None, tag, amount });
        self
    }

    pub fn fluidOutput(mut self, fluid: &str, amount: i32) -> Self
    {
        let fluid: Option<Identifier> = self.named(fluid);
        self.m_spec.fluid_outputs.push(JobFluidSpec { fluid, tag: None, amount });
        self
    }

    fn power(&mut self, kind: &str, amount: i64, perTick: bool) -> Option<JobEnergySpec>
    {
        Some(JobEnergySpec { kind: self.named(kind)?, amount, per_tick: perTick })
    }

    /// energy of a kind the job draws, each tick or in total
    pub fn energyIn(mut self, kind: &str, amount: i64, perTick: bool) -> Self
    {
        self.m_spec.energy_in = self.power(kind, amount, perTick);
        self
    }

    /// energy of a kind the job gives, as an engine or generator does
    pub fn energyOut(mut self, kind: &str, amount: i64, perTick: bool) -> Self
    {
        self.m_spec.energy_out = self.power(kind, amount, perTick);
        self
    }

    /// a line recipe viewers show under the job
    pub fn note(mut self, text: &str) -> Self
    {
        self.m_spec.notes.push(text.to_owned());
        self
    }

    pub fn build(self) -> Result<()>
    {
        if let Some(error) = self.m_error
        {
            return Err(error);
        }

        let machine: Identifier = self.m_machine?;
        if self.m_rules.is_empty()
        {
            return expectUnit(call(HostCall::AddMachineJob { machine, job: self.m_spec }));
        }

        expectUnit(call(HostCall::AddMachineJobRules { machine, job: self.m_spec, rules: self.m_rules }))
    }
}

pub(crate) fn addFuel(machine: &str, item: Option<Identifier>, tag: Option<Identifier>, value: i32) -> Result<()>
{
    let fuel: MachineFuelSpec = MachineFuelSpec { item, tag, meta: ANY_DAMAGE, value };
    expectUnit(call(HostCall::AddMachineFuel { machine: ownIdentifier(machine)?, fuel }))
}

// the host's answers held per epoch, since they carry numeric ids, so machines asking every tick stay cheap
struct JobEntry
{
    m_machine: String,
    m_inputs: Vec<Option<StackData>>,
    m_fluids: Vec<FluidStack>,
    m_job: Option<Rc<JobMatch>>,
}

struct FuelEntry
{
    m_machine: String,
    m_stack: StackData,
    m_furnace: bool,
    m_value: i32,
}

#[derive(Default)]
struct Caches
{
    m_epoch: Option<u32>,
    m_jobs: HashMap<u64, JobEntry>,
    m_fuels: HashMap<u64, FuelEntry>,
}

// a full cache starts over, counts in the keys make many entries as stacks shrink
const CACHE_LIMIT: usize = 4096;

thread_local!
{
    static CACHES: RefCell<Caches> = RefCell::new(Caches::default());
}

fn withCaches<R>(read: impl FnOnce(&mut Caches) -> R) -> R
{
    CACHES.with(|caches|
    {
        let mut caches: RefMut<Caches> = caches.borrow_mut();
        let epoch: u32 = crate::epoch();
        if caches.m_epoch != Some(epoch)
        {
            *caches = Caches { m_epoch: Some(epoch), ..Caches::default() };
        }

        read(&mut caches)
    })
}

fn raws(inputs: &[Option<ItemStack>]) -> Vec<Option<StackData>>
{
    inputs.iter().map(|stack| stack.as_ref().and_then(|stack| stack.raw().ok())).collect()
}

// hashed without allocating, the entry still compares the whole question since keys may collide
fn jobKey(machine: &str, inputs: &[Option<ItemStack>], fluids: &[FluidStack]) -> u64
{
    let mut hasher: DefaultHasher = DefaultHasher::new();
    machine.hash(&mut hasher);

    for input in inputs
    {
        input.as_ref().and_then(|stack| stack.raw().ok()).map(|raw| (raw.id, raw.count, raw.damage)).hash(&mut hasher);
    }

    for fluid in fluids
    {
        (&fluid.fluid, fluid.amount).hash(&mut hasher);
    }

    hasher.finish()
}

impl JobEntry
{
    fn answers(&self, machine: &str, inputs: &[Option<ItemStack>], fluids: &[FluidStack]) -> bool
    {
        let same: bool = self.m_inputs.len() == inputs.len()
            && self
                .m_inputs
                .iter()
                .zip(inputs)
                .all(|(held, stack)| *held == stack.as_ref().and_then(|stack| stack.raw().ok()));
        same && self.m_machine == machine && self.m_fluids == fluids
    }
}

/// needs api 1.40, the first job of a machine the stacks and fluids can run, each job input taking the first stack in
/// order an earlier input left free; the match's slots say which stack each input took. Cached until the next remap,
/// so a machine may ask every tick
pub fn findJob(machine: &str, inputs: &[Option<ItemStack>], fluids: &[FluidStack]) -> Option<Rc<JobMatch>>
{
    let key: u64 = jobKey(machine, inputs, fluids);
    let cached: Option<Option<Rc<JobMatch>>> = withCaches(|caches|
    {
        caches.m_jobs.get(&key).filter(|entry| entry.answers(machine, inputs, fluids)).map(|entry| entry.m_job.clone())
    });
    if let Some(job) = cached
    {
        return job;
    }

    let id: Identifier = ownIdentifier(machine).ok()?;
    let held: Vec<Option<StackData>> = raws(inputs);
    let job: Option<Rc<JobMatch>> =
        match call(HostCall::MatchMachineJob { machine: id, inputs: held.clone(), fluids: fluids.to_vec() })
        {
            HostReply::Job(found) => found.map(Rc::new),
            _ => None,
        };
    let entry: JobEntry =
        JobEntry { m_machine: machine.to_owned(), m_inputs: held, m_fluids: fluids.to_vec(), m_job: job.clone() };
    withCaches(|caches|
    {
        if caches.m_jobs.len() >= CACHE_LIMIT
        {
            caches.m_jobs.clear();
        }

        caches.m_jobs.insert(key, entry);
    });
    job
}

/// needs api 1.40, findJob as an owned match
pub fn matchJob(machine: &str, inputs: &[Option<ItemStack>], fluids: &[FluidStack]) -> Option<JobMatch>
{
    findJob(machine, inputs, fluids).map(|job| (*job).clone())
}

fn fuelKey(machine: &str, stack: &StackData, furnace: bool) -> u64
{
    let mut hasher: DefaultHasher = DefaultHasher::new();
    (machine, stack.id, stack.damage, furnace).hash(&mut hasher);
    hasher.finish()
}

/// needs api 1.40, a stack's fuel value for a machine in the unit the machine defines, from its own entries, or
/// with furnace the furnace burn time when it has none, 0 when it is no fuel there; cached until the next remap
pub fn machineFuel(machine: &str, stack: &ItemStack, furnace: bool) -> i32
{
    let Ok(raw) = stack.raw()
    else
    {
        return 0;
    };

    let key: u64 = fuelKey(machine, &raw, furnace);
    let cached: Option<i32> = withCaches(|caches|
    {
        caches
            .m_fuels
            .get(&key)
            .filter(|entry|
            {
                entry.m_machine == machine
                    && entry.m_stack.id == raw.id
                    && entry.m_stack.damage == raw.damage
                    && entry.m_furnace == furnace
            })
            .map(|entry| entry.m_value)
    });
    if let Some(value) = cached
    {
        return value;
    }

    let Ok(id) = ownIdentifier(machine)
    else
    {
        return 0;
    };

    let value: i32 = match call(HostCall::MachineFuel { machine: id, stack: raw.clone(), furnace })
    {
        HostReply::Int(ticks) => ticks,
        _ => 0,
    };
    let entry: FuelEntry =
        FuelEntry { m_machine: machine.to_owned(), m_stack: raw, m_furnace: furnace, m_value: value };
    withCaches(|caches|
    {
        if caches.m_fuels.len() >= CACHE_LIMIT
        {
            caches.m_fuels.clear();
        }

        caches.m_fuels.insert(key, entry);
    });
    value
}
