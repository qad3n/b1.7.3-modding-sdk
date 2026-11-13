use crate::Identifier;
use crate::abi::StackData;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum PlayerRequest
{
    Players
    {
        world: Option<i32>,
    },

    Chat
    {
        player: Option<i32>,
        message: String,
    },

    Give
    {
        player: i32,
        stack: StackData,
    },

    HeldStack
    {
        player: i32,
    },

    SetHeldStack
    {
        player: i32,
        stack: Option<StackData>,
    },

    Teleport
    {
        player: i32,
        dimension: i32,
        pos: [f64; 3],
    },

    LocalPlayer,

    /// an operator on a server, always true in single player
    IsOp
    {
        player: i32,
    },

    DamageHeld
    {
        player: i32,
        amount: i32,
    }, // api 1.7: the biomes of a mod dimension come from the mod, listed in the index order its handler answers with

    /// api 1.14: how fast a player digs a block with what it holds, EntityPlayer.getCurrentPlayerStrVsBlock, answered
    /// with Float
    DigSpeed
    {
        player: i32,
        block: i32,
    },

    /// api 1.20: holds the player's current item in use for ticks, as EntityPlayer.setItemInUse, 0 lets it go
    SetItemInUse
    {
        player: i32,
        ticks: i32,
    },

    /// api 1.20: the player lets go of the item in use, its item hears onStoppedUsing
    StopUsingItem
    {
        player: i32,
    },

    /// api 1.25: as beta, an achievement needs its parent; answered with Bool, true when newly earned
    UnlockAchievement
    {
        player: i32,
        achievement: Identifier,
    },

    /// api 1.25: answered with Bool
    HasAchievement
    {
        player: i32,
        achievement: Identifier,
    },

    /// api 1.25: a named stat such as <modid>:bosses_killed; mod blocks and items also count mined/<id>,
    /// crafted/<id>, used/<id> and broken/<id> on their own; api 1.56 registers an unregistered name with defaults
    AddStat
    {
        player: i32,
        stat: String,
        amount: i32,
    },

    /// api 1.25: answered with Int
    StatValue
    {
        player: i32,
        stat: String,
    },

    /// api 1.40: SlotAccessory clicked by a player, the cursor stack and the slot swap when the slot takes the cursor
    /// stack, authority only, answered with Bool
    SwapAccessory
    {
        player: i32,
        slot: Identifier,
    },
}
