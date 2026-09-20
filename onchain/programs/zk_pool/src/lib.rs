pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use state::*;

declare_id!("CZjsHYcE6oPhj2ZAKVdnaz36FQHC7SBd7MSY1jmAaj3o");

#[program]
pub mod zk_pool {
    use super::*;
}
