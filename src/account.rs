/// Amount is a number of tokens.
pub type Amount = u64;

/**
An account can hold tokens.

```
# use crate::blockchain::account::Account;
let account = Account::new();
# println!("{:?}", account);
```
*/
#[derive(Debug, Clone)]
pub struct Account {
    /// Number of tokens held.
    pub tokens: Amount,
}

impl Account {
    /// Constructor
    pub fn new() -> Self {
        Self { tokens: 0 }
    }
}

impl Default for Account {
    fn default() -> Self {
        Self::new()
    }
}
