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
use std::array::from_fn;
use std::iter::once;

use anyhow::Result;
use plonky2::plonk::config::Hasher;
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

    fn get_path(hash: &Hash) -> [bool; 256] {    
        from_fn(|i| hash.elements[3 - i / 64].0 >> (63 - i % 64) & 1 > 0)
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
        let mut path: Vec<bool> = Self::get_path(&key).to_vec();

       self.nodes.insert(path.clone(), PoseidonHash::hash_or_noop(&[value]));

       for _ in 0..256 {
            path.pop();
            
            let [left, right] = [false, true].map(|v| self.get_digest(&path.iter().cloned().chain(once(v)).collect::<Vec<_>>()));
            self.nodes.insert(path.clone(),  PoseidonHash::two_to_one(left, right));
        };
    }

    fn get_digest(&self, index: &[bool]) -> Hash {
        static DEFAULT_HASHS: LazyLock<[Hash; 257]> = LazyLock::new(|| {
            let mut default_hashes = [PoseidonHash::hash_or_noop(&[GoldilocksField::ZERO;1]); DEPTH + 1];
            (0..256).rev().for_each(|i| default_hashes[i]=PoseidonHash::two_to_one(default_hashes[i+1], default_hashes[i+1]));
            default_hashes
        });
        self.nodes.get(index).cloned().unwrap_or(DEFAULT_HASHS[index.len()])
    }
   
    pub fn get_merkle_proof(&self, key: &[bool]) -> [Hash; 256] {
       from_fn(|i| self.get_digest(&key[0..255 - i].iter().cloned().chain(once(!key[255 - i])).collect::<Vec<_>>()))
    }
}