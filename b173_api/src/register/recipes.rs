use crate::rt::call;
use crate::{Error, ItemRef, ItemStack, Result, expectUnit, ownIdentifier};
use mod_protocol::Identifier;
use mod_protocol::abi::HostCall;
use mod_protocol::spec::{FuelSpec, RecipeSpec, ShapedSpec, ShapelessSpec, SmeltingSpec, StackSpec};
use std::collections::BTreeMap;

fn stackSpec(stack: &ItemStack) -> Result<StackSpec>
{
    let item: Identifier = stack.item.name().ok_or("a recipe stack needs a registered item")?;
    Ok(StackSpec { item, count: stack.count, meta: stack.damage })
}

pub struct Recipes;

pub struct ShapedBuilder
{
    m_name: String,
    m_result: ItemStack,
    m_pattern: Vec<String>,
    m_key: BTreeMap<String, StackSpec>,
    m_error: Option<Error>,
}

impl ShapedBuilder
{
    pub fn pattern<const N: usize>(self, rows: [&str; N]) -> Self { self.rows(&rows) }

    /// the pattern as a slice, for recipe tables whose rows differ in count
    pub fn rows(mut self, rows: &[&str]) -> Self
    {
        self.m_pattern = rows.iter().map(|row| (*row).to_owned()).collect();
        self
    }

    pub fn key(self, symbol: char, item: impl Into<ItemRef>) -> Self { self.keyStack(symbol, ItemStack::of(item, 1)) }

    /// a key that needs one metadata, or any metadata with damage -1
    pub fn keyStack(mut self, symbol: char, stack: ItemStack) -> Self
    {
        match stackSpec(&stack)
        {
            Ok(stack) =>
            {
                self.m_key.insert(symbol.to_string(), stack);
            }

            Err(error) => self.m_error = Some(error),
        }

        self
    }

    pub fn build(self) -> Result<()>
    {
        if let Some(error) = self.m_error
        {
            return Err(error);
        }

        let spec: RecipeSpec = RecipeSpec::Shaped(ShapedSpec
        {
            pattern: self.m_pattern,
            key: self.m_key,
            result: stackSpec(&self.m_result)?,
        });
        expectUnit(call(HostCall::RegisterRecipe { name: self.m_name, spec }))
    }
}

impl Recipes
{
    pub fn shaped(&self, name: &str, result: ItemStack) -> ShapedBuilder
    {
        ShapedBuilder
        {
            m_name: name.to_owned(),
            m_result: result,
            m_pattern: Vec::new(),
            m_key: BTreeMap::new(),
            m_error: None,
        }
    }

    pub fn shapeless(&self, name: &str, result: ItemStack, ingredients: &[ItemStack]) -> Result<()>
    {
        let ingredients: Vec<StackSpec> = ingredients.iter().map(stackSpec).collect::<Result<_>>()?;
        let spec: RecipeSpec = RecipeSpec::Shapeless(ShapelessSpec { ingredients, result: stackSpec(&result)? });
        expectUnit(call(HostCall::RegisterRecipe { name: name.to_owned(), spec }))
    }

    pub fn smelting(&self, name: &str, input: impl Into<ItemRef>, result: ItemStack) -> Result<()>
    {
        let input: Identifier = input.into().name().ok_or("smelting input is not registered")?;
        let spec: RecipeSpec = RecipeSpec::Smelting(SmeltingSpec { input, result: stackSpec(&result)? });
        expectUnit(call(HostCall::RegisterRecipe { name: name.to_owned(), spec }))
    }

    pub fn fuel(&self, name: &str, item: impl Into<ItemRef>, ticks: i32) -> Result<()>
    {
        let item: Identifier = item.into().name().ok_or("fuel item is not registered")?;

        expectUnit(call(HostCall::RegisterRecipe
        {
            name: name.to_owned(),
            spec: RecipeSpec::Fuel(FuelSpec { item, ticks }),
        }))
    }

    pub fn remove(&self, id: &str) -> Result<()> { expectUnit(call(HostCall::RemoveRecipe { id: ownIdentifier(id)? })) }
}
