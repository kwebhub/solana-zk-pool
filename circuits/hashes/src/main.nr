use poseidon::{hash_1, hash_3};

fn main(nullifier: Field, secret: Field, amount: Field) -> pub (Field, Field) {
    let commitment = hash_3(nullifier, secret, amount);
    let nullifier_hash = hash_1(nullifier);
    (commitment, nullifier_hash)
}

#[test]
fn test_hashes_deterministic() {
    let (c1, n1) = main(1, 2, 3);
    let (c2, n2) = main(1, 2, 3);
    assert(c1 == c2);
    assert(n1 == n2);
}

#[test]
fn test_unique_amount() {
    let (c1, n1) = main(1, 2, 3);
    let (c2, n2) = main(1, 2, 5);
    assert(c1 != c2);
    assert(n1 == n2);
}

#[test]
fn test_unique_secret() {
    let (c1, n1) = main(1, 2, 3);
    let (c2, n2) = main(1, 5, 3);
    assert(c1 != c2);
    assert(n1 == n2);
}

#[test]
fn test_unique_nullifier() {
    let (c1, n1) = main(1, 2, 3);
    let (c2, n2) = main(5, 2, 3);
    assert(c1 != c2);
    assert(n1 != n2);
}
