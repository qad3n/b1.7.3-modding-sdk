use crate::ItemStack;
use mod_protocol::abi::{DrawList, DrawOp, ScreenOp};

// colors of the b1.7.3 container sheets
mod gui
{
    pub const FACE: u32 = 0xFFC6C6C6;
    pub const LIGHT: u32 = 0xFFFFFFFF;
    pub const SHADOW: u32 = 0xFF555555;
    pub const EDGE: u32 = 0xFF000000;
    pub const SLOT: u32 = 0xFF8B8B8B;
    pub const SLOT_SHADOW: u32 = 0xFF373737;
}

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

    // FontRenderer.drawString without the shadow, as container titles draw
    pub fn plain(&mut self, x: i32, y: i32, text: &str, color: u32) -> &mut Self
    {
        self.m_list.ops.push(DrawOp::Text { x, y, text: text.to_owned(), color, shadow: false, centered: false });
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

    // the uv region [u, v, width, height] of a texture of size [width, height] stretched over the rectangle
    pub fn region(&mut self, texture: &str, rect: [i32; 4], uv: [f32; 4], size: [f32; 2]) -> &mut Self
    {
        let [x, y, w, h] = rect;
        self.m_list.ops.push(DrawOp::Region { texture: texture.to_owned(), x, y, w, h, uv, size });
        self
    }

    pub fn image(&mut self, texture: &str, x: i32, y: i32, w: i32, h: i32) -> &mut Self
    {
        self.m_list.ops.push(DrawOp::Image { texture: texture.to_owned(), x, y, w, h });
        self
    }

    // needs api 1.3, color is argb so the alpha byte fades the whole image
    pub fn tinted(&mut self, texture: &str, x: i32, y: i32, w: i32, h: i32, color: u32) -> &mut Self
    {
        self.m_list.ops.push(DrawOp::TintedImage { texture: texture.to_owned(), x, y, w, h, color });
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

    // a container background as the b1.7.3 gui sheets paint it, rounded corners included, for any size
    pub fn panel(&mut self, x: i32, y: i32, w: i32, h: i32) -> &mut Self
    {
        let rects: [([i32; 4], u32); 19] = [
            ([2, 2, w - 4, h - 4], gui::FACE),
            ([2, 1, w - 5, 1], gui::LIGHT),
            ([1, 2, w - 4, 1], gui::LIGHT),
            ([1, 3, 2, h - 6], gui::LIGHT),
            ([3, 3, 1, 1], gui::LIGHT),
            ([w - 3, 3, 2, h - 5], gui::SHADOW),
            ([3, h - 3, w - 4, 1], gui::SHADOW),
            ([3, h - 2, w - 5, 1], gui::SHADOW),
            ([w - 4, h - 4, 1, 1], gui::SHADOW),
            ([2, 0, w - 5, 1], gui::EDGE),
            ([0, 2, 1, h - 5], gui::EDGE),
            ([3, h - 1, w - 5, 1], gui::EDGE),
            ([w - 1, 3, 1, h - 5], gui::EDGE),
            ([1, 1, 1, 1], gui::EDGE),
            ([w - 3, 1, 1, 1], gui::EDGE),
            ([w - 2, 2, 1, 1], gui::EDGE),
            ([1, h - 3, 1, 1], gui::EDGE),
            ([2, h - 2, 1, 1], gui::EDGE),
            ([w - 2, h - 2, 1, 1], gui::EDGE),
        ];
        for ([left, top, width, height], color) in rects
        {
            self.rect(x + left, y + top, width, height, color);
        }
        self
    }

    // the sunken frame of a vanilla slot around an item area of size pixels with its corner at x y
    pub fn slot(&mut self, x: i32, y: i32, size: i32) -> &mut Self
    {
        self.rect(x, y, size, size, gui::SLOT);
        self.rect(x - 1, y - 1, size + 1, 1, gui::SLOT_SHADOW);
        self.rect(x - 1, y, 1, size, gui::SLOT_SHADOW);
        self.rect(x + size, y - 1, 1, 1, gui::SLOT);
        self.rect(x - 1, y + size, 1, 1, gui::SLOT);
        self.rect(x + size, y, 1, size, gui::LIGHT);
        self.rect(x, y + size, size + 1, 1, gui::LIGHT);
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
    fn release(&mut self, _x: i32, _y: i32, _button: i32) {}
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
    // true takes the key away from the vanilla screen, hovered is the stack in the vanilla slot under the mouse
    fn key(&mut self, _character: char, _key: i32, _hovered: Option<ItemStack>) -> bool { false }
    // the mouse wheel, delta positive when rolled away from the player, true takes it away from the vanilla screen
    fn scroll(&mut self, _x: i32, _y: i32, _delta: i32) -> bool { false }
    // needs api 1.5, true when the panel covers the point so the vanilla slot under it shows no highlight or tooltip
    fn covered(&mut self, _x: i32, _y: i32) -> bool { false }
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
        mod_protocol::abi::OverlayOp::Mouse { x, y, button } =>
        {
            mod_protocol::abi::GuestReply::Bool(overlay.mouse(x, y, button))
        }
        mod_protocol::abi::OverlayOp::Key { character, key, hovered } =>
        {
            let character: char = char::from_u32(character).unwrap_or('\0');
            mod_protocol::abi::GuestReply::Bool(overlay.key(character, key, hovered.map(ItemStack::fromRaw)))
        }
        mod_protocol::abi::OverlayOp::Scroll { x, y, delta } =>
        {
            mod_protocol::abi::GuestReply::Bool(overlay.scroll(x, y, delta))
        }
        mod_protocol::abi::OverlayOp::Cover { x, y } => mod_protocol::abi::GuestReply::Bool(overlay.covered(x, y)),
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
        ScreenOp::Release { x, y, button } => screen.release(x, y, button),
    }
    None
}
