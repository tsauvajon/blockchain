use crate::account::Account;
use crate::id::Id;
use crate::Error;

/// Snapshot of the world, not to have to rebuild it every time we query it.
pub trait WorldState {
    /// Return an account that exists in the world, by its ID.
    fn get_account_by_id(&self, id: &Id) -> Result<&Account, Error>;
    /// Return a mutable reference to an account that exists in the world, by its ID.
    fn get_account_by_id_mut(&mut self, id: &Id) -> Result<&mut Account, Error>;
    /// Register a new account in the world.
    fn add_account(&mut self, id: Id) -> Result<(), Error>;

    /// Is the world in its genesis, i.e. are we currently creating that world?
    fn is_genesis(&self) -> bool;
}
