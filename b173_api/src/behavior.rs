use crate::{BlockCtx, Entity, EntityInfo, ItemStack, Player, World};
use mod_protocol::abi::{BlockPos, EnergyDemand, FaceOverlay, FluidStack, Pose, PoseInput, StateData, UseAction};
use mod_protocol::spec::{BlockMethod, EffectMethod, EntityMethod, ItemMethod, TileMethod};

/// write the methods you need inside an impl marked #[block_behavior], only those cross from the game into wasm
pub trait BlockBehavior
{
    fn onActivated(&self, _ctx: &mut BlockCtx, _player: Player) -> bool { false }
    fn onClicked(&self, _ctx: &mut BlockCtx, _player: Player) {}
    fn onAdded(&self, _ctx: &mut BlockCtx) {}
    fn onRemoved(&self, _ctx: &mut BlockCtx) {}
    fn onPlaced(&self, _ctx: &mut BlockCtx, _face: i32) {}
    fn onPlacedBy(&self, _ctx: &mut BlockCtx, _entity: Entity) {}
    fn onNeighborChanged(&self, _ctx: &mut BlockCtx, _neighbor: i32) {}
    fn onUpdateTick(&self, _ctx: &mut BlockCtx) {}
    fn onRandomDisplayTick(&self, _ctx: &mut BlockCtx) {}
    fn onEntityCollided(&self, _ctx: &mut BlockCtx, _entity: Entity) {}
    fn onEntityWalking(&self, _ctx: &mut BlockCtx, _entity: Entity) {}
    fn onDestroyedByPlayer(&self, _ctx: &mut BlockCtx) {}
    fn onDestroyedByExplosion(&self, _ctx: &mut BlockCtx) {}
    fn onHarvest(&self, _ctx: &mut BlockCtx, _player: Player) {}
    fn canPlaceAt(&self, _ctx: &mut BlockCtx) -> bool { true }
    fn canStay(&self, _ctx: &mut BlockCtx) -> bool { true }
    fn poweringTo(&self, _ctx: &mut BlockCtx, _direction: i32) -> bool { false }
    fn indirectlyPoweringTo(&self, _ctx: &mut BlockCtx, _direction: i32) -> bool { false }
    fn canProvidePower(&self) -> bool { false }
    fn texture(&self, _face: i32, _meta: i32) -> i32 { 0 }
    fn colorMultiplier(&self, _ctx: &mut BlockCtx) -> u32 { 0xFFFFFF }
    /// needs api 1.12, a face's sprite slot from where the block stands, as Block.getBlockTexture; None keeps texture
    fn faceTexture(&self, _ctx: &mut BlockCtx, _face: i32) -> Option<i32> { None }
    /// needs api 1.45, a face's sprite for a named state, preferred over texture; None keeps texture or the spec's
    fn textureFor(&self, _face: i32, _state: &StateData) -> Option<String> { None }
    /// needs api 1.13, the share of the block one tick of the player's digging breaks, 0 for unbreakable; None keeps
    /// beta's hardness and tool speed
    fn strength(&self, _ctx: &mut BlockCtx, _player: Player) -> Option<f32> { None }
    /// needs api 1.13, a sprite drawn over a face after the block, as a glowing rune
    fn overlay(&self, _ctx: &mut BlockCtx, _face: i32) -> Option<FaceOverlay> { None }
    /// needs api 1.14, quarter turns of each face's sprite by face index, as a block drawn on its side
    fn faceRotations(&self, _ctx: &mut BlockCtx) -> Option<[u8; 6]> { None }
    /// needs api 1.33, the faces a connecting block joins where it stands, a bit per face index; model cuboids whose
    /// connect names a face show only with its bit, and the loader keeps the answer for the rest of the tick
    fn connections(&self, _ctx: &mut BlockCtx) -> u8 { 0 }
    /// needs api 1.33, how bright the block draws given the light where it stands, as a machine lit up by work
    fn brightness(&self, _ctx: &mut BlockCtx, _base: f32) -> Option<f32> { None }
    /// needs api 1.53, an event World::blockEvent sent to this block, on the server and on every client near it
    fn onBlockEvent(&self, _ctx: &mut BlockCtx, _kind: i32, _data: i32) {}
    fn overridden(&self) -> &'static [&'static str] { unmarked(std::any::type_name::<Self>(), "block_behavior") }
}

/// a held item use either passes through or replaces the stack in hand
pub enum Use
{
    Pass,
    Done(Option<ItemStack>),
}

pub trait ItemBehavior
{
    fn onRightClick(&self, _world: World, _player: Player, _stack: ItemStack) -> Use { Use::Pass }
    fn onUseOnBlock(&self, _world: World, _player: Player, _stack: ItemStack, _pos: BlockPos, _face: i32) -> Use
    {
        Use::Pass
    }
    fn onHitEntity(&self, _stack: ItemStack, _target: Entity, _attacker: i32) -> bool { true }
    fn onInteractEntity(&self, _stack: ItemStack, _target: Entity) {}
    fn onBlockDestroyed(&self, _stack: ItemStack, _block: i32, _pos: BlockPos, _player: Player) -> bool { true }
    fn icon(&self, _damage: i32) -> i32 { 0 }
    fn strVsBlock(&self, _stack: ItemStack, _block: i32) -> f32 { 1.0 }
    fn canHarvest(&self, _block: i32) -> bool { false }
    fn onCrafted(&self, _stack: ItemStack, _player: Player) {}
    /// needs api 1.20, how many ticks the item can be held in use, 0 for an item never held; onRightClick starts the
    /// use with Player::setItemInUse
    fn useDuration(&self, _stack: ItemStack) -> i32 { 0 }
    /// needs api 1.20, how the player holds it in use, a bow also narrows the view while it draws
    fn useAction(&self, _stack: ItemStack) -> UseAction { UseAction::None }
    /// needs api 1.20, every tick it is held in use, left ticks to go
    fn onUsingTick(&self, _world: World, _player: Player, _stack: ItemStack, _left: i32) {}
    /// needs api 1.20, the player let go with left ticks to go, as ItemBow.onPlayerStoppedUsing
    fn onStoppedUsing(&self, _world: World, _player: Player, _stack: ItemStack, _left: i32) -> Use { Use::Pass }
    /// needs api 1.20, the use ran its whole duration, as Item.onEaten
    fn onUseFinish(&self, _world: World, _player: Player, _stack: ItemStack) -> Use { Use::Pass }
    /// needs api 1.23, the item sheet slot shown while held in use for used ticks, None keeps the usual icon
    fn useIcon(&self, _stack: ItemStack, _used: i32) -> Option<i32> { None }
    /// needs api 1.33, the name one stack shows in place of the item's own, as a map or template numbers itself
    fn name(&self, _stack: ItemStack) -> Option<String> { None }
    fn overridden(&self) -> &'static [&'static str] { unmarked(std::any::type_name::<Self>(), "item_behavior") }
}

pub trait EntityBehavior
{
    fn onTick(&self, _entity: Entity, _info: &EntityInfo) {}
    fn onLivingUpdate(&self, _entity: Entity, _info: &EntityInfo) {}
    fn onInteract(&self, _entity: Entity, _player: Player) -> bool { false }
    /// None cancels the hit, Some changes the damage
    fn onHurt(&self, _entity: Entity, _source: Option<i32>, amount: i32) -> Option<i32> { Some(amount) }
    fn onDeath(&self, _entity: Entity, _source: Option<i32>) {}
    fn canSpawn(&self, _entity: Entity, _info: &EntityInfo) -> bool { true }
    /// client only, the draw program for this frame of the entity's spec model
    fn pose(&self, _input: &PoseInput) -> Pose { Pose::default() }
    /// implementing this replaces the template's AI, the entity only does what this does
    fn onAi(&self, _entity: Entity, _info: &EntityInfo) {}
    /// needs api 1.28, EntityLiving.moveEntityWithHeading with the entity's own or its rider's input; true when this
    /// moved it, false leaves the move to the template; Entity::accelerate, step and animateWalk build a move
    fn onTravel(&self, _entity: Entity, _info: &EntityInfo, _strafe: f32, _forward: f32) -> bool { false }
    fn overridden(&self) -> &'static [&'static str] { unmarked(std::any::type_name::<Self>(), "entity_behavior") }
}

pub trait TileBehavior
{
    fn onTick(&self, _ctx: &mut BlockCtx) {}
    fn onRemoved(&self, _ctx: &mut BlockCtx) {}
    /// needs api 1.20, the ops that draw the tile's registered model this frame, in block space from its lower corner
    fn pose(&self, _ctx: &mut BlockCtx, _partial: f32, _ticks: i32) -> Pose { Pose::default() }
    /// needs api 1.33, a stack offered from a face by a pipe, a machine or World::insertItem, the count left over;
    /// simulate changes nothing, and without this the tile's slots take it as a chest would
    fn insertItem(&self, _ctx: &mut BlockCtx, _face: i32, stack: ItemStack, _simulate: bool) -> i32 { stack.count }
    /// needs api 1.33, up to max items given out of a face; without this the first filled slot gives them
    fn extractItem(&self, _ctx: &mut BlockCtx, _face: i32, _max: i32, _simulate: bool) -> Option<ItemStack> { None }
    /// needs api 1.33, the energy kind the tile takes from a face and the packet sizes it takes now, None takes none;
    /// write it together with receiveEnergy
    fn energyDemand(&self, _ctx: &mut BlockCtx, _face: i32) -> Option<EnergyDemand> { None }
    /// needs api 1.33, energy pushed in from a face in the kind energyDemand named, the amount taken
    fn receiveEnergy(&self, _ctx: &mut BlockCtx, _face: i32, _amount: i64, _simulate: bool) -> i64 { 0 }
    /// needs api 1.33, fluid poured in from a face, the amount taken; write it together with drainFluid
    fn fillFluid(&self, _ctx: &mut BlockCtx, _face: i32, _stack: &FluidStack, _simulate: bool) -> i32 { 0 }
    /// needs api 1.33, up to max of the fluid given out of a face
    fn drainFluid(&self, _ctx: &mut BlockCtx, _face: i32, _max: i32, _simulate: bool) -> Option<FluidStack> { None }
    fn overridden(&self) -> &'static [&'static str] { unmarked(std::any::type_name::<Self>(), "tile_behavior") }
}

/// needs api 1.29, write the methods you need inside an impl marked #[effect_behavior]; onTick also runs on a client
/// for its own player, check World::isAuthority before changing the world
pub trait EffectBehavior
{
    fn onAdded(&self, _entity: Entity, _level: i32) {}
    fn onTick(&self, _entity: Entity, _level: i32, _ticks: i32) {}
    fn onRemoved(&self, _entity: Entity) {}
    fn overridden(&self) -> &'static [&'static str] { unmarked(std::any::type_name::<Self>(), "effect_behavior") }
}

pub(crate) const BLOCK_METHODS: [(&str, BlockMethod); 29] = [
    ("onActivated", BlockMethod::Activated),
    ("onClicked", BlockMethod::Clicked),
    ("onAdded", BlockMethod::Added),
    ("onRemoved", BlockMethod::Removed),
    ("onPlaced", BlockMethod::Placed),
    ("onPlacedBy", BlockMethod::PlacedBy),
    ("onNeighborChanged", BlockMethod::NeighborChanged),
    ("onUpdateTick", BlockMethod::UpdateTick),
    ("onRandomDisplayTick", BlockMethod::RandomDisplayTick),
    ("onEntityCollided", BlockMethod::EntityCollided),
    ("onEntityWalking", BlockMethod::EntityWalking),
    ("onDestroyedByPlayer", BlockMethod::DestroyedByPlayer),
    ("onDestroyedByExplosion", BlockMethod::DestroyedByExplosion),
    ("onHarvest", BlockMethod::Harvest),
    ("canPlaceAt", BlockMethod::CanPlaceAt),
    ("canStay", BlockMethod::CanStay),
    ("poweringTo", BlockMethod::PoweringTo),
    ("indirectlyPoweringTo", BlockMethod::IndirectlyPoweringTo),
    ("canProvidePower", BlockMethod::CanProvidePower),
    ("texture", BlockMethod::Texture),
    ("colorMultiplier", BlockMethod::ColorMultiplier),
    ("faceTexture", BlockMethod::FaceTexture),
    ("strength", BlockMethod::Strength),
    ("overlay", BlockMethod::Overlay),
    ("faceRotations", BlockMethod::FaceRotation),
    ("connections", BlockMethod::Connections),
    ("brightness", BlockMethod::Brightness),
    ("textureFor", BlockMethod::StateTexture),
    ("onBlockEvent", BlockMethod::BlockEvent),
];

pub(crate) const ITEM_METHODS: [(&str, ItemMethod); 16] = [
    ("onRightClick", ItemMethod::RightClick),
    ("onUseOnBlock", ItemMethod::UseOnBlock),
    ("onHitEntity", ItemMethod::HitEntity),
    ("onInteractEntity", ItemMethod::InteractEntity),
    ("onBlockDestroyed", ItemMethod::BlockDestroyed),
    ("icon", ItemMethod::Icon),
    ("strVsBlock", ItemMethod::StrVsBlock),
    ("canHarvest", ItemMethod::CanHarvest),
    ("onCrafted", ItemMethod::Crafted),
    ("useDuration", ItemMethod::UseDuration),
    ("useAction", ItemMethod::UseAction),
    ("onUsingTick", ItemMethod::UsingTick),
    ("onStoppedUsing", ItemMethod::StoppedUsing),
    ("onUseFinish", ItemMethod::UseFinish),
    ("useIcon", ItemMethod::UseIcon),
    ("name", ItemMethod::Name),
];

pub(crate) const ENTITY_METHODS: [(&str, EntityMethod); 9] = [
    ("onTick", EntityMethod::Tick),
    ("onLivingUpdate", EntityMethod::LivingUpdate),
    ("onInteract", EntityMethod::Interact),
    ("onHurt", EntityMethod::Hurt),
    ("onDeath", EntityMethod::Death),
    ("canSpawn", EntityMethod::CanSpawn),
    ("pose", EntityMethod::Pose),
    ("onAi", EntityMethod::Ai),
    ("onTravel", EntityMethod::Travel),
];

pub(crate) const TILE_METHODS: [(&str, TileMethod); 9] = [
    ("onTick", TileMethod::Tick),
    ("onRemoved", TileMethod::Removed),
    ("pose", TileMethod::Pose),
    ("insertItem", TileMethod::InsertItem),
    ("extractItem", TileMethod::ExtractItem),
    ("energyDemand", TileMethod::Energy),
    ("receiveEnergy", TileMethod::Energy),
    ("fillFluid", TileMethod::Fluid),
    ("drainFluid", TileMethod::Fluid),
];

pub(crate) const EFFECT_METHODS: [(&str, EffectMethod); 3] =
    [("onAdded", EffectMethod::Added), ("onTick", EffectMethod::Tick), ("onRemoved", EffectMethod::Removed)];

// the attribute macro replaces overridden, so this default only runs for an impl that forgot it
fn unmarked(name: &str, attribute: &str) -> &'static [&'static str]
{
    crate::warn(&format!("{name} has no #[{attribute}] attribute, the game calls none of its methods"));
    &[]
}

pub(crate) fn methods<M: Copy>(names: &[&str], table: &[(&str, M)]) -> Vec<M>
{
    table.iter().filter(|(name, _)| names.contains(name)).map(|(_, method)| *method).collect()
}
