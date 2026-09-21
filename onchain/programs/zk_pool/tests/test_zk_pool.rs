use anchor_lang::prelude::Pubkey;
use std::str::FromStr;

const PROGRAM_ID_STR: &str = "CZjsHYcE6oPhj2ZAKVdnaz36FQHC7SBd7MSY1jmAaj3o";
const MIN_DEPOSIT_LAMPORTS: u64 = 1_000_000;
const INITIAL_BALANCE: u64 = 10_000_000_000; // 10 SOL

fn program_id() -> Pubkey {
    Pubkey::from_str(PROGRAM_ID_STR).unwrap()
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

#[test]
fn test_program_id_parses() {
    let id = program_id();
    assert_eq!(id.to_bytes().len(), 32);
    assert_ne!(id, Pubkey::default());
}

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

#[test]
fn test_min_deposit_amount() {
    assert_eq!(MIN_DEPOSIT_LAMPORTS, 1_000_000);
}

#[test]
fn test_initial_balance_is_enough() {
    const _: () = assert!(INITIAL_BALANCE >= 10 * MIN_DEPOSIT_LAMPORTS);
}
