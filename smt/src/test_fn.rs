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



}

fn hash_to_index(hash: &Hash) -> [bool; 256] { from_fn(|i| hash.elements[3 - i / 64].0 >> (63 - i % 64) & 1 > 0) }

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