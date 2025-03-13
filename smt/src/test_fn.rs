use std::{array::from_fn, fmt::format};
pub use plonky2::hash::poseidon::PoseidonHash;
use plonky2::{
    field::{goldilocks_field::GoldilocksField, types::Field},
    hash::hash_types::HashOut,
};
use plonky2::field::types::PrimeField64;
pub type Hash = HashOut<GoldilocksField>;
use std::iter::once;
use std::iter::Iterator;
use plonky2::plonk::config::Hasher;
use blake3::Hasher as blake3;

#[test]
fn teset_fn() {
    let key = HashOut {
        elements: [
            GoldilocksField::from_canonical_u64(0x1234567890ABCDEF), // 元素0
            GoldilocksField::from_canonical_u64(0xFEDCBA0987654321), // 元素1
            GoldilocksField::from_canonical_u64(0x0A1B2C3D4E5F6A7B), // 元素2
            GoldilocksField::from_canonical_u64(0x8C9DAEBFC0D1E2F3), // 元素3
        ],
    };
    let mut index = [&key, &key].map(hash_to_index).concat();

    println!("{}", index.len());

    println!("{:?}", once(false));

    let key_path1: [bool; 256] = from_fn(|i| key.elements[3 - i/64].0 >> (63 - i%64) & 1 > 0);
    let key_path2: [u64; 256] = from_fn(|i| key.elements[3 - i/64].0 >> (63 - i%64) & 1);
    let mut key_path3 = split_hash_le(key, 64);
    key_path3.reverse();
    assert_eq!(key_path1.to_vec(), key_path3.clone());

    let flat_map = [['a', 'b', 'c', 'd'], ['e', 'f', 'g', 'h']].iter().flat_map(|v| *v).collect::<Vec<_>>();
    assert_eq!(flat_map, ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h']);

    let words = ["alpha", "beta", "gamma"];
    let merged: String = words.iter().flat_map(|s| s.chars()).collect();

    assert_eq!(merged, "alphabetagamma");

    let mun = [[1,2,3,4],[5,6,7,8]];
    let merged = mun.iter().flat_map(|s| s).collect::<Vec<_>>();
    println!("num:{:?}", merged);

    let flod_map: i32 = [1,2,3,4,5,6].iter().fold(0, |sum, i| sum + i );
    assert_eq!(flod_map, 21);

    let flod_str: String = ["hello", "world"].iter().fold(String::new(), |str, i| {
        if str.is_empty() {
            i.to_string()
        } else {
            format!("{}, {}", str, i)
        }
    });
    assert_eq!(flod_str, "hello, world");
    
    let nested = vec![vec![1, 2], vec![3, 4]];
    let flat: Vec<_> = nested.into_iter().flatten().collect();
    assert_eq!(flat, vec![1, 2, 3, 4]);

    let map_value = [1, 2].map(|v| v +1 );

    assert_eq!(map_value, [2, 3]);

    let addr: HashOut<GoldilocksField> = HashOut {
        elements: [
            GoldilocksField::from_canonical_u64(0x12345e7890ABCDEF), // 元素0
            GoldilocksField::from_canonical_u64(0xFEDCBb0987654321), // 元素1
            GoldilocksField::from_canonical_u64(0xF4DCBA0987654321), // 元素2
            GoldilocksField::from_canonical_u64(0x8C1DAEBFC0D1E2F3), // 元素3
        ],
    };

    let key = HashOut {
        elements: [
            GoldilocksField::from_canonical_u64(0x1234567890ABCDEF), // 元素0
            GoldilocksField::from_canonical_u64(0xFEDCBA0987654321), // 元素1
            GoldilocksField::from_canonical_u64(0x0A1B2C3D4E5F6A7B), // 元素2
            GoldilocksField::from_canonical_u64(0x8C9DAEBFC0D1E2F3), // 元素3
        ],
    };
    let addr = PoseidonHash::hash_no_pad(&[GoldilocksField::from_canonical_u64(1)]);
    let key = PoseidonHash::hash_no_pad(&[GoldilocksField::from_canonical_u64(2)]);
    println!("addr:{:?}, key:{:?}", addr, key);

    let addr_bk = hash_v(&[1]);
    let key_bk = hash_v(&[2]);
    println!("addr:{:?}, key:{:?}", addr_bk, key_bk);

    let addr_bk_to_p = blake3_to_hashout(addr_bk);

    println!("addr_bk_to_p:{:?}", addr_bk_to_p);

    // let index = [&addr, &key].map(hash_to_index).concat();
    // println!("index: {:?}", index);

    let slice = ["l", "o", "r", "e", "m"];
    let mut iter = slice.chunks_exact(2);
    println!("iter: {:?}", iter);

    let i = iter.for_each(|v| println!("{:?}", v[0]));

}

fn hash_to_index(hash: &Hash) -> [bool; 256] { from_fn(|i| hash.elements[3 - i / 64].0 >> (63 - i % 64) & 1 > 0) }

fn hash_v(value: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::new();
    hasher.update(value);
    let mut hash = [0; 32];
    hash.copy_from_slice(hasher.finalize().as_bytes());
    hash
}

pub fn blake3_to_hashout(blake3_hash: [u8; 32]) -> HashOut<GoldilocksField> {
    let mut chunks = [0u64; 4];
    blake3_hash.chunks_exact(8)
        .enumerate()
        .for_each(|(i, chunk)| {
            chunks[i] = u64::from_le_bytes(chunk.try_into().unwrap());
        });
    
    HashOut {
        elements: chunks.map(GoldilocksField)
    }
}


fn split_hash_le(hash: HashOut<GoldilocksField>, bits_per_element: usize) -> Vec<bool> {
    hash.elements
        .iter()
        .flat_map(|&elem| split_le_field_element(elem, bits_per_element))
        .collect()
}

fn split_le_field_element(value: GoldilocksField, num_bits: usize) -> Vec<bool> {
    let value_u64 = value.to_canonical_u64();
    let mut bits = Vec::with_capacity(num_bits);
        for i in 0..num_bits {
        bits.push((value_u64 >> i) & 1 == 1);
    }
    if num_bits > 64 {
        bits.resize(num_bits, false);
    }
    bits
}