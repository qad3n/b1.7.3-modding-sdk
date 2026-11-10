use crate::{BlockCtx, Entity, EntityInfo, ItemStack, Player, World};
use mod_protocol::abi::{BlockPos, Pose, PoseInput};
use mod_protocol::spec::{BlockMethod, EntityMethod, ItemMethod, TileMethod};

// write the methods you need inside an impl marked #[block_behavior], only those cross from the game into wasm
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
    // needs api 1.12, a face's sprite slot from where the block stands, as Block.getBlockTexture; None keeps texture
    fn faceTexture(&self, _ctx: &mut BlockCtx, _face: i32) -> Option<i32> { None }
    fn overridden(&self) -> &'static [&'static str] { unmarked(std::any::type_name::<Self>(), "block_behavior") }
}

// a held item use either passes through or replaces the stack in hand
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
    fn overridden(&self) -> &'static [&'static str] { unmarked(std::any::type_name::<Self>(), "item_behavior") }
}

pub trait EntityBehavior
{
    fn onTick(&self, _entity: Entity, _info: &EntityInfo) {}
    fn onLivingUpdate(&self, _entity: Entity, _info: &EntityInfo) {}
    fn onInteract(&self, _entity: Entity, _player: Player) -> bool { false }
    // None cancels the hit, Some changes the damage
    fn onHurt(&self, _entity: Entity, _source: Option<i32>, amount: i32) -> Option<i32> { Some(amount) }
    fn onDeath(&self, _entity: Entity, _source: Option<i32>) {}
    fn canSpawn(&self, _entity: Entity, _info: &EntityInfo) -> bool { true }
    // client only, the draw program for this frame of the entity's spec model
    fn pose(&self, _input: &PoseInput) -> Pose { Pose::default() }
    // implementing this replaces the template's AI, the entity only does what this does
    fn onAi(&self, _entity: Entity, _info: &EntityInfo) {}
    fn overridden(&self) -> &'static [&'static str] { unmarked(std::any::type_name::<Self>(), "entity_behavior") }
}

pub trait TileBehavior
{
    fn onTick(&self, _ctx: &mut BlockCtx) {}
    fn onRemoved(&self, _ctx: &mut BlockCtx) {}
    fn overridden(&self) -> &'static [&'static str] { unmarked(std::any::type_name::<Self>(), "tile_behavior") }
}

pub(crate) const BLOCK_METHODS: [(&str, BlockMethod); 22] = [
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
];

pub(crate) const ITEM_METHODS: [(&str, ItemMethod); 9] = [
    ("onRightClick", ItemMethod::RightClick),
    ("onUseOnBlock", ItemMethod::UseOnBlock),
    ("onHitEntity", ItemMethod::HitEntity),
    ("onInteractEntity", ItemMethod::InteractEntity),
    ("onBlockDestroyed", ItemMethod::BlockDestroyed),
    ("icon", ItemMethod::Icon),
    ("strVsBlock", ItemMethod::StrVsBlock),
    ("canHarvest", ItemMethod::CanHarvest),
    ("onCrafted", ItemMethod::Crafted),
];

pub(crate) const ENTITY_METHODS: [(&str, EntityMethod); 8] = [
    ("onTick", EntityMethod::Tick),
    ("onLivingUpdate", EntityMethod::LivingUpdate),
    ("onInteract", EntityMethod::Interact),
    ("onHurt", EntityMethod::Hurt),
    ("onDeath", EntityMethod::Death),
    ("canSpawn", EntityMethod::CanSpawn),
    ("pose", EntityMethod::Pose),
    ("onAi", EntityMethod::Ai),
];

pub(crate) const TILE_METHODS: [(&str, TileMethod); 2] =
    [("onTick", TileMethod::Tick), ("onRemoved", TileMethod::Removed)];

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
