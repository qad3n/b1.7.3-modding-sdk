use crate::Identifier;
use crate::abi::{BlockPos, Handle, ScreenSpec};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum ScreenRequest
{
    RegisterKeybind
    {
        name: String,
        key: i32,
        handler: Handle,
    },

    RegisterScreen
    {
        name: String,
        #[serde(with = "crate::wire::named")]
        spec: ScreenSpec,
        handler: Handle,
    },

    RegisterHud
    {
        handler: Handle,
    },

    /// draws over a vanilla screen by its class name, GuiInventory for the player inventory, and may take its clicks
    RegisterOverlay
    {
        target: String,
        handler: Handle,
    },

    /// layers drawn over every player, asked for each player each frame
    RegisterPlayerLayer
    {
        handler: Handle,
    },

    /// the width in pixels the client font draws a string at, zero on a server
    TextWidth
    {
        text: String,
    },

    OpenScreen
    {
        player: i32,
        screen: Identifier,
        pos: Option<BlockPos>,
        data: Vec<u8>,
    },

    CloseScreen
    {
        player: i32,
    },

    ScreenSize,

    /// api 1.4: lines a mod adds under the name of every item tooltip, asked for the hovered stack each frame
    RegisterTooltip
    {
        handler: Handle,
    },

    /// api 1.8: the pixels of a texture a mod draws at runtime, argb rows from the top left, bound by its name from then
    /// on wherever a texture is named; a no-op on a dedicated server
    SetDynamicTexture
    {
        name: Identifier,
        width: u32,
        height: u32,
        pixels: Vec<u32>,
    },

    /// api 1.8: draws in the world after it and before the hand each frame, the handler answers ops around the camera
    RegisterWorldRender
    {
        handler: Handle,
    },

    /// api 1.12: the terrain atlas slot of a block texture for the texture hooks, answered with Int, -1 when unknown; a
    /// call during registration also reserves the texture a block spec does not name
    TerrainSlot
    {
        texture: String,
    },

    /// api 1.13: cuts a terrain texture into a grid of tiles, each blown up to a whole sprite of its own, as giant blocks
    /// show one part of a vanilla face on each block; call while registering, then ask terrainSlot for each tile by name
    TerrainTiles
    {
        texture: String,
        columns: i32,
        rows: i32,
    },

    /// api 1.23: the item sheet slot of a texture for the item icon hooks, answered with Int and -1 when unknown; a
    /// call during registration also reserves the texture no item spec names
    ItemSlot
    {
        texture: String,
    },

    /// api 1.33: a crafting table result slot on a container screen, showing what the nine screen slots from first
    /// make and taking one craft from them as the workbench does; register phase, the screen's own mod
    ScreenCraftSlot
    {
        screen: String,
        first: i32,
        x: i32,
        y: i32,
    },

    /// api 1.33: redraws a terrain or item sprite from square argb pixels a mod made in code, as TextureFX effects
    /// do each tick; client only, a server ignores it
    PaintSprite
    {
        texture: Identifier,
        size: u32,
        pixels: Vec<u32>,
    },

    /// api 1.35: the pixels of a terrain or item sprite as the active texture packs give it, first frame of an
    /// animation, answered with Pixels; the base image a TextureFX copies before drawing over it, client only
    SpritePixels
    {
        texture: Identifier,
    },

    /// api 2.1: the client clipboard as Text, an error on a server or when it holds no text
    Clipboard,
}
