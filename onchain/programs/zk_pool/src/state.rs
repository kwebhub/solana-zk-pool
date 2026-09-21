use crate::constants::ROOT_HISTORY_SIZE;
use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct PoolState {
    pub authority: Pubkey,
    pub next_leaf_index: u64,
    pub total_deposits: u64,
    pub current_root_index: u64,
    pub roots: [[u8; 32]; ROOT_HISTORY_SIZE],
}

impl PoolState {
    pub fn is_known_root(&self, root: &[u8; 32]) -> bool {
        self.roots.iter().any(|r| r == root)
    }

    pub fn add_root(&mut self, new_root: [u8; 32]) {
        let index = ((self.current_root_index + 1) % ROOT_HISTORY_SIZE as u64) as usize;
        self.current_root_index = index as u64;
        self.roots[index] = new_root;
    }
}

#[account]
#[derive(InitSpace)]
pub struct NullifierRecord {
    pub pool: Pubkey,
    pub nullifier_hash: [u8; 32],
    pub recipient: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

impl NullifierRecord {
    pub const SEED: &'static [u8] = b"nullifier_record";
}
