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

type F = GoldilocksField;
const D: usize = 2;
type C = PoseidonGoldilocksConfig;

#[test]
fn main() {
    let (base_data, x_target) = base_circuit();
    let (recursive_data, proof_target) = recursive_verifier_circuit(&base_data);

    let mut base_witness = PartialWitness::new();
    base_witness.set_target(x_target, F::from_canonical_u64(3));


    let base_proof = base_data.prove(base_witness).unwrap();
    println!("第一层证明！！");
    let mut recursive_witness = PartialWitness::new();
    recursive_witness.set_proof_with_pis_target(&proof_target, &base_proof);
    // recursive_witness.set_verifier_data_target(&verifier_data_target, &base_data.verifier_only);

    let recursive_proof = recursive_data.prove(recursive_witness).unwrap();
    recursive_data.verify(recursive_proof).unwrap();

    println!("第二层证明！！");

    let (base2_data, x_target, y_target) = base2_circuit();
    let mut base2_witness = PartialWitness::new();
    base2_witness.set_target(x_target, F::from_canonical_u64(4));
    base2_witness.set_target(y_target, F::from_canonical_u64(5));

    let base2_proof = base2_data.prove(base2_witness).unwrap();


    let (mut recursive2_data, proof2_target) = recursive_verifier_circuit(&base2_data);

    let mut recursive2_witness = PartialWitness::new();
    recursive2_witness.set_proof_with_pis_target(&proof2_target, &base2_proof);
    // recursive2_witness.set_verifier_data_target(&verifier_data_target, &base2_data.verifier_only);

    let  recursive2_proof = recursive2_data.prove(recursive2_witness).unwrap();

    // recursive2_data.common = recursive_data.common;
    recursive2_data.verify(recursive2_proof).unwrap();


    println!("嵌套证明完成！！！");
}

fn base_circuit() -> (CircuitData<F,C,2>, Target) {
    let mut builder = CircuitBuilder::<F, 2>::new(CircuitConfig::standard_recursion_zk_config());

    let x = builder.add_virtual_target();
    let y = builder.square(x);
    // Add your circuit here
    (builder.build::<C>(), x)
}

fn base2_circuit() -> (CircuitData<F,C,2>, Target, Target) {
    let mut builder = CircuitBuilder::<F, 2>::new(CircuitConfig::standard_recursion_zk_config());

    let x = builder.add_virtual_target();
    let y = builder.add_virtual_target();
    // Add your circuit here
    let one = builder.sub(y, x);
    builder.assert_one(one);
    (builder.build::<C>(), x, y)
}


fn recursive_verifier_circuit(base_data: &CircuitData<F, C, 2>) -> (CircuitData<F, C, 2>, ProofWithPublicInputsTarget<2>) {
    let mut builder = CircuitBuilder::<F, 2>::new(CircuitConfig::standard_recursion_config());

    let proof = builder.add_virtual_proof_with_pis(&base_data.common);
    // let verifier_data = builder.add_virtual_verifier_data(base_data.common.config.fri_config.cap_height);
    let verifier_data: VerifierCircuitTarget = builder.constant_verifier_data(&base_data.verifier_only);

    builder.verify_proof::<C>(&proof, &verifier_data, &base_data.common);
    (builder.build::<C>(), proof)

}