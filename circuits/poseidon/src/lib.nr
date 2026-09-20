use std::hash::poseidon2_permutation;

pub fn hash_1(input: Field) -> Field {
    let state = [input, 0, 0, 0];
    poseidon2_permutation(state)[0]
}

pub fn hash_2(input1: Field, input2: Field) -> Field {
    let state = [input1, input2, 0, 0];
    poseidon2_permutation(state)[0]
}

pub fn hash_3(input1: Field, input2: Field, input3: Field) -> Field {
    let state = [input1, input2, input3, 0];
    poseidon2_permutation(state)[0]
}
