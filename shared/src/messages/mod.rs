pub mod client;
pub mod macros;
pub mod server;

pub type SessionKey = [u8; 32];
pub type SessionId = i64;
pub type UserId = String;
