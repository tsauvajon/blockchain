use std::collections::HashMap;
use std::time::SystemTime;

fn main() {
    println!("Hello, world!");
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
    /// It is the result of the proof of work.
    nonce: u64,

    /// What account initiated the transaction.
    from_account_id: String,

    /// What data is contained in the transaction.
    record: TransactionRecord,

    /// Signed hash of the transaction.
    signature: Option<String>,

    /// Local time of creation.
    created_at: SystemTime,
}

impl Transaction {
    fn calculate_hash(&self) -> Vec<u8> {
        blake3::hash(self.signature.as_ref().unwrap().as_bytes())
            .as_bytes()
            .to_vec()
    }
}

#[derive(Debug)]
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
trait WorldState {}

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

        let is_genesis = self.blocks.len() == 0;
        if is_genesis {
            self.blocks.push(block);
            return Ok(());
        }

        if block.previous_hash != self.get_last_block_hash() {
            Err("invalid previous hash".to_string())
        } else {
            self.blocks.push(block);
            Ok(())
        }
    }
}
