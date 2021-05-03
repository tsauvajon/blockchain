/*!
This is a basic Proof of Work blockchain implemented in Rust.
I started that project to remind myself how a basic blockchain
could work, and to learn more about Rust.
*/
#![deny(warnings, missing_docs, clippy::all, clippy::cargo)]

extern crate blockchain;

use blockchain::block::Block;
use blockchain::blockchain::Blockchain;
use blockchain::transaction::{Transaction, TransactionRecord};
use blockchain::Error;

#[cfg(not(tarpaulin_include))]
fn main() -> Result<(), Error> {
    let mut chain = Blockchain::new();
    let mut block = Block::new();

    block.transactions.push(Transaction::new(
        0,
        TransactionRecord::CreateUserAccount("someone".into()),
        None,
    ));

    block.transactions.push(Transaction::new(
        0,
        TransactionRecord::CreateUserAccount("someone else".into()),
        None,
    ));

    block.transactions.push(Transaction::new(
        0,
        TransactionRecord::MintTokens {
            to: "someone".into(),
            amount: 400,
        },
        None,
    ));

    block.transactions.push(Transaction::new(
        0,
        TransactionRecord::SendTokens {
            to: "someone else".into(),
            amount: 200,
        },
        Some("someone".into()),
    ));

    block.hash = Some(block.calculate_hash());
    chain.add_block(block)
}
