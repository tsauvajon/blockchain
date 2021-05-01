/*!
This is a basic Proof of Work blockchain implemented in Rust.
I started that project to remind myself how a basic blockchain
could work, and to learn more about Rust.
*/

#![deny(warnings, missing_docs, clippy::all, clippy::cargo)]

use std::collections::HashMap;
use std::time::SystemTime;

#[derive(Debug)]
struct Block {
    // All transactions contained in this block.
    transactions: Vec<Transaction>,

    /// Hash of the full block, i.e. hash all transactions hashes.
    hash: Option<Vec<u8>>,

    /// Hash of the previous block.
    previous_hash: Option<Vec<u8>>,
}

impl Block {
    fn calculate_hash(&self) -> Vec<u8> {
        self.transactions
            .iter()
            .fold(&mut blake3::Hasher::new(), |hasher, transaction| {
                hasher.update(&transaction.calculate_hash())
            })
            .finalize()
            .as_bytes()
            .to_vec()
    }

    fn is_hash_valid(&self) -> bool {
        match &self.hash {
            None => false,
            Some(hash) => *hash == self.calculate_hash(),
        }
    }

    fn new() -> Self {
        Block {
            transactions: vec![],
            hash: None,
            previous_hash: None,
        }
    }
}

#[derive(Debug)]
enum TransactionRecord {
    /// Creates a new account from a public key.
    CreateUserAccount(String),

    /// Sends tokens to another account.
    SendTokens { to: String, amount: u64 },

    /// Create new tokens.
    MintTokens { to: String, amount: u64 },
}

/// A change of state in the blockchain.
#[derive(Debug)]
struct Transaction {
    /// "number only used once".
    nonce: u64,

    /// What account initiated the transaction.
    from_account_id: Option<String>,

    /// What data is contained in the transaction.
    record: TransactionRecord,

    /// Signed hash of the transaction.
    signature: Option<String>,

    /// Local time of creation.
    created_at: SystemTime,
}

impl Transaction {
    fn new(nonce: u64, record: TransactionRecord, from: Option<String>) -> Self {
        Transaction {
            nonce,
            from_account_id: from,
            record,
            signature: None,
            created_at: SystemTime::now(),
        }
    }

    fn calculate_hash(&self) -> Vec<u8> {
        blake3::hash(
            format!(
                "{:?}_{:?}_{:?}_{:?}",
                self.record, self.nonce, self.from_account_id, self.created_at,
            )
            .as_bytes(),
        )
        .as_bytes()
        .to_vec()
    }

    // TODO: use a TransactionRecord trait for better polymorphism.
    fn apply<T: WorldState>(&self, world_state: &mut T) -> Result<(), String> {
        match &self.record {
            TransactionRecord::CreateUserAccount(id) => {
                world_state
                    .get_account_by_id(id)
                    .map_or(Ok(()), |_| Err("account already exists".to_string()))?;
                world_state.add_account(id.to_string())?;
                Ok(())
            }

            TransactionRecord::MintTokens { to, amount } => match &self.from_account_id {
                Some(_) => Err("users cannot mint tokens".to_string()),
                None => {
                    if !world_state.is_genesis() {
                        return Err("cannot mint tokens after genesis".to_string());
                    }

                    let to_acc = world_state.get_account_by_id_mut(to)?;

                    println!("minting {:?} tokens for {:?}", amount, to);
                    to_acc.tokens = to_acc
                        .tokens
                        .checked_add(*amount)
                        .ok_or("too many tokens")?;
                    Ok(())
                }
            },

            TransactionRecord::SendTokens { to, amount } => {
                let from = world_state
                    .get_account_by_id_mut(
                        self.from_account_id
                            .as_ref()
                            .ok_or("missing from account")?,
                    )
                    .map_err(|_| "from account doesn't exist")?;
                from.tokens = from
                    .tokens
                    .checked_sub(*amount)
                    .ok_or("not enough tokens")?;

                let to = world_state
                    .get_account_by_id_mut(to)
                    .map_err(|_| "to account doesn't exist")?;
                to.tokens = to.tokens.checked_add(*amount).ok_or("too many tokens")?;

                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Account {
    /// Number of tokens held.
    tokens: u64,
}

impl Account {
    /// Constructor
    fn new() -> Self {
        return Self { tokens: 0 };
    }
}

/// Snapshot of the world, not to have to rebuild it every time we query it.
trait WorldState {
    fn get_account_by_id(&self, id: &String) -> Result<&Account, String>;
    fn get_account_by_id_mut(&mut self, id: &String) -> Result<&mut Account, String>;
    fn add_account(&mut self, id: String) -> Result<(), String>;

    fn is_genesis(&self) -> bool;
}

/// Contains the state of the blockchain.
#[derive(Debug)]
struct Blockchain {
    /// All the blocks composing the blockchain.
    blocks: Vec<Block>,

    /// All accounts, it is the current "world state".
    accounts: HashMap<String, Account>,

    /// In-progress transactions.
    pending_transactions: Vec<Transaction>,
}

impl Blockchain {
    /// Get the hash of the last block in the chain.
    fn get_last_block_hash(&self) -> Option<Vec<u8>> {
        self.blocks.last()?.hash.clone()
    }

    /// If the block is correct, add it to the chain.
    fn add_block(&mut self, block: Block) -> Result<(), String> {
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

    fn new() -> Self {
        Blockchain {
            blocks: vec![],
            accounts: HashMap::new(),
            pending_transactions: vec![],
        }
    }
}

impl WorldState for Blockchain {
    fn is_genesis(&self) -> bool {
        self.blocks.len() == 0
    }

    fn get_account_by_id(&self, id: &String) -> Result<&Account, String> {
        Ok(self
            .accounts
            .get(id)
            .ok_or("account doesn't exist".to_string())?)
    }

    fn get_account_by_id_mut(&mut self, id: &String) -> Result<&mut Account, String> {
        Ok(self
            .accounts
            .get_mut(id)
            .ok_or("account doesn't exist".to_string())?)
    }

    fn add_account(&mut self, id: String) -> Result<(), String> {
        if self.accounts.contains_key(&id) {
            Err("account already exists".to_string())
        } else {
            self.accounts.insert(id, Account::new());
            Ok(())
        }
    }
}

#[test]
fn test_calculate_block_hash_is_deterministic() {
    let block1 = Block::new();
    let block2 = Block::new();
    assert_eq!(block1.calculate_hash(), block2.calculate_hash());
}

#[test]
fn test_calculate_block_hash_is_deterministic_with_transactions() {
    let mut block1 = Block::new();
    let mut block2 = Block::new();

    let transaction1 = Transaction::new(
        5,
        TransactionRecord::CreateUserAccount("hi".to_string()),
        None,
    );
    let mut transaction2 = Transaction::new(
        5,
        TransactionRecord::CreateUserAccount("hi".to_string()),
        None,
    );
    // make sure transactions are equal, even though that's
    // not what we're testing here
    transaction2.created_at = transaction1.created_at;
    assert_eq!(format!("{:?}", transaction1), format!("{:?}", transaction2));

    block1.transactions.push(transaction1);
    block2.transactions.push(transaction2);
    assert_eq!(block1.calculate_hash(), block2.calculate_hash())
}

#[test]
fn test_calculate_block_hash_does_not_collide() {
    let block1 = Block::new();
    let mut block2 = Block::new();

    block2.transactions.push(Transaction::new(
        5,
        TransactionRecord::CreateUserAccount("hi".to_string()),
        None,
    ));

    assert_ne!(block1.calculate_hash(), block2.calculate_hash());
}

#[test]
fn test_add_block() {
    let mut chain = Blockchain::new();
    let mut block = Block::new();

    block.transactions.push(Transaction {
        nonce: 0,
        from_account_id: Some("hello".to_string()),
        record: TransactionRecord::CreateUserAccount("world".to_string()),
        signature: Some("signature".to_string()),
        created_at: SystemTime::now(),
    });
    block.hash = Some(block.calculate_hash());

    assert_eq!(Ok(()), chain.add_block(block))
}

#[cfg(test)]
mod transaction_tests {
    use super::*;

    fn create_user(world_state: &mut impl WorldState, id: &str) -> Result<(), String> {
        let transaction = Transaction::new(
            0,
            TransactionRecord::CreateUserAccount(id.to_string()),
            None,
        );
        transaction.apply(world_state)
    }

    fn mint_tokens(world_state: &mut impl WorldState, id: &str, amount: u64) -> Result<(), String> {
        let transaction = Transaction::new(
            0,
            TransactionRecord::MintTokens {
                to: id.to_string(),
                amount,
            },
            None,
        );
        transaction.apply(world_state)
    }

    fn send_tokens(
        world_state: &mut impl WorldState,
        from: &str,
        id: &str,
        amount: u64,
    ) -> Result<(), String> {
        let transaction = Transaction::new(
            0,
            TransactionRecord::SendTokens {
                to: id.to_string(),
                amount,
            },
            Some(from.to_string()),
        );
        transaction.apply(world_state)
    }

    #[test]
    fn test_apply_create_user() {
        let mut chain = Blockchain::new();
        assert_eq!(Ok(()), create_user(&mut chain, "someone"));
    }

    #[test]
    fn test_apply_create_already_existing_user() {
        let mut chain = Blockchain::new();
        create_user(&mut chain, "someone").unwrap();

        assert_eq!(
            Err("account already exists".to_string()),
            create_user(&mut chain, "someone")
        );
    }

    #[test]
    fn test_apply_mint() {
        let mut chain = Blockchain::new();
        let account_id = "someone";

        create_user(&mut chain, account_id).unwrap();

        assert_eq!(Ok(()), mint_tokens(&mut chain, account_id, 200));

        let account = chain.get_account_by_id(&account_id.to_string()).unwrap();
        assert_eq!(200, account.tokens)
    }

    #[test]
    fn test_apply_mint_missing_user() {
        let mut chain = Blockchain::new();
        assert_eq!(
            Err("account doesn't exist".to_string()),
            mint_tokens(&mut chain, "I don't exist", 200),
        );
    }

    #[test]
    fn test_prevent_using_minting() {
        let mut chain = Blockchain::new();
        let account_id = "someone";

        create_user(&mut chain, account_id).unwrap();

        let transaction = Transaction::new(
            1,
            TransactionRecord::MintTokens {
                to: account_id.to_string(),
                amount: 200,
            },
            Some(account_id.to_string()),
        );
        assert_eq!(
            Err("users cannot mint tokens".to_string()),
            transaction.apply(&mut chain),
        );
    }

    #[test]
    fn test_apply_mint_not_genesis() {
        let mut chain = Blockchain::new();

        let mut block = Block::new();
        block.hash = Some(block.calculate_hash());
        chain.add_block(block).unwrap();

        create_user(&mut chain, "someone").unwrap();
        assert_eq!(
            Err("cannot mint tokens after genesis".to_string()),
            mint_tokens(&mut chain, "someone", 200)
        );
    }

    #[test]
    fn test_send_tokens() {
        let mut chain = Blockchain::new();

        create_user(&mut chain, "sender").unwrap();
        create_user(&mut chain, "receiver").unwrap();
        mint_tokens(&mut chain, "sender", 200).unwrap();

        let res = send_tokens(&mut chain, "sender", "receiver", 180);
        assert_eq!(Ok(()), res);

        let sender = chain.get_account_by_id(&"sender".to_string()).unwrap();
        assert_eq!(20, sender.tokens);

        let receiver = chain.get_account_by_id(&"receiver".to_string()).unwrap();
        assert_eq!(180, receiver.tokens);
    }

    #[test]
    fn test_send_tokens_not_enough_tokens() {
        let mut chain = Blockchain::new();

        create_user(&mut chain, "sender").unwrap();
        create_user(&mut chain, "receiver").unwrap();
        mint_tokens(&mut chain, "sender", 200).unwrap();

        let res = send_tokens(&mut chain, "sender", "receiver", 5000);
        assert_eq!(Err("not enough tokens".to_string()), res);
    }

    #[test]
    fn test_send_tokens_overflow() {
        let mut chain = Blockchain::new();

        create_user(&mut chain, "sender").unwrap();
        mint_tokens(&mut chain, "sender", u64::MAX).unwrap();
        create_user(&mut chain, "receiver").unwrap();
        mint_tokens(&mut chain, "receiver", u64::MAX).unwrap();

        let res = send_tokens(&mut chain, "sender", "receiver", 5000);
        assert_eq!(Err("too many tokens".to_string()), res);
    }
}

#[test]
fn test_cannot_create_duplicate_accounts() {
    let mut chain = Blockchain::new();

    chain.add_account("someone".to_string()).unwrap();
    assert_eq!(
        Err("account already exists".to_string()),
        chain.add_account("someone".to_string())
    )
}

#[cfg(not(tarpaulin_include))]
fn main() -> Result<(), String> {
    let mut chain = Blockchain::new();
    let mut block = Block::new();

    block.transactions.push(Transaction::new(
        0,
        TransactionRecord::CreateUserAccount("someone".to_string()),
        None,
    ));

    block.transactions.push(Transaction::new(
        0,
        TransactionRecord::CreateUserAccount("someone else".to_string()),
        None,
    ));

    block.transactions.push(Transaction::new(
        0,
        TransactionRecord::MintTokens {
            to: "someone".to_string(),
            amount: 400,
        },
        None,
    ));

    block.transactions.push(Transaction::new(
        0,
        TransactionRecord::SendTokens {
            to: "someone else".to_string(),
            amount: 200,
        },
        Some("someone".to_string()),
    ));

    block.hash = Some(block.calculate_hash());
    chain.add_block(block)
}
