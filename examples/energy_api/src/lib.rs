// the api crate convention: a provider mod publishes its names and message types as <modid>_api, consumers depend on it
use serde::{Deserialize, Serialize};

pub const SERVICE: &str = "energy:network";
pub const LOOKUP: &str = "energy:storage";
pub const CHANGED_EVENT: &str = "energy:changed";
pub const VERSION: u32 = 1;

pub mod methods
{
    pub const TOTAL: &str = "total";
    pub const STORED: &str = "stored";
    pub const INSERT: &str = "insert";
    pub const EXTRACT: &str = "extract";
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Amount(pub i64);

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Changed
{
    pub world: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub stored: Amount,
}
