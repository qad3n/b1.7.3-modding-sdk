use super::{BlockPos, EntityInfo, StackData};
use crate::Identifier;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum DrawOp
{
    Rect
    {
        x: i32, y: i32, w: i32, h: i32, color: u32
    },

    Gradient
    {
        x: i32, y: i32, w: i32, h: i32, top: u32, bottom: u32
    },

    Text
    {
        x: i32, y: i32, text: String, color: u32, shadow: bool, centered: bool
    },

    Texture
    {
        texture: String, x: i32, y: i32, u: i32, v: i32, w: i32, h: i32
    },

    Item
    {
        x: i32, y: i32, stack: StackData
    },

    /// a whole texture of any size stretched over a rectangle, blended by its alpha
    Image
    {
        texture: String, x: i32, y: i32, w: i32, h: i32
    },

    Button
    {
        id: i32, x: i32, y: i32, w: i32, h: i32, label: String, enabled: bool
    },

    Particle
    {
        name: String, pos: [f64; 3], motion: [f64; 3]
    },

    /// a u v region of a texture of any size stretched over a rectangle, as Gui.drawScaledCustomSizeModalRect
    Region
    {
        texture: String, x: i32, y: i32, w: i32, h: i32, uv: [f32; 4], size: [f32; 2]
    },

    /// api 1.3, Image with its color and alpha multiplied by an argb tint, as vanilla fades the portal overlay
    TintedImage
    {
        texture: String, x: i32, y: i32, w: i32, h: i32, color: u32
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct DrawList
{
    pub ops: Vec<DrawOp>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct SlotSpec
{
    pub index: i32,
    pub x: i32,
    pub y: i32,
    pub output: bool,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct ScreenSpec
{
    pub width: i32,
    pub height: i32,
    pub background: Option<String>,
    pub title: String,
    /// a container screen syncs these tile entity slots plus the player inventory through vanilla slot packets,
    /// opened without a block they live only while the screen is open and drop at the player when it closes
    pub slots: Vec<SlotSpec>,
    pub player_inventory: Option<[i32; 2]>,
    /// a plain screen drawn over the world without the dark backdrop, as dialogue screens are
    pub transparent: bool,
}

/// a piece of a player model a mod draws over a player, as armor layers and worn accessories do
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum BipedPart
{
    Head,
    Body,
    RightArm,
    LeftArm,
    RightLeg,
    LeftLeg,
    /// the cape, drawn swinging like a vanilla cloak
    Cloak,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct PlayerLayer
{
    pub texture: String,
    pub color: u32,
    pub parts: Vec<BipedPart>,
    /// how far the layer's boxes grow past the player's, armor uses 1.0 and 0.5
    pub inflate: f32,
    /// drawn at full brightness, as glowing armor
    pub glow: bool,
    /// scales the layer about the model origin, slightly smaller layers sit inside the skin
    pub scale: f32,
    /// blended by the texture's alpha, as a translucent shield
    pub blend: bool,
    /// extra geometry in the player model's space after the parts, such as wings, quads and transforms only
    pub ops: Vec<ModelOp>,
}

/// a call to an overlay on a vanilla container screen, positions relative to the container's corner
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum OverlayOp
{
    Draw
    {
        size: [i32; 2], mouse: [i32; 2]
    },

    /// answered with whether the overlay took the click away from the screen
    Mouse
    {
        x: i32, y: i32, button: i32
    },

    /// api 1.2: a key typed on the screen, hovered is the stack in the vanilla slot under the mouse, answered with whether
    /// the overlay took the key away from the screen
    Key
    {
        character: u32, key: i32, hovered: Option<StackData>
    },

    /// api 1.2: the mouse wheel over the screen, delta positive when rolled away from the player, answered as Mouse
    Scroll
    {
        x: i32, y: i32, delta: i32
    },

    /// api 1.5: asked each frame before the screen draws, answered with whether the overlay covers the point so the
    /// vanilla slot under it shows no highlight or tooltip
    Cover
    {
        x: i32, y: i32
    },
    /// api 1.6: asked each frame before the screen draws, answered with whether the overlay fills the container, while
    /// one does the screen draws and asks only that overlay
    Exclusive,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum ScreenOp
{
    Init
    {
        width: i32,
        height: i32,
        pos: Option<BlockPos>,
        data: Vec<u8>,
    },

    Draw
    {
        mouse: [i32; 2],
        partial: f32,
    },

    Button
    {
        id: i32,
    },

    Mouse
    {
        x: i32,
        y: i32,
        button: i32,
    },

    Key
    {
        character: u32,
        key: i32,
    },
    Tick,
    Close,
    /// a mouse button let go, for sliders dragged since its click
    Release
    {
        x: i32,
        y: i32,
        button: i32,
    },
}

/// what RenderLiving knows when it draws a living entity
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct PoseInput
{
    pub entity: EntityInfo,
    pub walk: f32,
    pub walk_speed: f32,
    pub age: f32,
    pub head_yaw: f32,
    pub head_pitch: f32,
    pub partial: f32,
    pub on_ground: bool,
    pub riding: bool,
    pub ridden_by_local: bool,
    pub riding_local: bool,
    /// the rider's width plus height, zero without one
    pub rider_size: f32,
    /// EntityLiving.hurtTime, counting down from 10 after a hit
    pub hurt_time: i32,
    pub synced: Vec<u8>,
}

/// one step of a model draw program, GL transforms in block units after the model's -1 scale flip
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum ModelOp
{
    Push,
    Pop,
    Translate([f32; 3]),
    /// degrees around an axis
    Rotate(f32, [f32; 3]),
    Scale([f32; 3]),
    Color([f32; 4]),
    /// binds a texture by identifier, as Render.loadTexture
    Texture(String),
    Blend(bool),
    /// draws a part and its children with this rotation and pivot
    Part
    {
        part: u16,
        rotation: [f32; 3],
        pivot: [f32; 3],
    },
    /// draws a part as the spec placed it
    Rest(u16),
    /// turns to face the camera like RenderSnowball, for sprites
    FaceCamera,
    /// quads of x, y, z, u, v vertices in GL units and 0 to 1 texture coordinates, four per quad
    Quads(Vec<[f32; 5]>),
    /// glNormal3f for the quads that follow
    Normal([f32; 3]),
    /// GL_LIGHTING off or back on, for glowing sprites
    Lighting(bool),
    /// api 1.8: a stack held in the hand at the end of a part, drawn as RenderBiped draws a held item
    HeldItem
    {
        part: u16,
        stack: StackData,
    },
    /// api 1.33: a stack drawn where the transform stands as a resting dropped item looks, a block as a small cube
    /// and an item as a sprite turned to the camera, more copies for a bigger stack, lit by a 0 to 1 brightness
    Item
    {
        stack: StackData,
        brightness: f32,
    },
    /// api 1.35: a line of the client font with its corner where the transform stands, one unit a font pixel, as a
    /// sign draws its text; scale the transform down first
    Text
    {
        text: String,
        x: i32,
        y: i32,
        color: u32,
    },
    /// api 1.35: glDepthMask for what follows, off so text over a face never hides what draws after it
    DepthMask(bool),
}

/// the draw program for one frame: pre runs where preRenderCallback does, model replaces ModelBase.render
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct Pose
{
    pub pre: Vec<ModelOp>,
    pub model: Vec<ModelOp>,
}

/// api 1.13: a sprite laid over a block face, as BlockTFCastleMagic's runes
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub struct FaceOverlay
{
    /// a terrain atlas slot, as terrainSlot answers
    pub sprite: i32,
    /// multiplies the sprite, 0xFFFFFF keeps its colors
    pub color: u32,
    /// drawn at full brightness instead of the light in front of the face
    pub glow: bool,
    /// pixels the sprite floats out from the face
    pub lift: f32,
}

/// api 1.33: how a block's item draws in slots, in hand and dropped: model cuboids, and a tile model posed once,
/// both in block space from the lower corner as the block stands in the world
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct ItemModelSpec
{
    pub cuboids: Vec<crate::spec::Cuboid>,
    pub tile: Option<Identifier>,
    pub pose: Pose,
}
