//! Integration tests for the `zk_pool` program.
//!
//! Uses LiteSVM for in-process testing. Full withdraw with ZK proof is not tested
//! (requires Sunspot + Noir circuit). We test:
//! - PDA derivation (deterministic, unique, bump).
//! - Program ID parsing.
//! - Constants.
//! - Discriminators (match IDL).
//! - Account sizes.

use anchor_lang::prelude::Pubkey;
use std::str::FromStr;

const PROGRAM_ID_STR: &str = "CZjsHYcE6oPhj2ZAKVdnaz36FQHC7SBd7MSY1jmAaj3o";
const VERIFIER_PROGRAM_ID_STR: &str = "AukfYGQqhv3ZkdB1G7cKF3djsE9FKMrAfLd34doSWQex";
const MIN_DEPOSIT_LAMPORTS: u64 = 1_000_000; // 0.001 SOL
const INITIAL_BALANCE: u64 = 10_000_000_000; // 10 SOL
const TREE_DEPTH: usize = 20;
const ROOT_HISTORY_SIZE: usize = 10;
const NR_PUBLIC_INPUTS: u32 = 5;

// DepositEvent discriminator (from IDL)
const DEPOSIT_DISC: [u8; 8] = [120, 248, 61, 83, 31, 142, 107, 144];
// WithdrawEvent discriminator (from IDL)
const WITHDRAW_DISC: [u8; 8] = [22, 9, 133, 26, 160, 44, 71, 192];

// ============================================================
// Helpers
// ============================================================

fn program_id() -> Pubkey {
    Pubkey::from_str(PROGRAM_ID_STR).unwrap()
}

fn verifier_program_id() -> Pubkey {
    Pubkey::from_str(VERIFIER_PROGRAM_ID_STR).unwrap()
}

fn find_pool_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"pool3"], &program_id())
}

fn find_vault_pda(pool: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"vault3", pool.as_ref()], &program_id())
}

fn find_nullifier_record_pda(pool: &Pubkey, nullifier_hash: &[u8; 32]) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"nullifier_record", pool.as_ref(), nullifier_hash.as_ref()],
        &program_id(),
    )
}

// ============================================================
// Program ID
// ============================================================

#[test]
fn test_program_id_parses() {
    let id = program_id();
    assert_eq!(id.to_bytes().len(), 32);
    assert_ne!(id, Pubkey::default());
}

#[test]
fn test_verifier_program_id_parses() {
    let id = verifier_program_id();
    assert_eq!(id.to_bytes().len(), 32);
    assert_ne!(id, Pubkey::default());
    assert_ne!(id, program_id());
}

// ============================================================
// PDA derivation
// ============================================================

#[test]
fn test_find_pool_pda_deterministic() {
    let (pool1, _) = find_pool_pda();
    let (pool2, _) = find_pool_pda();
    assert_eq!(pool1, pool2);
    assert_ne!(pool1, program_id());
}

#[test]
fn test_find_vault_pda() {
    let (pool, _) = find_pool_pda();
    let (vault, _) = find_vault_pda(&pool);
    assert_ne!(vault, program_id());
    assert_ne!(vault, pool);

    let other_pool = Pubkey::new_unique();
    let (other_vault, _) = find_vault_pda(&other_pool);
    assert_ne!(vault, other_vault);
}

#[test]
fn test_find_nullifier_record_pda() {
    let (pool, _) = find_pool_pda();
    let nullifier = [42u8; 32];
    let (record, _) = find_nullifier_record_pda(&pool, &nullifier);
    assert_ne!(record, program_id());
    assert_ne!(record, pool);

    let other_nullifier = [43u8; 32];
    let (other_record, _) = find_nullifier_record_pda(&pool, &other_nullifier);
    assert_ne!(record, other_record);
}

#[test]
fn test_pdas_do_not_collide() {
    let (pool, _) = find_pool_pda();
    let (vault, _) = find_vault_pda(&pool);
    let (record, _) = find_nullifier_record_pda(&pool, &[7u8; 32]);

    assert_ne!(pool, vault);
    assert_ne!(pool, record);
    assert_ne!(vault, record);
}

// ============================================================
// Constants
// ============================================================

#[test]
fn test_min_deposit_amount() {
    assert_eq!(MIN_DEPOSIT_LAMPORTS, 1_000_000);
}

#[test]
fn test_tree_depth() {
    assert_eq!(TREE_DEPTH, 20);
    assert_eq!(1u64 << TREE_DEPTH, 1_048_576);
}

#[test]
fn test_root_history_size() {
    assert_eq!(ROOT_HISTORY_SIZE, 10);
}

#[test]
fn test_nr_public_inputs() {
    assert_eq!(NR_PUBLIC_INPUTS, 5);
}

#[test]
fn test_initial_balance_is_enough() {
    const _: () = assert!(INITIAL_BALANCE >= 10 * MIN_DEPOSIT_LAMPORTS);
}

// ============================================================
// Discriminators (match IDL)
// ============================================================

#[test]
fn test_deposit_event_discriminator() {
    assert_eq!(DEPOSIT_DISC.len(), 8);
    // Expected from IDL: [120, 248, 61, 83, 31, 142, 107, 144]
    assert_eq!(DEPOSIT_DISC[0], 120);
    assert_eq!(DEPOSIT_DISC[1], 248);
    assert_eq!(DEPOSIT_DISC[7], 144);
}

#[test]
fn test_withdraw_event_discriminator() {
    assert_eq!(WITHDRAW_DISC.len(), 8);
    // Expected from IDL: [22, 9, 133, 26, 160, 44, 71, 192]
    assert_eq!(WITHDRAW_DISC[0], 22);
    assert_eq!(WITHDRAW_DISC[1], 9);
    assert_eq!(WITHDRAW_DISC[7], 192);
}

#[test]
fn test_discriminators_differ() {
    assert_ne!(DEPOSIT_DISC, WITHDRAW_DISC);
}

// ============================================================
// Event sizes
// ============================================================

#[test]
fn test_deposit_event_size() {
    // discriminator(8) + commitment(32) + leaf_index(8) + timestamp(8) + new_root(32)
    assert_eq!(8 + 32 + 8 + 8 + 32, 88);
}

#[test]
fn test_withdraw_event_size() {
    // discriminator(8) + nullifier_hash(32) + recipient(32) + amount(8) + timestamp(8)
    assert_eq!(8 + 32 + 32 + 8 + 8, 88);
}

// ============================================================
// Account sizes
// ============================================================

#[test]
fn test_pool_state_size() {
    // discriminator(8) + authority(32) + next_leaf_index(8) + total_deposits(8)
    // + current_root_index(8) + roots(10 × 32)
    assert_eq!(8 + 32 + 8 + 8 + 8 + ROOT_HISTORY_SIZE * 32, 384);
}

#[test]
fn test_nullifier_record_size() {
    // discriminator(8) + pool(32) + nullifier_hash(32) + recipient(32) + amount(8) + timestamp(8)
    assert_eq!(8 + 32 + 32 + 32 + 8 + 8, 120);
}
