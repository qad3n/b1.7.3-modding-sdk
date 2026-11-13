pub mod abi;
#[cfg(feature = "data")]
pub mod data;
pub mod handshake;
pub mod identifier;
pub mod rules;
pub mod spec;
pub mod wire;

pub use identifier::Identifier;

/// mods declare api = "2.x" and load on any 2.y with y >= x
pub const API_MAJOR: u32 = 2;
pub const API_MINOR: u32 = 1;
