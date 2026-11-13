use crate::abi::BlockPos;
use serde::{Deserialize, Serialize};

/// api 2.1: songs a mod brings in from the web, kept in the world that imported them and sent to the clients that play
/// them
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum MediaRequest
{
    /// downloads and compresses a song in the background, answered with Int, the job to ask Status about; text is a
    /// youtube link, any other link to a song, or words to search for
    Import
    {
        text: String, limit: MediaLimit
    },

    /// answered with Media, a finished job is forgotten once it was asked about
    Status
    {
        job: i32
    },

    /// plays a song from the jukebox at pos on this client as if it started at world time start, so every listener
    /// hears the same moment, fetching the file from the server first when this client lacks it
    Play
    {
        world: i32, pos: BlockPos, song: Song
    },

    /// moves the stack player holds into the empty jukebox at pos with its components, as a beta record goes in
    InsertHeld
    {
        world: i32, pos: BlockPos, player: i32
    },
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub struct MediaLimit
{
    pub seconds: i32,
    pub bytes: i64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Song
{
    pub hash: String,
    pub title: String,
    pub start: i64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum MediaStatus
{
    Running,
    Done(MediaInfo),
    Failed(String),
    Unknown,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct MediaInfo
{
    pub hash: String,
    pub title: String,
    pub seconds: i32,
    pub bytes: i64,
}
