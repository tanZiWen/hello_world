mod transaction;
mod zk;
pub mod test;

pub use crate::transaction::Transaction;
pub use crate::zk::Circuit;
pub use crate::zk::Field;
pub use crate::zk::Field64;
pub use crate::zk::GoldilocksField;
pub use crate::zk::Hash;
pub use crate::zk::MerkleProofTarget;
pub use crate::zk::PoseidonHash;
pub use crate::zk::WitnessWrite;
pub use crate::zk::HashOut;
use core::hash;
use std::array::from_fn;
use std::iter::once;

use anyhow::Result;
use plonky2::field::types::PrimeField64;
use plonky2::plonk::config::GenericHashOut;
use plonky2::plonk::config::Hasher;
use plonky2::util::serialization::gate_serialization::default;
use std::collections::HashMap;
use std::sync::LazyLock;

pub struct Interpreter {
    s: HashMap<Hash, GoldilocksField>,
    smt: SparseMerkleTree,

}

impl Interpreter {
    pub fn new() -> Self { 
        Self{
            s: HashMap::new(),
            smt: SparseMerkleTree::new(),
        }
     }
    pub fn prove(&mut self, addr: Hash) -> (GoldilocksField, [Hash; 256]) {
        let proof = self.smt.get_merkle_proof(&SparseMerkleTree::get_path(&addr));
        let value = self.s.entry(addr).or_default();
        (*value, proof)
    }
    pub fn root(&self) -> Hash { self.smt.root() }
    pub fn transit(&mut self, tx: Transaction) -> Result<()> {
        tx.vk.verify(self.root(), tx.new, tx.proof)?;
        Ok(self.insert(tx.vk.address(), tx.new))
    }
    pub fn insert(&mut self, addr: Hash, value: GoldilocksField) { 
        println!("addr: {:?}, value: {:?}", addr, value);
        self.s.insert(addr, value);
        self.smt.insert(addr, value);
     }
     pub fn get_path(&mut self, addr: Hash)-> [bool; 256] { SparseMerkleTree::get_path(&addr) }
}

const DEPTH: usize = 256;
struct SparseMerkleTree {
    nodes: HashMap<Vec<bool>, Hash>,
}

impl  SparseMerkleTree {

    fn hash_children(left: Hash, right: Hash) -> Hash {
        PoseidonHash::two_to_one(left, right)
    }

    fn get_path(hash: &Hash) -> [bool; 256] {    
        from_fn(|i| hash.elements[3 - i / 64].0 >> (63 - i % 64) & 1 > 0)
    }

    fn split_hash_le(hash: HashOut<GoldilocksField>, bits_per_element: usize) -> Vec<bool> {
        hash.elements
            .iter()
            .flat_map(|&elem| Self::split_le_field_element(elem, bits_per_element))
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

    fn new() -> Self {
      
        SparseMerkleTree {
            nodes: HashMap::new(),
        }
    }

    fn root(&self) -> Hash {
        self.get_digest(&vec![])
    }

   
    fn insert(&mut self, key: Hash, value: GoldilocksField) {
        let mut path: Vec<bool> = Self::get_path(&key).into();

        self.nodes.insert(path.clone(), PoseidonHash::hash_no_pad(&[value]));

       for _ in 0..256 {
            path.pop();
            
            let [left, right] = [false, true].map(|v| self.get_digest(&path.iter().cloned().chain(once(v)).collect::<Vec<_>>()));
             
            self.nodes.insert(path.clone(),  PoseidonHash::two_to_one(left, right));
        };
    }

    fn get_digest(&self, index: &[bool]) -> Hash {
        static DEFAULT_HASHS: LazyLock<[Hash; 257]> = LazyLock::new(|| {
            let mut default_hashes = [Hash::ZERO; DEPTH + 1];
            (0..256).rev().for_each(|i| default_hashes[i]=PoseidonHash::two_to_one(default_hashes[i+1], default_hashes[i+1]));
            default_hashes
        });
        self.nodes.get(index).cloned().unwrap_or(DEFAULT_HASHS[index.len()])
    }
   
    pub fn get_merkle_proof(&self, key: &[bool]) -> [Hash; 256] {
       from_fn(|i| self.get_digest(&key[0..255 - i].iter().cloned().chain(once(!key[255 - i])).collect::<Vec<_>>()))
    }

}