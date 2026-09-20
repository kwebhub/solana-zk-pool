mod merkle_tree;
use merkle_tree::compute_merkle_root;
use poseidon::{hash_1, hash_2, hash_3};

global TREE_DEPTH: u32 = 20;

fn main(
    root: pub Field,
    nullifier_hash: pub Field,
    recipient: pub Field,
    amount: pub Field,
    nullifier: Field,
    secret: Field,
    merkle_proof: [Field; TREE_DEPTH],
    is_even: [bool; TREE_DEPTH],
) {
    let commitment = hash_3(nullifier, secret, amount);

    let computed_nullifier_hash = hash_1(nullifier);

    assert(computed_nullifier_hash == nullifier_hash, "Invalid nullifier hash");

    let computed_root = compute_merkle_root(commitment, merkle_proof, is_even);

    assert(computed_root == root, "Invalid Merkle proof");

    let _ = recipient;
}

fn compute_empty_tree_zeros() -> [Field; TREE_DEPTH] {
    let mut zeros: [Field; TREE_DEPTH] = [0; TREE_DEPTH];
    let mut current: Field = 0;

    for i in 0..TREE_DEPTH {
        zeros[i] = current;
        current = hash_2(current, current);
    }

    zeros
}

#[test]
fn test_generate_valid_inputs() {
    let nullifier: Field = 12345;
    let secret: Field = 67890;
    let amount: Field = 1000000000;
    let recipient: Field = 123456789;

    let commitment = hash_3(nullifier, secret, amount);
    let nullifier_hash = hash_1(nullifier);
    let zeros = compute_empty_tree_zeros();

    let merkle_proof = zeros;
    let is_even = [true; TREE_DEPTH];
    let root = merkle_tree::compute_merkle_root(commitment, merkle_proof, is_even);

    println("=== Данные для Prover.toml ===");
    println(f"root = \"{root}\"");
    println(f"secret = \"{secret}\"");
    println(f"amount = \"{amount}\"");

    println("is_even = [");
    for i in 0..TREE_DEPTH {
        print("1");
        if i < TREE_DEPTH - 1 {
            println(",");
        } else {
            println("");
        }
    }
    println("]");

    println("merkle_proof = [");
    for i in 0..TREE_DEPTH {
        let prove = zeros[i];
        print(f"\"{prove}\"");
        if i < TREE_DEPTH - 1 {
            println(",");
        } else {
            println("");
        }
    }
    println("]");

    println(f"nullifier = \"{nullifier}\"");
    println(f"nullifier_hash = \"{nullifier_hash}\"");
    println(f"recipient = \"{recipient}\"");
    println("=== Конец данных ===");

    let computed = merkle_tree::compute_merkle_root(commitment, merkle_proof, is_even);

    assert(computed == root);
}

#[test]
fn test_print_commitment_for_js() {
    let nullifier: Field = 12345;
    let secret: Field = 67890;
    let amount: Field = 1000000000;

    let commitment = hash_3(nullifier, secret, amount);
    let nullifier_hash = hash_1(nullifier);

    println(f"commitment = 0x{commitment}");
    println(f"nullifier_hash = 0x{nullifier_hash}");
}
