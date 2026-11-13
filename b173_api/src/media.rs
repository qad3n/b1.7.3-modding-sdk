use crate::reply::{expectInt, expectUnit};
use crate::rt::call;
use crate::{BlockPos, Player, Result, World};
use mod_protocol::abi::{HostReply, MediaLimit, MediaRequest, MediaStatus, Song};

/// needs api 2.1, starts downloading and compressing a song on the side that owns the world, the job to poll with
/// mediaStatus; text is a youtube link, any other link to a song, or words to search for
pub fn importMedia(text: &str, limit: MediaLimit) -> Result<i32>
{
    expectInt(call(MediaRequest::Import { text: text.to_owned(), limit }))
}

/// needs api 2.1, where an import stands, a finished one answers once and is then Unknown
pub fn mediaStatus(job: i32) -> MediaStatus
{
    match call(MediaRequest::Status { job })
    {
        HostReply::Media(status) => status,
        HostReply::Error(reason) => MediaStatus::Failed(reason),
        _ => MediaStatus::Unknown,
    }
}

/// needs api 2.1, plays a song from the jukebox at pos on this client in step with every other listener
pub fn playSong(world: World, pos: BlockPos, song: Song) -> Result<()>
{
    expectUnit(call(MediaRequest::Play { world: world.dimension, pos, song }))
}

/// needs api 2.1, puts the stack player holds into the empty jukebox at pos with its components
pub fn insertHeld(world: World, pos: BlockPos, player: Player) -> Result<()>
{
    expectUnit(call(MediaRequest::InsertHeld { world: world.dimension, pos, player: player.id }))
}
