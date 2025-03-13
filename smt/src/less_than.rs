use plonky2::field::extension::Extendable;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::field::types::Field;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::target::{BoolTarget, Target};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::{CircuitConfig, CircuitData};
use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};
use plonky2::iop::witness::{PartialWitness, WitnessWrite};

// 假设的 P2Type 枚举（根据你的上下文推测）
#[derive(Debug, PartialEq)]
enum P2Type {
    Integer(usize), // 整数类型，带位大小
    Field,          // 字段类型
    Other,          // 其他类型
}

// 比较逻辑的实现

    // 获取整数类型和目标（模拟 get_integer）
    fn get_integer(value: Target) -> Result<(P2Type, Target), &'static str> {
        // 假设输入已经是 Target，类型为 Integer(32)
        Ok((P2Type::Integer(64), value))
    }

    // 获取位大小（参考你之前的代码）
    fn get_integer_bitsize(typ: &P2Type) -> Option<usize> {
        const FIELD_BIT_SIZE: usize = 64; // 假设字段大小为 64 位
        Some(
            usize::try_from(match typ {
                P2Type::Integer(bit_size) => *bit_size,
                P2Type::Field => FIELD_BIT_SIZE,
                _ => return None,
            })
            .unwrap(),
        )
    }

    // 小于比较电路
    fn less_than(
        builder : &mut CircuitBuilder<GoldilocksField, 2>,
        lhs: Target,
        rhs: Target,
    ) -> Result<BoolTarget, &'static str> {
        let (type_of_a, target_a) = get_integer(lhs)?;
        let (type_of_b, target_b) = get_integer(rhs)?;
        assert!(type_of_a == type_of_b);

        if let Some(bit_size) = get_integer_bitsize(&type_of_a) {
            let mut split_a = builder.split_le(target_a, bit_size);
            let mut split_b = builder.split_le(target_b, bit_size);

            split_a.reverse(); // 高位在前
            split_b.reverse();

            let mut first_i_minus_1_are_equal: Option<BoolTarget> = None;
            let mut result: Option<BoolTarget> = None;

            for i in 0..split_a.len() {
                let is_first = i == 0;
                let is_last = i == (split_a.len() - 1);

                let not_a_i = builder.not(split_a[i]);
                let not_a_i_and_b_i = builder.and(not_a_i, split_b[i]);

                let equal = builder.is_equal(split_a[i].target, split_b[i].target);

                let not_a_i_and_b_i_and_first_i_minus_1_equal = if is_first {
                    not_a_i_and_b_i
                } else {
                    builder
                        .and(not_a_i_and_b_i, first_i_minus_1_are_equal.unwrap())
                };

                result = if is_first {
                    Some(not_a_i_and_b_i_and_first_i_minus_1_equal)
                } else {
                    Some(builder.or(
                        result.unwrap(),
                        not_a_i_and_b_i_and_first_i_minus_1_equal,
                    ))
                };

                if !is_last {
                    first_i_minus_1_are_equal = if is_first {
                        Some(equal)
                    } else {
                        Some(builder.and(first_i_minus_1_are_equal.unwrap(), equal))
                    };
                }
            }

            Ok(result.unwrap())
        } else {
            Err("Invalid bit size")
        }
    }


#[test]
fn main() {
    // 配置
    type F = GoldilocksField;
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    let config = CircuitConfig::standard_recursion_config();

    // 创建比较器
    let mut builder = CircuitBuilder::<F, 2>::new(CircuitConfig::standard_recursion_zk_config());

    // 创建输入
    let a = builder.add_virtual_target();
    let b = builder.add_virtual_target();

    // 构建小于电路
    let result = less_than(&mut builder, a, b).unwrap();
    builder.assert_one(result.target);
    // 注册结果为公共输入
    builder.register_public_input(result.target);

    // 构建电路
    let circuit = builder.build::<C>();

    // 测试 1: a = 5, b = 10 (5 < 10)
    let mut pw = PartialWitness::new();
    pw.set_target(a, F::from_canonical_u64(5)).unwrap();
    pw.set_target(b, F::from_canonical_u64(10)).unwrap();
    let proof = circuit.prove(pw).unwrap();
    circuit.verify(proof.clone()).unwrap();

    // 测试 2: a = 10, b = 5 (10 < 5)
    let mut pw = PartialWitness::new();
    pw.set_target(a, F::from_canonical_u64(10)).unwrap();
    pw.set_target(b, F::from_canonical_u64(5)).unwrap();
    let proof = circuit.prove(pw).unwrap();
    circuit.verify(proof.clone()).unwrap();

    // 测试 3: a = 7, b = 7 (7 < 7)
    let mut pw = PartialWitness::new();
    pw.set_target(a, F::from_canonical_u64(7)).unwrap();
    pw.set_target(b, F::from_canonical_u64(6)).unwrap();
    let proof = circuit.prove(pw).unwrap();
    circuit.verify(proof).unwrap();

    println!("Less-than circuit works successfully!");
}