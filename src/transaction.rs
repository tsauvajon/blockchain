use crate::account::Amount;
use crate::id::Id;
use crate::world::WorldState;
use crate::{Error, Hash, Nonce};
use std::time::SystemTime;

/// The cryptographic signature of a transaction.
pub type Signature = String;

/// A transaction record is describing the action a transaction
/// executes against the Blockchain.
#[derive(Debug)]
pub enum TransactionRecord {
    /// Creates a new account from a public key.
    CreateUserAccount(Id),

    /// Sends tokens to another account.
    SendTokens {
        /// ID of the account receiving the tokens.
        to: Id,
        /// Number of tokens to send.
        amount: Amount,
    },

    /// Create new tokens.
    MintTokens {
        /// ID of the account receiving the tokens.
        to: Id,
        /// Number of tokens to send.
        amount: Amount,
    },
}

/** A change of state in the blockchain.

```
# use crate::blockchain::transaction::{Transaction, TransactionRecord};
# use crate::blockchain::blockchain::Blockchain;
# let mut blockchain = Blockchain::new();

let id = "some unique ID";
let transaction = Transaction::new(0, TransactionRecord::CreateUserAccount(id.into()), None);

transaction.apply(&mut blockchain);
```
*/
#[derive(Debug)]
pub struct Transaction {
    /// "number only used once".
    pub nonce: Nonce,

    /// What account initiated the transaction.
    pub from_account_id: Option<Id>,

    /// What data is contained in the transaction.
    pub record: TransactionRecord,

    /// Signed hash of the transaction.
    pub signature: Option<Signature>,

    /// Local time of creation.
    pub created_at: SystemTime,
}

impl Transaction {
    /// Constructor
    pub fn new(nonce: Nonce, record: TransactionRecord, from: Option<Id>) -> Self {
        Transaction {
            nonce,
            from_account_id: from,
            record,
            signature: None,
            created_at: SystemTime::now(),
        }
    }

    /// Calculate the cryptographic hash of this transaction.
    pub fn calculate_hash(&self) -> Hash {
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

    /// Execute this transaction against the Blockchain.
    /// TODO: use a TransactionRecord trait for better polymorphism.
    pub fn apply<T: WorldState>(&self, world_state: &mut T) -> Result<(), Error> {
        match &self.record {
            TransactionRecord::CreateUserAccount(id) => {
                world_state
                    .get_account_by_id(id)
                    .map_or(Ok(()), |_| Err("account already exists".to_string()))?;
                world_state.add_account(id.to_owned())?;
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

#[cfg(test)]
mod transaction_tests {
    use super::*;
    use crate::block::Block;
    use crate::blockchain::Blockchain;

    fn create_user(world_state: &mut impl WorldState, id: &str) -> Result<(), Error> {
        let transaction =
            Transaction::new(0, TransactionRecord::CreateUserAccount(id.into()), None);
        transaction.apply(world_state)
    }

    fn mint_tokens(
        world_state: &mut impl WorldState,
        id: &str,
        amount: Amount,
    ) -> Result<(), Error> {
        let transaction = Transaction::new(
            0,
            TransactionRecord::MintTokens {
                to: id.into(),
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
        amount: Amount,
    ) -> Result<(), Error> {
        let transaction = Transaction::new(
            0,
            TransactionRecord::SendTokens {
                to: id.into(),
                amount,
            },
            Some(from.into()),
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

        let account = chain.get_account_by_id(&account_id.into()).unwrap();
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
                to: account_id.into(),
                amount: 200,
            },
            Some(account_id.into()),
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

        let sender = chain.get_account_by_id(&"sender".into()).unwrap();
        assert_eq!(20, sender.tokens);

        let receiver = chain.get_account_by_id(&"receiver".into()).unwrap();
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
        mint_tokens(&mut chain, "sender", Amount::MAX).unwrap();
        create_user(&mut chain, "receiver").unwrap();
        mint_tokens(&mut chain, "receiver", Amount::MAX).unwrap();

        let res = send_tokens(&mut chain, "sender", "receiver", 5000);
        assert_eq!(Err("too many tokens".to_string()), res);
    }
}
