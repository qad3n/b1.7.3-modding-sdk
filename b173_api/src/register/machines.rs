use crate::refs::MachineRef;
use crate::rt::call;
use crate::{Error, ItemStack, Result, expectUnit, ownIdentifier};
use mod_protocol::Identifier;
use mod_protocol::abi::HostCall;
use mod_protocol::spec::{MachineRecipeSpec, MachineSlotSpec, MachineSpec};

pub struct Machines;

pub struct MachineRecipeBuilder
{
    m_machine: Result<Identifier>,
    m_spec: MachineRecipeSpec,
    m_error: Option<Error>,
}

fn slot(stack: &ItemStack, chance: f32) -> Result<MachineSlotSpec>
{
    let item: Identifier = stack.item.name().ok_or("a machine recipe stack needs a registered item")?;
    Ok(MachineSlotSpec { item: Some(item), tag: None, meta: stack.damage, count: stack.count, chance })
}

impl MachineRecipeBuilder
{
    /// a stack the machine takes, damage -1 takes any
    pub fn input(mut self, stack: ItemStack) -> Self
    {
        match slot(&stack, 1.0)
        {
            Ok(slot) => self.m_spec.inputs.push(slot),
            Err(error) => self.m_error = Some(error),
        }

        self
    }

    /// any item of a tag
    pub fn inputTag(mut self, tag: &str, count: i32) -> Self
    {
        match Identifier::parse(tag)
        {
            Ok(tag) =>
            {
                self.m_spec.inputs.push(MachineSlotSpec { item: None, tag: Some(tag), meta: -1, count, chance: 1.0 });
            }

            Err(error) => self.m_error = Some(error.into()),
        }

        self
    }

    pub fn output(self, stack: ItemStack) -> Self { self.chance(stack, 1.0) }

    /// an output the machine gives with a chance from 0 to 1
    pub fn chance(mut self, stack: ItemStack, chance: f32) -> Self
    {
        match slot(&stack, chance)
        {
            Ok(slot) => self.m_spec.outputs.push(slot),
            Err(error) => self.m_error = Some(error),
        }

        self
    }

    /// a line under the recipe such as its energy, time or heat
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

        expectUnit(call(HostCall::AddMachineRecipe { machine: self.m_machine?, recipe: self.m_spec }))
    }
}

impl Machines
{
    /// needs api 1.35, a machine named <modid>:<name> recipe viewers show as a category, run by the workstation
    /// items with the first as its icon, titled by that item's name when no title is given
    pub fn register(&self, name: &str, title: Option<&str>, workstations: &[&str]) -> Result<MachineRef>
    {
        let workstations: Vec<Identifier> =
            workstations.iter().map(|item| ownIdentifier(item)).collect::<Result<_>>()?;
        let spec: MachineSpec = MachineSpec { title: title.map(str::to_owned), workstations };
        super::content::registered(name, |id| HostCall::RegisterMachine { name: id.clone(), spec })
    }

    /// needs api 1.35, a recipe for a machine of this mod by name or of any mod by its full name
    pub fn recipe(&self, machine: &str) -> MachineRecipeBuilder
    {
        MachineRecipeBuilder { m_machine: ownIdentifier(machine), m_spec: MachineRecipeSpec::default(), m_error: None }
    }

    /// needs api 1.40, a job the machine runs and finds with matchJob, duration in ticks
    pub fn job(&self, machine: &str, duration: i32) -> crate::JobBuilder { crate::JobBuilder::new(machine, duration) }

    /// needs api 1.40, what an item is worth to a machine in its own unit (burn ticks, amplifier, quality), standing
    /// over the furnace fuels for that machine only, found with machineFuel
    pub fn fuel(&self, machine: &str, item: &str, value: i32) -> Result<()>
    {
        crate::jobs::addFuel(machine, Some(ownIdentifier(item)?), None, value)
    }

    /// needs api 1.40, a fuel value for every item of a tag
    pub fn fuelTag(&self, machine: &str, tag: &str, value: i32) -> Result<()>
    {
        crate::jobs::addFuel(machine, None, Some(ownIdentifier(tag.trim_start_matches('#'))?), value)
    }

    /// needs api 1.35, an item that runs a machine, minecraft:crafting and minecraft:smelting included
    pub fn workstation(&self, machine: &str, item: &str) -> Result<()>
    {
        expectUnit(call(HostCall::AddWorkstation { machine: ownIdentifier(machine)?, item: ownIdentifier(item)? }))
    }
}
