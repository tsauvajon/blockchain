use crate::types::Amount;

#[derive(Debug, Clone)]
pub struct Account {
    /// Number of tokens held.
    pub tokens: Amount,
}

impl Account {
    /// Constructor
    pub fn new() -> Self {
        return Self { tokens: 0 };
    }
}