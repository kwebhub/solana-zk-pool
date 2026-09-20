use poseidon::hash_2;

pub fn compute_merkle_root<let DEPTH: u32>(
    leaf: Field,
    path: [Field; DEPTH],
    is_even: [bool; DEPTH],
) -> Field {
    let mut current = leaf;

    for i in 0..DEPTH {
        let sibling = path[i];

        let (left, right) = if is_even[i] {
            (current, sibling)
        } else {
            (sibling, current)
        };

        current = hash_2(left, right);
    }

    current
}
