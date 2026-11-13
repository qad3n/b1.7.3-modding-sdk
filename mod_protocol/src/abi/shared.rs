use super::{FluidStack, StackData};
use crate::Identifier;
use serde::{Deserialize, Serialize};

/// api 1.44: a container filled or drained, the stack one item turns into, none when used up, and the fluid moved
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ContainerSwap
{
    pub stack: Option<StackData>,
    pub fluid: FluidStack,
}

/// api 1.40: whose loot a pool joins
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum LootTarget
{
    Block,
    Entity,
    Chest,
}

/// api 1.40: a machine job matched to the stacks a machine holds, slots giving the index of the stack each job input
/// took, outputs with their items resolved to this session's numeric ids
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct JobMatch
{
    /// the job's place among its machine's jobs, stable until the content reloads
    pub index: u32,
    pub slots: Vec<u32>,
    pub inputs: Vec<crate::spec::JobInputSpec>,
    pub outputs: Vec<JobOutput>,
    /// fluid inputs name the given fluid that covered them, a tag input included
    pub fluid_inputs: Vec<FluidStack>,
    pub fluid_outputs: Vec<FluidStack>,
    /// a job timed by fuel has the machine fuel value of the stack that filled its first input here
    pub duration: i32,
    pub energy_in: Option<crate::spec::JobEnergySpec>,
    pub energy_out: Option<crate::spec::JobEnergySpec>,
}

/// api 1.40: one output of a matched job, the stack it gives, or count of the input it hands back with its damage set
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct JobOutput
{
    pub stack: Option<StackData>,
    pub count: i32,
    pub chance: f32,
    pub from_input: Option<u32>,
    pub damage: Option<i32>,
}

/// api 1.40: an accessory slot and the stack worn in it
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Worn
{
    pub slot: Identifier,
    pub icon: Option<String>,
    pub stack: Option<StackData>,
}

/// api 1.32: the value of a game rule, in data files written as {bool = true} or {int = 3}
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum GameRuleValue
{
    Bool(bool),
    Int(i32),
}

/// api 1.33: the energy kind a block takes and the packet sizes it takes now, both ends included
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct EnergyDemand
{
    pub kind: Identifier,
    pub min: i64,
    pub max: i64,
}

/// api 2.0: what a block holds of an energy kind and the most it can hold, in the asker's kind
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct EnergyLevel
{
    pub kind: Identifier,
    pub stored: i64,
    pub capacity: i64,
}

/// api 2.0: what a loader-kept tank holds, no fluid when it is empty
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct TankLevel
{
    pub fluid: Option<Identifier>,
    pub amount: i32,
    pub capacity: i32,
}

/// api 1.33: the bits HostCall::Capabilities answers with
pub mod capability
{
    pub const ITEMS: i32 = 1;
    pub const FLUIDS: i32 = 2;
    pub const ENERGY: i32 = 4;
    /// a mod tile that answers item transfers itself rather than through its slots
    pub const HANDLER: i32 = 8;
}

/// how a modifier combines with an attribute: adds first, then base multipliers summed, then total multipliers one by one
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ModifierOperation
{
    Add,
    MultiplyBase,
    MultiplyTotal,
}

/// entity state beyond EntityInfo that ported mods read or change, new fields go at the end
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum EntityField
{
    FallDistance,
    OnGround,
    Sneaking,
    InWater,
    Riding,
    FireTicks,
    Air,
    TicksExisted,
    /// entity id of the vehicle or the rider, -1 for none, read only
    RidingId,
    RiderId,
    /// EntityLiving steering input, a riding player's keys reach its vehicle through these
    MoveForward,
    MoveStrafe,
    Jumping,
    Yaw,
    Pitch,
    /// EntityCreature.playerToAttack, -1 for none
    TargetId,
    /// EntityMob.attackStrength, the melee damage of a monster
    AttackStrength,
    /// Entity.setSize, writing either keeps the other
    Width,
    Height,
    /// EntityLiving.swingProgress, above zero while a player swings its arm, read only
    SwingProgress,
    /// the block id a moving_block or primed_block entity draws
    Block,
    /// Entity.isCollided, the last move ran into a block on any side, read only
    Collided,
    /// EntityLiving.health, writing heals or hurts without a hit
    Health,
    /// the bottom of the bounding box, a player's feet rather than its eyes, read only
    Bottom,
    /// EntityLiving.moveSpeed, how fast its own AI walks it
    MoveSpeed,
    /// Entity.stepHeight, how tall a block it walks up without jumping
    StepHeight,
    /// the health it heals up to and the hearts a player sees, 20 unless a mod raised it
    MaxHealth,
    /// drawn without its model or shadow on this side, as an invisibility potion in later versions
    Invisible,
    /// who fired an arrow, snowball, egg, fireball or mod projectile, -1 for nobody and -2 for anything else
    Shooter,
    /// api 1.12: EntityLiving.deathTime, ticks since its health ran out, it is removed when this reaches 20
    DeathTime,
    /// api 1.12: the metadata a moving_block or primed_block entity draws its block with
    BlockMeta,
    /// api 1.20: ticks left on a player's item in use, 0 when it holds nothing in use, read only
    ItemInUse,
    /// api 1.28: Entity.handleLavaMovement, read only
    InLava,
    /// api 1.28: what EntityLiving.moveEntityWithHeading multiplies horizontal motion by each tick, 0.91 in the air
    /// and the block's slipperiness times 0.91 on the ground, 0.546 for stone; read only
    Slipperiness,
    /// api 1.33: EntityItem.delayBeforeCanPickup, ticks before a player can take a dropped stack
    PickupDelay,
    /// api 1.33: EntityMinecart.minecartType, 0 to ride, 1 with a chest, 2 with a furnace; read only
    MinecartType,
    /// api 1.36: Entity.getMountedYOffset, how far above the mount's feet a rider sits; setting it overrides the
    /// mount's own on the side that sets it, so a mod sets it on server and client
    Seat,
    /// api 1.36: EntityLiving.renderYawOffset, the body's yaw the head turns from
    BodyYaw,
    /// api 1.57: EntityTNTPrimed.fuse or a primed_block's, ticks left before it goes off; a primed_block counts it
    /// down on both sides and its mod sets it off once it drops below zero, a server write reaches watching clients
    Fuse,
}
