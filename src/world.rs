use crate::account::Account;
use crate::id::Id;
use crate::types::Error;

/// Snapshot of the world, not to have to rebuild it every time we query it.
pub trait WorldState {
    fn get_account_by_id(&self, id: &Id) -> Result<&Account, Error>;
    fn get_account_by_id_mut(&mut self, id: &Id) -> Result<&mut Account, Error>;
    fn add_account(&mut self, id: Id) -> Result<(), Error>;

    fn is_genesis(&self) -> bool;
}
