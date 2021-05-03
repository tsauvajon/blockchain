pub mod account;
pub mod block;
pub mod blockchain;
pub mod id;
pub mod transaction;
pub mod world;

pub type Error = String;
pub type Signature = String;
pub type Hash = Vec<u8>;
pub type Nonce = u64;
