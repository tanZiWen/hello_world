use interpreter::Circuit;
use interpreter::Field64;
use interpreter::Interpreter;
use interpreter::MerkleProofTarget;
use interpreter::PoseidonHash;
use interpreter::Transaction;
use interpreter::WitnessWrite;

pub use interpreter::Hash;
use plonky2::plonk::config::GenericHashOut;
use plonky2::plonk::config::Hasher;
use plonky2::iop::target::BoolTarget;
use plonky2::iop::generator::GeneratedValues;
use plonky2::iop::witness::PartitionWitness;
use plonky2::iop::target::Target;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::generator::SimpleGenerator;
use serde::{Serialize, Deserialize};
use plonky2::field::types::PrimeField64;


fn main() {
    let c = Circuit::new(|builder| {
        let this = builder.add_virtual_hash_public_input();
        let root = builder.add_virtual_hash_public_input();
        let new = builder.add_virtual_public_input();
        let old = builder.add_virtual_target();
        let path = MerkleProofTarget { siblings: builder.add_virtual_hashes(256) };
        let index = this.elements.map(|v: plonky2::iop::target::Target| builder.split_le(v, 64)).concat();
        let one = builder.sub(new, old);

        builder.verify_merkle_proof::<PoseidonHash>(vec![old], &index, root, &path);
        builder.assert_one(one);
        (this, root, path, old, new)
    });
    let vk = c.vk();
    let mut s = Interpreter::new();
    for i in 0..16 {
        let (old, path) = s.prove(vk.address());
        // if i == 0 {
        //     s.insert(vk.address(), old);
        // }
        println!("old: {:?}", old);
        // print!("path: {:?}", path);
        println!("root: {:?}", s.root());
        let new = old.add_one();
        let proof_result = c.prove(|w, t| {
            println!("vk.address():{:?}", vk.address());
            w.set_hash_target(t.0, vk.address())?;
            w.set_hash_target(t.1, s.root())?;
            (0..256).try_for_each(|i| w.set_hash_target(t.2.siblings[i], path[i]))?;
            w.set_target(t.3, old)?;
            w.set_target(t.4, new)
        });

        match proof_result {
            Ok((proof, _)) => {
                print!("Transitioning\n");
                let tx: Transaction = Transaction { new, proof, vk: vk.clone() };
                eprintln!("transaction[{i}]: {:?}", s.transit(tx));
            }
            Err(e) => {
                eprintln!("Proving failed: {:?}", e);
                panic!("PROVING FAILURE");
            }
        } 
    }
}