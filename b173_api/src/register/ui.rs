use crate::rt::call;
use crate::screen::{Draw, Screen};
use crate::{ItemStack, Result, dispatch, expectUnit};
use mod_protocol::abi::{ScreenRequest, ScreenSpec, SlotSpec};
use std::rc::Rc;

pub struct Screens;

pub struct ScreenBuilder
{
    pub m_spec: ScreenSpec,
}

impl ScreenBuilder
{
    pub fn new(title: &str, width: i32, height: i32) -> ScreenBuilder
    {
        ScreenBuilder { m_spec: ScreenSpec { width, height, title: title.to_owned(), ..ScreenSpec::default() } }
    }

    pub fn transparent(mut self) -> Self
    {
        self.m_spec.transparent = true;
        self
    }

    pub fn background(mut self, texture: &str) -> Self
    {
        self.m_spec.background = Some(texture.to_owned());
        self
    }

    pub fn slot(mut self, index: i32, x: i32, y: i32) -> Self
    {
        self.m_spec.slots.push(SlotSpec { index, x, y, output: false });
        self
    }

    pub fn outputSlot(mut self, index: i32, x: i32, y: i32) -> Self
    {
        self.m_spec.slots.push(SlotSpec { index, x, y, output: true });
        self
    }

    pub fn playerInventory(mut self, x: i32, y: i32) -> Self
    {
        self.m_spec.player_inventory = Some([x, y]);
        self
    }
}

impl Screens
{
    pub fn register(&self, name: &str, spec: ScreenBuilder, screen: impl Screen + 'static) -> Result<()>
    {
        let handle: u32 = dispatch::addScreen(screen);
        expectUnit(call(ScreenRequest::RegisterScreen { name: name.to_owned(), spec: spec.m_spec, handler: handle }))
    }

    /// needs api 1.33, a crafting table result slot on a registered container screen, over its nine slots from first
    pub fn craftResult(&self, name: &str, first: i32, x: i32, y: i32) -> Result<()>
    {
        expectUnit(call(ScreenRequest::ScreenCraftSlot { screen: name.to_owned(), first, x, y }))
    }

    /// layers over every player model, from the player and its synced bytes
    pub fn playerLayer(
        &self,
        handler: impl Fn(&mod_protocol::abi::EntityInfo, &[u8]) -> Vec<mod_protocol::abi::PlayerLayer> + 'static,
    ) -> Result<()>
    {
        let handle: u32 = dispatch::addLayer(Rc::new(handler));
        expectUnit(call(ScreenRequest::RegisterPlayerLayer { handler: handle }))
    }

    pub fn overlay(&self, target: &str, overlay: impl crate::screen::Overlay + 'static) -> Result<()>
    {
        let handle: u32 = dispatch::addOverlay(overlay);
        expectUnit(call(ScreenRequest::RegisterOverlay { target: target.to_owned(), handler: handle }))
    }

    /// needs api 1.4, lines pushed here show under the item name in every vanilla slot tooltip
    pub fn tooltip(&self, handler: impl Fn(&ItemStack, &mut Vec<String>) + 'static) -> Result<()>
    {
        let handle: u32 = dispatch::addTooltip(Rc::new(handler));
        expectUnit(call(ScreenRequest::RegisterTooltip { handler: handle }))
    }

    /// needs api 1.8, ops drawn after the world each frame, in block units from the camera position the handler gets
    pub fn worldRender(
        &self,
        handler: impl Fn(f32, [f64; 3]) -> Vec<mod_protocol::abi::ModelOp> + 'static,
    ) -> Result<()>
    {
        let handle: u32 = dispatch::addWorldRender(Rc::new(handler));
        expectUnit(call(ScreenRequest::RegisterWorldRender { handler: handle }))
    }

    pub fn hud(&self, handler: impl Fn(&mut Draw, f32, i32, i32) + 'static) -> Result<()>
    {
        let handle: u32 = dispatch::addHud(Rc::new(handler));
        expectUnit(call(ScreenRequest::RegisterHud { handler: handle }))
    }
}
