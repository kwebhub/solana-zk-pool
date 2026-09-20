mod merkle_tree;
use merkle_tree::compute_merkle_root;
use poseidon::{hash_1, hash_2, hash_3};

global TREE_DEPTH: u32 = 20;

fn main(
    root: pub Field,
    nullifier_hash: pub Field,
    recipient: pub Field,
    recipient_binding: pub Field,
    amount: pub Field,
    nullifier: Field,
    secret: Field,
    note_secret: Field,
    merkle_proof: [Field; TREE_DEPTH],
    is_even: [bool; TREE_DEPTH],
) {
    // 1. Проверка nullifier_hash
    let computed_nullifier_hash = hash_1(nullifier);
    assert(computed_nullifier_hash == nullifier_hash, "Invalid nullifier hash");

    // 2. Проверка binding: hash_2(note_secret, recipient) == recipient_binding
    let computed_binding = hash_2(note_secret, recipient);
    assert(computed_binding == recipient_binding, "Invalid recipient binding");

    // 3. Проверка Merkle proof
    let commitment = hash_3(nullifier, secret, amount);
    let computed_root = compute_merkle_root(commitment, merkle_proof, is_even);
    assert(computed_root == root, "Invalid Merkle proof");
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
    let note_secret: Field = 11111;
    let amount: Field = 1000000000;
    let recipient: Field = 123456789;

    let commitment = hash_3(nullifier, secret, amount);
    let nullifier_hash = hash_1(nullifier);
    let recipient_binding = hash_2(note_secret, recipient);
    let zeros = compute_empty_tree_zeros();

    let merkle_proof = zeros;
    let is_even = [true; TREE_DEPTH];
    let root = merkle_tree::compute_merkle_root(commitment, merkle_proof, is_even);

    println("=== Prover.toml data ===");
    println(f"root = \"{root}\"");
    println(f"nullifier_hash = \"{nullifier_hash}\"");
    println(f"recipient = \"{recipient}\"");
    println(f"recipient_binding = \"{recipient_binding}\"");
    println(f"amount = \"{amount}\"");
    println(f"nullifier = \"{nullifier}\"");
    println(f"secret = \"{secret}\"");
    println(f"note_secret = \"{note_secret}\"");

    println("merkle_proof = [");
    for i in 0..TREE_DEPTH {
        let z = zeros[i];
        print(f"\"{z}\"");
        if i < TREE_DEPTH - 1 { println(","); } else { println(""); }
    }
    println("]");

    println("is_even = [");
    for i in 0..TREE_DEPTH {
        print("1");
        if i < TREE_DEPTH - 1 { println(","); } else { println(""); }
    }
    println("]");
    println("=== End Prover.toml data ===");

    let computed_root = merkle_tree::compute_merkle_root(commitment, merkle_proof, is_even);
    assert(computed_root == root);
}
