use plonky2::plonk::circuit_data::VerifierCircuitData;
use std::collections::hash_map::DefaultHasher;
use zk::*;
use zk::txn::Transaction;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use plonky2::util::serialization::DefaultGateSerializer;
use plonky2::field::goldilocks_field::GoldilocksField as F;
use plonky2::field::types::{Field, PrimeField64};

fn hash<T: Hash>(t: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);
    hasher.finish()
}

fn main() -> Result<(), anyhow::Error>  {
    let config: CircuitConfig = CircuitConfig::standard_recursion_config();

    // Initialize a ZKP circuit with 2 inputs
    let zk_circuit_1: ZKPCircuit = ZKPCircuit::new(config.clone(), 2, |builder, targets| {
        let one = builder.one();
        let s: Target = builder.add(targets[0], one);
        builder.register_public_input(targets[0]);
        builder.register_public_input(targets[1]);
        builder.connect(s, targets[1]);
    });

    let mut m = HashMap::new();
    m.insert(hash(&zk_circuit_1.get_vk()), 0);

    let mut txns: Vec<Vec<u8>> = Vec::new();

    for i in 0..10 {
        let proof: ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2> = zk_circuit_1.prove(vec![i, i+1]).expect("REASON");

        let tx = Transaction{
            vk: zk_circuit_1.get_vk(),
            proof_data: proof.to_bytes(),
            common: zk_circuit_1.get_common_circuit_data()
        };

        let tx_data = tx.serialize();

        txns.push(tx_data);
    }

    for txn in txns {
        let tx: Transaction = Transaction::deserialize(&txn[..]);

        let common = deserialize_common_from_bytes(tx.common.clone())?;

        let proof_data: ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2> = deserialize_proof_from_bytes(tx.proof_data, common.clone())?;

        let vk = deserialize_vk_from_bytes(tx.vk.clone())?;

        if m.get(&hash(&tx.vk)) != Some(&(proof_data.public_inputs[0].to_canonical_u64() as i32)) || verify_circuit_data(proof_data.clone(), vk, common).is_err() {
            continue;
        } {
            m.insert(hash(&tx.vk), proof_data.clone().public_inputs[1].to_canonical_u64() as i32);
        }
    }

    println!("Map: {:?}", m);
    Ok(())  // Return Ok when the function completes successfully
}
