use crate::{Entity, ItemStack, Player, World};
use mod_protocol::Identifier;
pub use mod_protocol::abi::EventData;
use mod_protocol::abi::{BlockPos, EventEnvelope, EventKind, StackData};

/// one fired event, listeners read it, cancel it, set its result or change its data
pub struct Event
{
    pub(crate) m_envelope: EventEnvelope,
}

impl Event
{
    pub fn kind(&self) -> EventKind { self.m_envelope.kind }

    pub fn data(&self) -> &EventData { &self.m_envelope.data }

    pub fn dataMut(&mut self) -> &mut EventData { &mut self.m_envelope.data }

    pub fn cancel(&mut self) { self.m_envelope.canceled = true; }

    pub fn isCanceled(&self) -> bool { self.m_envelope.canceled }

    pub fn setResult(&mut self, result: i32) { self.m_envelope.result = Some(result); }

    pub fn result(&self) -> Option<i32> { self.m_envelope.result }

    pub fn world(&self) -> Option<World>
    {
        let dimension: i32 = match &self.m_envelope.data
        {
            EventData::World { world }
            | EventData::Chunk { world, .. }
            | EventData::Block { world, .. }
            | EventData::Neighbor { world, .. }
            | EventData::Drops { world, .. }
            | EventData::ItemUse { world, .. }
            | EventData::Item { world, .. }
            | EventData::Player { world, .. }
            | EventData::BreakSpeed { world, .. } => *world,
            EventData::Entity { entity }
            | EventData::Attack { entity, .. }
            | EventData::Death { entity, .. }
            | EventData::Spawn { entity, .. } => entity.world,
            _ => return None,
        };
        Some(World { dimension })
    }

    pub fn player(&self) -> Option<Player>
    {
        match &self.m_envelope.data
        {
            EventData::Block { player, .. } => player.map(|id| Player { id }),
            EventData::ItemUse { player, .. }
            | EventData::Item { player, .. }
            | EventData::Player { player, .. }
            | EventData::Dimension { player, .. }
            | EventData::Chat { player, .. }
            | EventData::BreakSpeed { player, .. }
            | EventData::Harvest { player, .. } => Some(Player { id: *player }),
            _ => None,
        }
    }

    pub fn entity(&self) -> Option<Entity>
    {
        match &self.m_envelope.data
        {
            EventData::Entity { entity }
            | EventData::Attack { entity, .. }
            | EventData::Death { entity, .. }
            | EventData::Spawn { entity, .. } =>
            {
                Some(Entity { world: World { dimension: entity.world }, id: entity.id })
            }

            _ => None,
        }
    }

    pub fn entityKind(&self) -> Option<Identifier>
    {
        match &self.m_envelope.data
        {
            EventData::Entity { entity }
            | EventData::Attack { entity, .. }
            | EventData::Death { entity, .. }
            | EventData::Spawn { entity, .. } => entity.kind.clone(),
            _ => None,
        }
    }

    pub fn pos(&self) -> Option<BlockPos>
    {
        match &self.m_envelope.data
        {
            EventData::Block { pos, .. }
            | EventData::Neighbor { pos, .. }
            | EventData::Drops { pos, .. }
            | EventData::BreakSpeed { pos, .. } => Some(*pos),
            EventData::ItemUse { pos, .. } => *pos,
            _ => None,
        }
    }

    /// adds a drop to a harvest or death event
    pub fn addDrop(&mut self, stack: ItemStack)
    {
        let Ok(raw) = stack.toRaw()
        else
        {
            return;
        };

        if let EventData::Drops { drops, .. } | EventData::Death { drops, .. } = &mut self.m_envelope.data
        {
            drops.push(raw);
        }
    }

    pub fn setAmount(&mut self, value: i32)
    {
        if let EventData::Attack { amount, .. } = &mut self.m_envelope.data
        {
            *amount = value;
        }
    }

    pub fn setSpeed(&mut self, value: f32)
    {
        if let EventData::BreakSpeed { speed, .. } = &mut self.m_envelope.data
        {
            *speed = value;
        }
    }

    pub fn setMessage(&mut self, text: &str)
    {
        if let EventData::Chat { message, .. } = &mut self.m_envelope.data
        {
            *message = text.to_owned();
        }
    }

    pub fn stack(&self) -> Option<ItemStack>
    {
        let raw: &StackData = match &self.m_envelope.data
        {
            EventData::Item { stack, .. } | EventData::ItemUse { stack: Some(stack), .. } => stack,
            _ => return None,
        };
        Some(ItemStack::fromRaw(raw.clone()))
    }

    pub fn custom(&self) -> Option<(&Identifier, &[u8])>
    {
        match &self.m_envelope.data
        {
            EventData::Custom { event, data } => Some((event, data)),
            _ => None,
        }
    }
}

pub trait EventType
{
    const KIND: EventKind;
}

macro_rules! event_types
{
    ($($name:ident,)*) =>
    {
        $(
            pub struct $name;

            impl EventType for $name
            {
                const KIND: EventKind = EventKind::$name;
            }
        )*
    };
}

event_types!
{
    Setup, ClientSetup, ServerSetup, LoadComplete, Shutdown, ServerStarting, ServerStarted, ServerStopping, TickPre, TickPost,
    WorldLoad, WorldSave, WorldUnload, ChunkLoad, ChunkUnload, ChunkGenerate, ChunkPopulate, BlockPlace, BlockBreak,
    BlockHarvestDrops, BlockActivate, NeighborChange, ExplosionAffect, ItemUse, ItemUseFirst, ItemUseOnBlock, ItemDestroyed,
    ItemPickup, ItemToss, EntityJoinWorld, EntityTick, LivingAttack, LivingHurt, LivingDeath, LivingDrops, LivingSpawnCheck,
    PlayerLogin, PlayerLogout, PlayerRespawn, PlayerChangeDimension, PlayerSleep, PlayerChat, PlayerInteract, PlayerBreakSpeed,
    PlayerCanHarvest, ItemCrafted, ItemSmelted, BucketFill, RenderWorldLast, RenderHud, RenderBlockHighlight, ScreenOpen,
    KeyInput, ClientTick, EntityRemove, LightningStrike,
}
