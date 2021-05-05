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

/// Module account contains implementation for accounts.
pub mod account;

/// Module block contains Block manipulation logic, including hashing.
pub mod block;

/**
 Module blockchain contains the general implementation of the Blockchain,
 including holding the overall state of the chain, chain manipulation etc.
*/
pub mod blockchain;

/// Module id can define and generate unique identifiers.
pub mod id;

/// Module transaction implements transactions: actions to apply, signature,
/// hash...
pub mod transaction;

/// Module world contains abstract definitions of the world state.
pub mod world;

/// An error message.
pub type Error = String;
/// The hash of some data.
pub type Hash = Vec<u8>;
/// A number that can only be used once.
pub type Nonce = u64;
