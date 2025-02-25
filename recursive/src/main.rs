pub mod test;

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
        prover::prove,
    },
    util::{timing::TimingTree, serialization::DefaultGateSerializer},
};
use log::{info, Level, LevelFilter};

type F = GoldilocksField;
type C = PoseidonGoldilocksConfig;
const D: usize = 2;

// 第 1 层 ZK 证明：证明 1 + 1 = 2
fn build_addition_circuit() -> (CircuitBuilder<F, 2>, Target, Target, Target) {
    let mut builder = CircuitBuilder::<F, 2>::new(CircuitConfig::standard_recursion_config());

    // 创建三个虚拟输入变量
    let x = builder.add_virtual_target();
    let y = builder.add_virtual_target();
    let z = builder.add_virtual_target();

    // 约束： x + y = z
    let sum = builder.add(x, y);
    builder.connect(sum, z);

    // 注册公共输入
    builder.register_public_input(x);
    builder.register_public_input(y);
    builder.register_public_input(z);

    (builder, x, y, z)
}

// 第 2 层 ZK 证明：验证 1 + 2 = 3
fn main() {
    // ============ 第 1 层 ZK 证明：证明 1 + 1 = 2 ============
    let (inner_builder, x, y, z) = build_addition_circuit();
    let inner_data = inner_builder.build::<C>();
    
    let mut witness =  PartialWitness::new();
    witness.set_target(x, F::from_canonical_u64(1));
    witness.set_target(y, F::from_canonical_u64(1));
    witness.set_target(z, F::from_canonical_u64(2));

    let inner_proof = inner_data.prove(witness).unwrap();

    assert!(inner_data.verifier_data().verify(inner_proof.clone()).is_ok());
    println!("✅ 第 1 层 ZK 证明（1 + 1 = 2）验证成功！");

    // ============ 第 2 层 ZK 证明：验证 1 + 2 = 3 ============
    let mut outer_builder = CircuitBuilder::<F, 2>::new(CircuitConfig::standard_recursion_config());

    // 创建外部电路，添加验证目标（Verifier Target）
    let proof_target = outer_builder.add_virtual_proof_with_pis(&inner_data.common);
    let vk_target = outer_builder.add_virtual_verifier_data(inner_data.common.config.fri_config.cap_height);

    // let outer_data = outer_builder.build::<C>();

    // 约束：验证第 1 层的 ZK 证明
    outer_builder.verify_proof::<C>(&proof_target, &vk_target, &inner_data.common);

    inner_proof.public_inputs.iter().for_each(|input| println!("Public inner_proof: {:?}", input));

    let outer_data = outer_builder.build::<C>();

    let mut outer_witness =  PartialWitness::new();
    outer_witness.set_proof_with_pis_target(&proof_target, &inner_proof);
    outer_witness.set_verifier_data_target(&vk_target, &inner_data.verifier_only);

    // 创建外部证明
    let mut timing = TimingTree::new("prove", Level::Debug);
    let outer_proof =  match outer_data.prove(outer_witness) {
        Ok(proof) => proof,
        Err(e) => panic!("Failed to prove: {:?}", e),
    };
    timing.print();

    outer_proof.public_inputs.iter().for_each(|input| println!("Public outer_proof: {:?}", input));

    assert!(outer_data.verify(outer_proof.clone()).is_ok());

    println!("🎉 嵌套 ZK 证明完成！");
}

