use crate::behavior::{self, ItemBehavior};
use crate::rt::call;
use crate::{ItemRef, Result, dispatch, expectUnit, ownIdentifier};
use mod_protocol::Identifier;
use mod_protocol::abi::ContentRequest;
use mod_protocol::spec::{Behavior, ItemMethod, ItemSpec, RecordSpec, ToolClass};
use std::rc::Rc;

pub struct Items;

impl Items
{
    pub fn register(&self, name: &str, spec: ItemSpecBuilder) -> Result<ItemRef>
    {
        let id: Identifier = ownIdentifier(name)?;
        expectUnit(call(ContentRequest::RegisterItem { name: id.to_string(), spec: spec.m_spec }))?;
        Ok(ItemRef::Named(id))
    }

    /// needs api 1.50, changes chosen fields of any item, other mods changing the same field are reported
    pub fn modify(&self, spec: mod_protocol::spec::ItemModifySpec) -> Result<()>
    {
        expectUnit(call(ContentRequest::ModifyItem { spec }))
    }

    /// content from data/ files and other mods registers before code runs, so lookups by name work here
    pub fn get(&self, name: &str) -> Result<ItemRef>
    {
        let id: Identifier = ownIdentifier(name)?;
        Ok(ItemRef::Named(id))
    }
}

pub struct ItemSpecBuilder
{
    pub m_spec: ItemSpec,
}

impl ItemSpecBuilder
{
    pub fn new() -> ItemSpecBuilder { ItemSpecBuilder { m_spec: ItemSpec::default() } }

    pub fn template(template: &str) -> ItemSpecBuilder
    {
        ItemSpecBuilder
        {
            m_spec: ItemSpec { template: super::named(Identifier::parse(template), "template"), ..ItemSpec::default() },
        }
    }

    setters!
    {
        texture(&str) => texture;
        name(&str) => name;
        maxStack(i32) => max_stack;
        maxDamage(i32) => max_damage;
        armor(mod_protocol::spec::ArmorSpec) => armor;
        fuel(i32) => fuel;
        color(u32) => color;
    }

    pub fn food(mut self, heal: i32) -> Self
    {
        self.m_spec.food = Some(mod_protocol::spec::FoodSpec { heal, wolf_food: false });
        self
    }

    pub fn tool(mut self, class: ToolClass, level: i32, speed: f32, damage: i32, durability: i32) -> Self
    {
        self.m_spec.tool = Some(mod_protocol::spec::ItemToolSpec { class, level, speed, damage, durability });
        self.m_spec.full3d = Some(true);
        self
    }

    pub fn record(mut self, sound: &str, title: &str) -> Self
    {
        self.m_spec.record = Some(RecordSpec { sound: sound.to_owned(), title: title.to_owned() });
        self
    }

    pub fn tag(mut self, tag: &str) -> Self
    {
        if let Some(tag) = super::named(Identifier::parse(tag), "tag")
        {
            self.m_spec.tags.push(tag);
        }

        self
    }

    pub fn behavior(mut self, behavior: impl ItemBehavior + 'static) -> Self
    {
        let methods: Vec<ItemMethod> = behavior::methods(behavior.overridden(), &behavior::ITEM_METHODS);
        let handle: u32 = dispatch::addItem(Rc::new(behavior));
        self.m_spec.behavior = Some(Behavior { handle, methods });
        self
    }
}

impl Default for ItemSpecBuilder
{
    fn default() -> Self { ItemSpecBuilder::new() }
}
