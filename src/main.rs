use std::collections::HashMap;
use std::time::SystemTime;

fn main() -> Result<(), String> {
    let chain = Blockchain::new();
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

    fn apply<T: WorldState>(&self, world_state: &mut T) -> Result<(), String> {
        match &self.record {
            TransactionRecord::CreateUserAccount(id) => {
                world_state
                    .get_account_by_id(id.clone())
                    .map_or(Err("account already exists".to_string()), |_| Ok(()))?;
                world_state.add_account(id.to_owned())?;
                Ok(())
            }

            TransactionRecord::SendTokens { to, amount } => {
                let from = world_state
                    .get_account_by_id(
                        self.from_account_id
                            .as_ref()
                            .expect("missing from account")
                            .to_owned(),
                    )
                    .expect("from account doesn't exist");
                let to = world_state
                    .get_account_by_id(to.to_owned())
                    .expect("to account doesn't exist");

                if from.tokens < *amount {
                    return Err("not enough tokens".to_string());
                }

                from.tokens.checked_sub(*amount).expect("not enough tokens");
                to.tokens.checked_add(*amount).expect("too many tokens");

                Ok(())
            }

            TransactionRecord::MintTokens { to, amount } => match &self.from_account_id {
                Some(_) => Err("users cannot mint tokens".to_string()),
                None => {
                    if !world_state.is_genesis() {
                        return Err("cannot mint tokens after genesis".to_string());
                    }

                    let to = world_state.get_account_by_id(to.to_owned())?;
                    to.tokens.checked_add(*amount).expect("too many tokens");
                    Ok(())
                }
            },
        }
    }
}

#[test]
fn test_apply_transaction() {
    let mut chain = Blockchain::new();
    let transaction = Transaction::new(
        0,
        TransactionRecord::CreateUserAccount("someone".to_string()),
        None,
    );

    assert_eq!(Ok(()), transaction.apply(&mut chain));
}

#[derive(Debug, Clone)]
struct Account {
    /// Number of tokens held.
    tokens: u64,
}

impl Account {
    /// Constructor
    pub fn new() -> Self {
        return Self { tokens: 0 };
    }
}

/// Snapshot of the world, not to have to rebuild it every time we query it.
trait WorldState {
    fn get_account_by_id(&self, id: String) -> Result<&Account, String>;
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
    fn add_block(mut self, block: Block) -> Result<(), String> {
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

        // TODO: use an ECS architecture instead.
        for transaction in &block.transactions {
            if let Err(err) = transaction.apply(&mut self) {
                // roll back (this is super bad)
                self.accounts = previous_state;
                return Err(err);
            };
        }
        for (i, transaction) in block.transactions.iter().enumerate() {
            if let Err(err) = transaction.apply(&mut self) {
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

    fn get_account_by_id(&self, id: String) -> Result<&Account, String> {
        Ok(self
            .accounts
            .get(&id)
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
fn test_add_block() {
    let chain = Blockchain::new();
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
