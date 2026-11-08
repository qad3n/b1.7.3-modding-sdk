use crate::ItemStack;
use mod_protocol::abi::{DrawList, DrawOp, ScreenOp};

// an immediate draw list, rebuilt each frame by the screen or hud that owns it
#[derive(Default)]
pub struct Draw
{
    pub(crate) m_list: DrawList,
}

impl Draw
{
    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u32) -> &mut Self
    {
        self.m_list.ops.push(DrawOp::Rect { x, y, w, h, color });
        self
    }

    pub fn gradient(&mut self, x: i32, y: i32, w: i32, h: i32, top: u32, bottom: u32) -> &mut Self
    {
        self.m_list.ops.push(DrawOp::Gradient { x, y, w, h, top, bottom });
        self
    }

    pub fn text(&mut self, x: i32, y: i32, text: &str, color: u32) -> &mut Self
    {
        self.m_list.ops.push(DrawOp::Text { x, y, text: text.to_owned(), color, shadow: true, centered: false });
        self
    }

    pub fn centered(&mut self, x: i32, y: i32, text: &str, color: u32) -> &mut Self
    {
        self.m_list.ops.push(DrawOp::Text { x, y, text: text.to_owned(), color, shadow: true, centered: true });
        self
    }

    pub fn texture(&mut self, texture: &str, x: i32, y: i32, u: i32, v: i32, w: i32, h: i32) -> &mut Self
    {
        self.m_list.ops.push(DrawOp::Texture { texture: texture.to_owned(), x, y, u, v, w, h });
        self
    }

    pub fn image(&mut self, texture: &str, x: i32, y: i32, w: i32, h: i32) -> &mut Self
    {
        self.m_list.ops.push(DrawOp::Image { texture: texture.to_owned(), x, y, w, h });
        self
    }

    pub fn item(&mut self, x: i32, y: i32, stack: &ItemStack) -> &mut Self
    {
        if let Ok(stack) = stack.toRaw()
        {
            self.m_list.ops.push(DrawOp::Item { x, y, stack });
        }
        self
    }

    pub fn button(&mut self, id: i32, x: i32, y: i32, w: i32, h: i32, label: &str) -> &mut Self
    {
        self.m_list.ops.push(DrawOp::Button { id, x, y, w, h, label: label.to_owned(), enabled: true });
        self
    }

    pub fn particle(&mut self, name: &str, pos: [f64; 3], motion: [f64; 3]) -> &mut Self
    {
        self.m_list.ops.push(DrawOp::Particle { name: name.to_owned(), pos, motion });
        self
    }
}

// a client screen, slots of a container screen sync through vanilla packets and are drawn by the host
pub trait Screen
{
    fn init(&mut self, _width: i32, _height: i32, _pos: Option<mod_protocol::abi::BlockPos>, _data: &[u8]) {}
    fn draw(&mut self, draw: &mut Draw, mouse: [i32; 2], partial: f32);
    fn button(&mut self, _id: i32) {}
    fn mouse(&mut self, _x: i32, _y: i32, _button: i32) {}
    fn key(&mut self, _character: char, _key: i32) {}
    fn tick(&mut self) {}
    fn close(&mut self) {}
}

// a panel drawn over a vanilla container screen, as a toggleable equipment list beside the inventory
pub trait Overlay
{
    fn draw(&mut self, draw: &mut Draw, size: [i32; 2], mouse: [i32; 2]);
    // true takes the click away from the vanilla screen
    fn mouse(&mut self, _x: i32, _y: i32, _button: i32) -> bool { false }
}

pub(crate) fn runOverlay(overlay: &mut dyn Overlay, op: mod_protocol::abi::OverlayOp) -> mod_protocol::abi::GuestReply
{
    match op
    {
        mod_protocol::abi::OverlayOp::Draw { size, mouse } =>
        {
            let mut draw: Draw = Draw::default();
            overlay.draw(&mut draw, size, mouse);
            mod_protocol::abi::GuestReply::Draw(draw.m_list)
        }
        mod_protocol::abi::OverlayOp::Mouse { x, y, button } => mod_protocol::abi::GuestReply::Bool(overlay.mouse(x, y, button)),
    }
}

pub(crate) fn run(screen: &mut dyn Screen, op: ScreenOp) -> Option<DrawList>
{
    match op
    {
        ScreenOp::Init { width, height, pos, data } => screen.init(width, height, pos, &data),
        ScreenOp::Draw { mouse, partial } =>
        {
            let mut draw: Draw = Draw::default();
            screen.draw(&mut draw, mouse, partial);
            return Some(draw.m_list);
        }
        ScreenOp::Button { id } => screen.button(id),
        ScreenOp::Mouse { x, y, button } => screen.mouse(x, y, button),
        ScreenOp::Key { character, key } => screen.key(char::from_u32(character).unwrap_or('\0'), key),
        ScreenOp::Tick => screen.tick(),
        ScreenOp::Close => screen.close(),
    }
    None
}
