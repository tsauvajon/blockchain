use crate::transaction::Transaction;
use crate::Hash;

/**
A block contains a number of transactions.
It is only valid in the context of the blockchain: its hash depends on the
previous block.

```
# use crate::blockchain::transaction::{Transaction, TransactionRecord};
# use crate::blockchain::block::Block;
# use std::str;

let mut block = Block::new();

let transaction = Transaction::new(5, TransactionRecord::CreateUserAccount("hi".into()), None);
block.transactions.push(transaction);

println!("{:02X?}", block.calculate_hash());
```
*/
#[derive(Debug)]
pub struct Block {
    /// All transactions contained in this block.
    pub transactions: Vec<Transaction>,

    /// Hash of the full block, i.e. hash all transactions hashes.
    pub hash: Option<Hash>,

    /// Hash of the previous block.
    pub previous_hash: Option<Hash>,
}

impl Block {
    /// Calculate the cryptographic hash of this block.
    pub fn calculate_hash(&self) -> Hash {
        self.transactions
            .iter()
            .fold(&mut blake3::Hasher::new(), |hasher, transaction| {
                hasher.update(&transaction.calculate_hash())
            })
            .finalize()
            .as_bytes()
            .to_vec()
    }

    /// Is this block's hash valid?
    pub fn is_hash_valid(&self) -> bool {
        match &self.hash {
            None => false,
            Some(hash) => *hash == self.calculate_hash(),
        }
    }

    /// Constructor
    pub fn new() -> Self {
        Block {
            transactions: vec![],
            hash: None,
            previous_hash: None,
        }
    }
}

impl Default for Block {
    fn default() -> Self {
        Self::new()
    }
}

#[test]
fn test_calculate_hash_is_deterministic() {
    let block1 = Block::new();
    let block2 = Block::new();
    assert_eq!(block1.calculate_hash(), block2.calculate_hash());
}

#[test]
fn test_calculate_hash_is_deterministic_with_transactions() {
    use crate::transaction::TransactionRecord;

    let mut block1 = Block::new();
    let mut block2 = Block::new();

    let transaction1 = Transaction::new(5, TransactionRecord::CreateUserAccount("hi".into()), None);
    let mut transaction2 =
        Transaction::new(5, TransactionRecord::CreateUserAccount("hi".into()), None);
    // make sure transactions are equal, even though that's
    // not what we're testing here
    transaction2.created_at = transaction1.created_at;
    assert_eq!(format!("{:?}", transaction1), format!("{:?}", transaction2));

    block1.transactions.push(transaction1);
    block2.transactions.push(transaction2);
    assert_eq!(block1.calculate_hash(), block2.calculate_hash())
}

#[test]
fn test_calculate_hash_does_not_collide() {
    use crate::transaction::TransactionRecord;

    let block1 = Block::new();
    let mut block2 = Block::new();

    block2.transactions.push(Transaction::new(
        5,
        TransactionRecord::CreateUserAccount("hi".into()),
        None,
    ));

    assert_ne!(block1.calculate_hash(), block2.calculate_hash());
}
