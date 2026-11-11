use crate::world::{contentGeneration, machineList};
use crate::{ItemStack, epoch, modId};
use mod_protocol::Identifier;
use mod_protocol::abi::{MachineRecipeView, MachineSlot, StackData};
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

type Recipes = Rc<Vec<Rc<MachineRecipeView>>>;
// keyed by the machine as callers spell it, each entry keeps the epoch and content generation it was read in
type RecipeCache = HashMap<String, (u32, i32, Recipes)>;

thread_local!
{
    static RECIPES: RefCell<RecipeCache> = RefCell::new(HashMap::new());
}

/// needs api 1.35, every recipe registered for a machine by any mod, a bare name means this mod's, cached until the
/// numeric ids or the content change
pub fn machineRecipes(machine: &str) -> Recipes
{
    let epoch: u32 = epoch();
    let generation: i32 = contentGeneration();
    let cached: Option<Recipes> = RECIPES.with(|cache|
    {
        let cache: Ref<RecipeCache> = cache.borrow();
        let (at, built, recipes): &(u32, i32, Recipes) = cache.get(machine)?;
        (*at == epoch && *built == generation).then(|| Rc::clone(recipes))
    });
    if let Some(recipes) = cached
    {
        return recipes;
    }

    let recipes: Recipes = match Identifier::parseIn(machine, &modId())
    {
        Ok(id) => Rc::new(
            machineList()
                .into_iter()
                .find(|view| view.id == id)
                .map(|view| view.recipes.into_iter().map(Rc::new).collect())
                .unwrap_or_default(),
        ),
        Err(_) => Rc::default(),
    };
    RECIPES.with(|cache| cache.borrow_mut().insert(machine.to_owned(), (epoch, generation, Rc::clone(&recipes))));
    recipes
}

// a slot takes a stack of one of its items, any damage when it lists -1, and at least its count
fn fills(slot: &MachineSlot, stack: &StackData) -> bool
{
    slot.stacks.iter().any(|want|
    {
        let damage: bool = want.damage == -1 || want.damage == stack.damage;
        want.id == stack.id && damage && stack.count >= want.count
    })
}

// each input slot needs a stack of its own, the bits of used mark the stacks already given to a slot
fn matches(recipe: &MachineRecipeView, stacks: &[ItemStack]) -> bool
{
    let mut used: u64 = 0;
    let count: usize = stacks.len().min(u64::BITS as usize);

    !recipe.inputs.is_empty()
        && recipe.inputs.iter().all(|slot|
        {
            let found: Option<usize> = (0..count)
                .find(|index| used & 1 << index == 0 && stacks[*index].raw().is_ok_and(|raw| fills(slot, &raw)));
            found.inspect(|index| used |= 1 << index).is_some()
        })
}

/// needs api 1.35, the first recipe of a machine whose inputs the stacks fill, tag and any damage inputs included, so
/// a machine runs on what every mod registered for it
pub fn machineRecipe(machine: &str, stacks: &[ItemStack]) -> Option<Rc<MachineRecipeView>>
{
    machineRecipes(machine).iter().find(|recipe| matches(recipe, stacks)).map(Rc::clone)
}
