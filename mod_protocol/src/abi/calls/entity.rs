use crate::Identifier;
use crate::abi::{BlockPos, EntityField, ModifierOperation, StackData};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum EntityRequest
{
    SpawnEntity
    {
        world: i32, kind: Identifier, pos: [f64; 3], data: Vec<u8>
    },

    SpawnItem
    {
        world: i32, pos: [f64; 3], stack: StackData
    },

    Entity
    {
        world: i32, id: i32
    },

    EntitiesNear
    {
        world: i32, pos: [f64; 3], radius: f64
    },

    MoveEntity
    {
        world: i32, id: i32, pos: Option<[f64; 3]>, motion: Option<[f64; 3]>, rotation: Option<[f32; 2]>
    },

    KillEntity
    {
        world: i32, id: i32
    },

    MoveTowards
    {
        world: i32, id: i32, pos: BlockPos, speed: f32
    },

    /// numeric fields answer Float, flags answer Bool
    EntityField
    {
        world: i32, id: i32, field: EntityField
    },

    /// entities whose box, grown by radius, the segment crosses, nearest first, as projectiles find what they hit
    EntitiesAlong
    {
        world: i32, from: [f64; 3], to: [f64; 3], radius: f64, exclude: Option<i32>
    },

    /// EntityLiving.mountEntity, None leaves the current vehicle
    Mount
    {
        world: i32, id: i32, vehicle: Option<i32>
    },

    /// flags take any value above zero as true
    SetEntityField
    {
        world: i32, id: i32, field: EntityField, value: f64
    },

    /// api 1.8: the stack a dropped item entity carries, answered with Stack and None for any other entity
    ItemEntityStack
    {
        world: i32, id: i32
    },

    /// api 1.17: the damage type of the hit being handled, such as minecraft:fall or minecraft:arrow, answered with Id
    /// and None outside an attack, hurt or onHurt call
    DamageType,

    /// api 1.17: DamageEntity with a vanilla damage type, answered with Bool
    DamageEntityAs
    {
        world: i32, id: i32, source: Option<i32>, amount: i32, kind: Identifier
    },

    /// api 1.18: adds or replaces the modifier with this id on a living entity's attribute, such as
    /// minecraft:movement_speed; saved with the entity and sent to a player's own client
    SetModifier
    {
        world: i32,
        id: i32,
        attribute: Identifier,
        modifier: Identifier,
        amount: f64,
        operation: ModifierOperation,
    },

    /// api 1.18
    RemoveModifier
    {
        world: i32, id: i32, attribute: Identifier, modifier: Identifier
    },

    /// api 1.22: a stronger or longer effect replaces the one already on the entity, level 0 is the first level
    AddEffect
    {
        world: i32, id: i32, effect: Identifier, level: i32, ticks: i32
    },

    /// api 1.22
    RemoveEffect
    {
        world: i32, id: i32, effect: Identifier
    },

    /// api 1.22: the level of an effect on an entity, answered with Int and -1 when it has none
    EffectLevel
    {
        world: i32, id: i32, effect: Identifier
    },

    /// api 1.28: Entity.moveFlying, turns strafe and forward input into motion along the entity's yaw at factor
    Accelerate
    {
        world: i32, id: i32, strafe: f32, forward: f32, factor: f32
    },

    /// api 1.28: Entity.moveEntity, moves by motion through the world's blocks and stops at what it hits, updating
    /// on_ground, collision and fall distance
    Step
    {
        world: i32, id: i32, motion: [f64; 3]
    },

    /// api 1.28: eases a living entity's leg swing toward how far it moved this tick, as the end of
    /// EntityLiving.moveEntityWithHeading
    AnimateWalk
    {
        world: i32, id: i32
    },

    /// api 1.27: an attribute's default with the entity's modifiers applied, answered with Double; a vanilla
    /// attribute whose base the game works out where it reads it starts from its registered default here
    AttributeValue
    {
        world: i32, id: i32, attribute: Identifier
    },

    /// api 1.31: every registered living entity a mob spawner can make, vanilla and mod, in numeric id order,
    /// answered with Ids
    MobList,

    /// api 1.33: changes the stack a dropped item entity holds, an empty count removes it
    SetItemStack
    {
        world: i32, id: i32, stack: StackData
    },

    /// api 1.33: every entity whose bounding box meets a box in world coordinates, answered with Entities
    EntitiesIn
    {
        world: i32, min: [f64; 3], max: [f64; 3]
    },

    /// api 1.33: drops a stack with a motion and pickup delay, as machines throw their output, answered with Int,
    /// the dropped item's entity id
    DropItem
    {
        world: i32, pos: [f64; 3], stack: StackData, motion: [f64; 3], delay: i32
    },

    /// api 1.40: every accessory slot in registration order with what the entity wears there, answered with
    /// Accessories, readable on clients too
    Accessories
    {
        world: i32, entity: i32
    },

    /// api 1.40: puts a stack in an entity's accessory slot or empties it, authority only, refused for a stack the
    /// slot's tag does not take or of more than one item
    SetAccessory
    {
        world: i32, entity: i32, slot: Identifier, stack: Option<StackData>
    },
}
