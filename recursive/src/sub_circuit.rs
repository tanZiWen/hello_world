use plonky2::{
    field::{goldilocks_field::GoldilocksField, types:: Field},
    iop::{
        witness::{PartialWitness, WitnessWrite},
        target::Target,
    },
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData, VerifierCircuitTarget},
        config::PoseidonGoldilocksConfig,
        verifier,
        proof::ProofWithPublicInputsTarget,
        
    },
};
use plonky2::iop::generator::generate_partial_witness;
use plonky2::plonk::prover::prove_with_partition_witness;
use plonky2::util::timing::TimingTree;

use crate::C;

type F = GoldilocksField;
const D: usize = 2;

const METHOD_COUNT: F = GoldilocksField(1);
const METHOD_SQUARE: F = GoldilocksField(2); 

fn build_circuit(count_c: &CircuitData<F,C,2>, square_c: &CircuitData<F,C,2>) -> (CircuitData<F,C,2>, Target, ProofWithPublicInputsTarget<2>) {
    let mut bi = CircuitBuilder::<F, 2>::new(CircuitConfig::standard_recursion_config());
    let target_selector = bi.add_virtual_target();
    let method_count = bi.constant(METHOD_COUNT);
    let method_square = bi.constant(METHOD_SQUARE);

    let is_count = bi.is_equal(target_selector, method_count);
    let is_square = bi.is_equal(target_selector, method_square);

    let proof_with_public_inputs = bi.add_virtual_proof_with_pis(&count_c.common);

    let c_target = bi.constant_verifier_data(&count_c.verifier_only);
    bi.conditionally_verify_proof_or_dummy::<C>(is_count, &proof_with_public_inputs, &c_target, &count_c.common).unwrap();

    let s_target = bi.constant_verifier_data(&square_c.verifier_only);

    bi.conditionally_verify_proof_or_dummy::<C>(is_square, &proof_with_public_inputs, &s_target, &square_c.common).unwrap();


    bi.register_public_inputs(&proof_with_public_inputs.public_inputs);

    (bi.build::<C>(), target_selector, proof_with_public_inputs)
}

fn recursive_verifier_circuit(base_data: &CircuitData<F, C, 2>) -> (CircuitData<F, C, 2>, ProofWithPublicInputsTarget<2>) {
    let mut builder = CircuitBuilder::<F, 2>::new(CircuitConfig::standard_recursion_config());

    let proof = builder.add_virtual_proof_with_pis(&base_data.common);
    let verifier_data: VerifierCircuitTarget = builder.constant_verifier_data(&base_data.verifier_only);

    builder.verify_proof::<C>(&proof, &verifier_data, &base_data.common);
    (builder.build::<C>(), proof)

}

fn count_circuit() -> (CircuitData<F,C,2>, Target, Target) {
    let mut builder = CircuitBuilder::<F, 2>::new(CircuitConfig::standard_recursion_config());

    let x = builder.add_virtual_target();
    let y = builder.add_virtual_target();

    let one = builder.sub(y, x);
    builder.assert_one(one);
    (builder.build::<C>(), x, y)
}

fn square_circuit() -> (CircuitData<F,C,2>, Target, Target) {
    let mut builder = CircuitBuilder::<F, 2>::new(CircuitConfig::standard_recursion_config());

    let x = builder.add_virtual_target();
    let y = builder.add_virtual_target();

    let s = builder.square(x);
    builder.connect(s, y); 
    (builder.build::<C>(), x, y)
}

#[test]
fn main() {
    println!("Test Sub circuit");
    let (count_data, x_target, y_target) = count_circuit();
    let (square_data, x_t, y_t) = square_circuit();
    let (circuit_data, target_selector, proof_b_target) = build_circuit(&count_data, &square_data);

    // Test count circuit
    let mut wi = PartialWitness::<GoldilocksField>::new();
    wi.set_target(x_target, F::from_canonical_u8(1));
    wi.set_target(y_target, F::from_canonical_u8(2));
    let proof = count_data.prove(wi).unwrap();
    println!("Count proof public inputs: {:?}", proof.public_inputs);
    assert!(count_data.verify(proof.clone()).is_ok());
    // Test build circuit
    let mut wi = PartialWitness::<GoldilocksField>::new();
    wi.set_target(target_selector, METHOD_COUNT);
    wi.set_proof_with_pis_target(&proof_b_target, &proof).unwrap();
    let final_proof = circuit_data.prove(wi).unwrap();
    println!("Final proof public inputs: {:?}", final_proof.public_inputs);
    assert!(circuit_data.verify(final_proof).is_ok());

    //Test Square circuit
    let mut wi = PartialWitness::<GoldilocksField>::new();
    wi.set_target(x_t, F::from_canonical_u8(2));
    wi.set_target(y_t, F::from_canonical_u8(4));
    let proof = square_data.prove(wi).unwrap();
    println!("Square proof public inputs: {:?}", proof.public_inputs);
    assert!(square_data.verify(proof.clone()).is_ok());

    // Test build circuit
    let mut wi = PartialWitness::<GoldilocksField>::new();
    wi.set_target(target_selector, METHOD_SQUARE);
    wi.set_proof_with_pis_target(&proof_b_target, &proof).unwrap();
    let final_proof = circuit_data.prove(wi).unwrap();
    println!("Final proof public inputs: {:?}", final_proof.public_inputs);
    assert!(circuit_data.verify(final_proof).is_ok());
}