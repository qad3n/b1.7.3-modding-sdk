use crate::rt::call;
use crate::{Error, ItemStack, Result, expectUnit, ownIdentifier};
use mod_protocol::Identifier;
use mod_protocol::abi::{FluidStack, HostCall, HostReply, JobMatch, StackData};
use mod_protocol::spec::{JobEnergySpec, JobFluidSpec, JobInputSpec, JobOutputSpec, MachineFuelSpec, MachineJobSpec};

const ANY_DAMAGE: i32 = -1;

/// needs api 1.40, a job for a machine of any mod, built up and sent with build
pub struct JobBuilder
{
    m_machine: Result<Identifier>,
    m_spec: MachineJobSpec,
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

        expectUnit(call(HostCall::AddMachineJob { machine: self.m_machine?, job: self.m_spec }))
    }
}

pub(crate) fn addFuel(machine: &str, item: Option<Identifier>, tag: Option<Identifier>, value: i32) -> Result<()>
{
    let fuel: MachineFuelSpec = MachineFuelSpec { item, tag, meta: ANY_DAMAGE, value };
    expectUnit(call(HostCall::AddMachineFuel { machine: ownIdentifier(machine)?, fuel }))
}

/// needs api 1.40, the first job of a machine the stacks and fluids can run, each job input taking the first stack in
/// order an earlier input left free; the match's slots say which stack each input took
pub fn matchJob(machine: &str, inputs: &[Option<ItemStack>], fluids: &[FluidStack]) -> Option<JobMatch>
{
    let machine: Identifier = ownIdentifier(machine).ok()?;
    let inputs: Vec<Option<StackData>> =
        inputs.iter().map(|stack| stack.as_ref().and_then(|stack| stack.raw().ok())).collect();
    match call(HostCall::MatchMachineJob { machine, inputs, fluids: fluids.to_vec() })
    {
        HostReply::Job(found) => found,
        _ => None,
    }
}

/// needs api 1.40, a stack's fuel value for a machine in the unit the machine defines, from its own entries, or
/// with furnace the furnace burn time when it has none, 0 when it is no fuel there
pub fn machineFuel(machine: &str, stack: &ItemStack, furnace: bool) -> i32
{
    let (Ok(machine), Ok(stack)) = (ownIdentifier(machine), stack.raw())
    else
    {
        return 0;
    };

    match call(HostCall::MachineFuel { machine, stack, furnace })
    {
        HostReply::Int(ticks) => ticks,
        _ => 0,
    }
}
