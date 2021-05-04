use crate::account::Account;
use crate::block::Block;
use crate::id::Id;
use crate::transaction::Transaction;
use crate::world::WorldState;
use crate::{Error, Hash};
use std::collections::HashMap;

/// Contains the state of the blockchain.
#[derive(Debug)]
pub struct Blockchain {
    /// All the blocks composing the blockchain.
    blocks: Vec<Block>,

    /// All accounts, it is the current "world state".
    accounts: HashMap<Id, Account>,

    /// In-progress transactions.
    pending_transactions: Vec<Transaction>,
}

impl Blockchain {
    /// Get the hash of the last block in the chain.
    fn get_last_block_hash(&self) -> Option<Hash> {
        self.blocks.last()?.hash.clone()
    }

    /// If the block is correct, add it to the chain.
    pub fn add_block(&mut self, block: Block) -> Result<(), Error> {
        if !block.is_hash_valid() {
            return Err("invalid hash".to_string());
        }

        if self.is_genesis() {
            self.blocks.push(block);
            return Ok(());
        }

        if block.previous_hash != self.get_last_block_hash() {
            return Err("invalid previous hash".to_string());
        }

        let previous_state = self.accounts.clone();
        for (i, transaction) in block.transactions.iter().enumerate() {
            if let Err(err) = transaction.apply(self) {
                // roll back (this is super bad)
                self.accounts = previous_state;
                return Err(format! {"err {:?} on transaction {:?}", err, i});
            };
        }

        self.blocks.push(block);
        Ok(())
    }

    pub fn new() -> Self {
        Blockchain {
            blocks: vec![],
            accounts: HashMap::new(),
            pending_transactions: vec![],
        }
    }
}

impl Default for Blockchain {
    fn default() -> Self {
        Self::new()
    }
}

impl WorldState for Blockchain {
    fn is_genesis(&self) -> bool {
        self.blocks.is_empty()
    }

    fn get_account_by_id(&self, id: &Id) -> Result<&Account, Error> {
        self.accounts
            .get(id)
            .ok_or_else(|| "account doesn't exist".to_string())
    }

    fn get_account_by_id_mut(&mut self, id: &Id) -> Result<&mut Account, Error> {
        self.accounts
            .get_mut(id)
            .ok_or_else(|| "account doesn't exist".to_string())
    }

    fn add_account(&mut self, id: Id) -> Result<(), Error> {
        if let std::collections::hash_map::Entry::Vacant(accounts) = self.accounts.entry(id) {
            accounts.insert(Account::new());
            Ok(())
        } else {
            Err("account already exists".to_string())
        }
    }
}

#[test]
fn test_add_block() {
    use crate::transaction::TransactionRecord;
    use std::time::SystemTime;

    let mut chain = Blockchain::new();
    let mut block = Block::new();

    block.transactions.push(Transaction {
        nonce: 0,
        from_account_id: Some("hello".into()),
        record: TransactionRecord::CreateUserAccount("world".into()),
        signature: Some("signature".to_string()),
        created_at: SystemTime::now(),
    });
    block.hash = Some(block.calculate_hash());

    assert_eq!(Ok(()), chain.add_block(block))
}

#[test]
fn test_cannot_create_duplicate_accounts() {
    let mut chain = Blockchain::new();

    chain.add_account("someone".into()).unwrap();
    assert_eq!(
        Err("account already exists".to_string()),
        chain.add_account("someone".into())
    )
}
