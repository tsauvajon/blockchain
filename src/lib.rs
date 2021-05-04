/*!
This is a basic Proof of Work blockchain implemented in Rust.
I started that project to remind myself how a basic blockchain
could work, and to learn more about Rust.
*/
#![deny(
    warnings,
    missing_doc_code_examples,
    missing_docs,
    clippy::all,
    clippy::cargo
)]

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
