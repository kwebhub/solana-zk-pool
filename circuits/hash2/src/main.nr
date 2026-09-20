use poseidon::hash_2;

fn main(left: Field, right: Field) -> pub Field {
        hash_2(left, right)
    }

#[test]
fn test_hash_2_deterministic() {
        let h1 = hash_2(1, 2);
        let h2 = hash_2(1, 2);
        assert(h1 == h2);
    }

#[test]
fn test_hash_2_not_comutative() {
        let h1 = hash_2(1, 2);
        let h2 = hash_2(2, 1);
        assert(h1 != h2);
    }
