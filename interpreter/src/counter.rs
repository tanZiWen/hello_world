use plonky2::{
    field::{goldilocks_field::GoldilocksField, types::Field},
    iop::{
        target::Target,
        witness::{PartialWitness, WitnessWrite},
    },
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData},
        config::PoseidonGoldilocksConfig,
        proof::ProofWithPublicInputs,
    },
};

use plonky2::util::serialization::DefaultGateSerializer;


const D: usize = 2;
type C = PoseidonGoldilocksConfig;
type F = GoldilocksField;

struct CounterCircuit {
    circuit_data: CircuitData<F, C, D>,
    current_target: Target,
    next_target: Target,
}

impl CounterCircuit {
    /// Build the counter circuit, returning the circuit data and input targets
    pub fn new() -> Self {
        let config = CircuitConfig::standard_recursion_config();
        let mut builder = CircuitBuilder::<F, D>::new(config);

        // Register the public input (next counter value)
        let next_target = builder.add_virtual_public_input();
        
        // Register the private input (current counter value)
        let current_target = builder.add_virtual_target();

         // Add constraint: next = current + 1
        let one = builder.one();
        let computed_next = builder.add(current_target, one); 
        builder.connect(computed_next, next_target);

        let circuit_data: CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2> = builder.build::<C>();

        Self {
            circuit_data,
            current_target,
            next_target,
        }
    }

     /// Generate the counter increment proof
    pub fn prove(&self, current_val: u64) -> Result<ProofWithPublicInputs<F, C, D>, anyhow::Error> {
        // Convert value to field element
        let current_f = F::from_canonical_u64(current_val);
        let next_f = current_f + F::ONE;

        // Create the witness and set input values
        let mut pw = PartialWitness::new();
        let _ = pw.set_target(self.current_target, current_f);
        let _ = pw.set_target(self.next_target, next_f);

        // Generate the proof
        let proof = self.circuit_data.prove(pw)?;
        Ok(proof)
    }

    /// Verify the counter proof
    pub fn verify(
        &self,
        proof: &ProofWithPublicInputs<F, C, D>,
        expected_next: u64,
    ) -> Result<(), anyhow::Error> {
        // Verify the validity of the proof
        self.circuit_data.verify(proof.clone())?;

        // Verify if the public input matches the expected value
        let public_inputs = proof.public_inputs.clone();
        let expected_next_f = F::from_canonical_u64(expected_next);
        
        if public_inputs.is_empty() || public_inputs[0] != expected_next_f {
            anyhow::bail!("Public input mismatch");
        }

        Ok(())
    }
}

#[test]
fn counter_example() -> Result<(), anyhow::Error> {
    // Initialize the counter circuit
    let counter_circuit = CounterCircuit::new();

    // Test case: Current value is 3, expected next value is 4
    let current_value = 3;
    let expected_next = 4;

     // Generate the proof
    let proof = counter_circuit.prove(current_value)?;
    println!("Proof generated successfully!");

    let bytes_result = counter_circuit.circuit_data.common.clone().to_bytes(&DefaultGateSerializer);
    match bytes_result {
        Ok(bytes) => println!("Expected next value length: {:?}", bytes.len()),
        Err(e) => println!("Error serializing circuit data: {:?}", e),
    }

    // Verify the proof
    counter_circuit.verify(&proof, expected_next)?;
    println!("Proof verified successfully!");

    Ok(())
}

