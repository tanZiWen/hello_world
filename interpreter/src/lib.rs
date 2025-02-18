pub mod txn;
pub mod counter;

pub use plonky2::{
    field::{goldilocks_field::GoldilocksField, types::Field},
    iop::{
        target::Target,
        witness::{PartialWitness, WitnessWrite},
    },
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData, VerifierOnlyCircuitData, CommonCircuitData, VerifierCircuitData},
        config::PoseidonGoldilocksConfig,
        proof::ProofWithPublicInputs,
    },
    util::serialization::DefaultGateSerializer,
};

const D: usize = 2;
type C = PoseidonGoldilocksConfig;
type F = GoldilocksField;

/// A structure representing a general ZKP circuit that can be customized with different constraints.
pub struct ZKPCircuit{
    pub circuit_data: CircuitData<F, C, D>,
    targets: Vec<Target>,
}

impl ZKPCircuit {
    /// Builds a new general ZKP circuit
    pub fn new(config: CircuitConfig, num_inputs: usize, constraint_fn: impl Fn(&mut CircuitBuilder<F, D>, &mut Vec<Target>)) -> Self {
        let mut builder = CircuitBuilder::<F, D>::new(config);
        let mut targets = Vec::new();

        // Create virtual targets for all inputs
        for _ in 0..num_inputs {
            let target = builder.add_virtual_target();
            targets.push(target);
        }

        // Add the custom constraints to the circuit
        constraint_fn(&mut builder, &mut targets);

        // Build the circuit
        let circuit_data: CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2> = builder.build::<C>();

        Self {
            circuit_data,
            targets,
        }
    }

    pub fn get_vk(self: &Self) -> Vec<u8> {
        self.circuit_data.verifier_only.clone().to_bytes().unwrap_or_else(|_| vec![])
    }

    pub fn get_common_circuit_data(&self) -> Vec<u8> {
        self.circuit_data.common.clone().to_bytes(&DefaultGateSerializer).unwrap_or_else(|_| vec![])
    }

    /// Generates the proof for a given set of inputs
    pub fn prove(
        &self,
        inputs: Vec<u64>,
    ) -> Result<ProofWithPublicInputs<F, C, D>, anyhow::Error> {

        if inputs.len() != self.targets.len() {
            println!("Input size mismatch, expected: {}, got: {}", self.targets.len(), inputs.len());
            anyhow::bail!("Input size mismatch");
        }

        // Convert input values to field elements
        let field_inputs: Vec<F> = inputs
            .iter()
            .map(|&val| F::from_canonical_u64(val))
            .collect();

        // Create witness and set the inputs
        let mut witness = PartialWitness::new();
        for (i, &val) in field_inputs.iter().enumerate() {
            let _ = witness.set_target(self.targets[i], val);
        }        
        // Generate proof
        let proof= self.circuit_data.prove(witness)?;

        Ok(proof)
    }

    /// Verifies the proof
    pub fn verify(
        &self,
        proof: &ProofWithPublicInputs<F, C, D>,
        expected_public_inputs: Vec<u64>,
    ) -> Result<(), anyhow::Error> {
        // Verify the validity of the proof
        self.circuit_data.verify(proof.clone())?;

        // Verify the public inputs
        if proof.public_inputs.len() != expected_public_inputs.len() {
            println!("Public input size mismatch, expected: {}, got: {}", expected_public_inputs.len(), proof.public_inputs.len());
            anyhow::bail!("Public input size mismatch");
        }
        let public_inputs = proof.public_inputs.clone();
        for (i, expected) in expected_public_inputs.iter().enumerate() {
            let expected_f = F::from_canonical_u64(*expected);
            if public_inputs[i] != expected_f {
                anyhow::bail!("Public input mismatch at index {}", i);
            }
        }

        Ok(())
    }
}

pub fn verify_circuit_data(proof: ProofWithPublicInputs<F, C, D>, vk: VerifierOnlyCircuitData<C, 2>, common:  CommonCircuitData<F, D>) -> Result<(), anyhow::Error> {
    VerifierCircuitData{
        verifier_only: vk.clone(),
        common,
    }.verify(proof)?;
    
    Ok(()) 
}

pub fn deserialize_vk_from_bytes(data: Vec<u8>) -> Result<VerifierOnlyCircuitData<C, 2>, anyhow::Error> {
    match VerifierOnlyCircuitData::from_bytes(data) {
        Ok(vk) => {
            println!("Verification key loaded successfully");
            Ok(vk)
        }
        Err(e) => {
            println!("Error loading verification key: {}", e);
            Err(anyhow::anyhow!(e))
        }
    }
}

pub fn deserialize_proof_from_bytes(data: Vec<u8>, common_data: CommonCircuitData<F, D>) -> Result<ProofWithPublicInputs<F, C, 2>, anyhow::Error> {
    match ProofWithPublicInputs::from_bytes(data, &common_data) {
        Ok(proof) => Ok(proof),
        Err(e) => {
            // Log more details about the error to aid debugging
            eprintln!("Deserialization failed with error: {:?}", e);
            Err(anyhow::anyhow!("Failed to deserialize proof: {:?}", e))
        }
    }
}

pub fn deserialize_common_from_bytes(data: Vec<u8>) -> Result<CommonCircuitData<F, D>, anyhow::Error> {
    match CommonCircuitData::from_bytes(data,&DefaultGateSerializer) {
        Ok(common) => Ok(common),
        Err(e) => {
            // Log more details about the error to aid debugging
            eprintln!("Deserialization common failed with error: {:?}", e);
            Err(anyhow::anyhow!("Failed to deserialize common: {:?}", e))
        }
    }
}

/// Example usage of the general ZKP library
#[test]
fn general_zkp_example() -> Result<(), anyhow::Error> {
    // Define the circuit configuration
    let config: CircuitConfig = CircuitConfig::standard_recursion_config();

    // Initialize a ZKP circuit with 2 inputs
    let zk_circuit = ZKPCircuit::new(config, 3, |builder, targets| {
        let sum: Target = builder.add_virtual_public_input();
        let s: Target = builder.add(targets[0], targets[1]);
        builder.connect(s, sum);
    });

    // Test case: x = 3, y = 5, z = 8 (x + y = z)
    let inputs = vec![3, 5, 8];

    // Generate proof for the test case
    let proof: ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2> = zk_circuit.prove(inputs.clone())?;
    println!("Proof generated successfully!");

    // Verify the proof
    zk_circuit.verify(&proof, vec![8])?;
    println!("Proof verified successfully!");

    Ok(())
}
