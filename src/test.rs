#[cfg(test)]
mod tests {
    use crate::*;

    use anyhow::Result;
    use isa::cpu::*;
    use isa::types::*;
    use proptest::prelude::*;
    use std::result::Result::Ok;
    use utils::binary_utils::*;

    fn setup_cpu() -> Cpu {
        Cpu::new()
    }

    fn execute_i_instruction(cpu: &mut Cpu, opcode: &str, rd: u8, rs1: u8, imm: u16) -> Result<()> {
        let instruction = IInstructionData {
            rd: U5(rd),
            rs1: U5(rs1),
            imm: U12(imm),
            ..Default::default()
        };
        let op = encode_program_line(opcode, InstructionData::I(instruction))?;
        cpu.execute_word(op)?;
        Ok(())
    }

    fn execute_u_instruction(cpu: &mut Cpu, opcode: &str, rd: u8, imm: u32) -> Result<()> {
        let instruction = UInstructionData {
            rd: U5(rd),
            imm: imm,
            ..Default::default()
        };
        let op = encode_program_line(opcode, InstructionData::U(instruction))?;
        cpu.execute_word(op)?;
        Ok(())
    }

    macro_rules! test_instruction_i {
        ($name:ident, $opcode:expr, $test_logic:expr) => {
            proptest! {
                #[test]
                fn $name(rd in 1u8..30, rs1 in 1u8..30, imm in -2048i16..2047){
                    let mut cpu = setup_cpu();
                    prop_assert!(execute_i_instruction(&mut cpu, $opcode, rd, rs1, i16_to_u16(imm)).is_ok());
                    $test_logic(&mut cpu, rd, rs1, imm).unwrap();
                }
            }
        };
    }

    macro_rules! test_instruction_u {
        ($name:ident, $opcode:expr, $test_logic:expr) => {
            proptest! {
                #[test]
                fn $name(rd in 1u8..30, imm in 0u32..0xFFFFFFFF){
                    let mut cpu = setup_cpu();
                    prop_assert!(execute_u_instruction(&mut cpu, $opcode, rd, imm).is_ok());
                    $test_logic(&mut cpu, rd, imm).unwrap();
                }
            }
        };
    }

    macro_rules! test_instruction_r {
        ($name:ident, $opcode:expr, $test_logic:expr) => {
            proptest! {
                #[test]
                fn $name(rd in 1u8..30, rs1 in 1u8..30, rs2 in 1u8..30, rs1_val in i32::MIN..i32::MAX, rs2_val in i32::MIN..i32::MAX){
                    let mut cpu = setup_cpu();

                    let instruction = RInstructionData {
                        rd: U5(rd),
                        rs1: U5(rs1),
                        rs2: U5(rs2),
                        ..Default::default()
                    };

                    cpu.write_x_i32(rs1, rs1_val).unwrap();
                    cpu.write_x_i32(rs2, rs2_val).unwrap();

                    let rs1_read_val = cpu.read_x_i32(rs1).unwrap();
                    let rs2_read_val = cpu.read_x_i32(rs2).unwrap();

                    let op = encode_program_line($opcode, InstructionData::R(instruction)).unwrap();
                    prop_assert!(cpu.execute_word(op).is_ok());

                    $test_logic(&mut cpu, rd, rs1, rs2, rs1_read_val, rs2_read_val).unwrap();
                }
            }
        };
    }

    test_instruction_i!(test_addi2, "ADDI", |cpu: &mut Cpu, rd, rs1, imm| {
        prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), imm as i32);
        Ok(())
    });

    test_instruction_u!(test_lui2, "LUI", |cpu: &mut Cpu, rd, imm| {
        let expected = (imm << 12) as u32;
        prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), expected);
        Ok(())
    });

    test_instruction_r!(
        test_add2,
        "ADD",
        |cpu: &mut Cpu, rd, rs1, rs2, rs1_read_val: i32, rs2_read_val: i32| {
            let (expected, _) = rs1_read_val.overflowing_add(rs2_read_val);
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), expected);
            Ok(())
        }
    );

    test_instruction_r!(
        test_sub,
        "SUB",
        |cpu: &mut Cpu, rd, rs1, rs2, rs1_read_val: i32, rs2_read_val: i32| {
            let (expected, _) = rs1_read_val.overflowing_sub(rs2_read_val);
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), expected);
            Ok(())
        }
    );

    test_instruction_r!(
        test_slt,
        "SLT",
        |cpu: &mut Cpu, rd, rs1, rs2, rs1_read_val: i32, rs2_read_val: i32| {
            let expected = if rs1_read_val < rs2_read_val { 1 } else { 0 };
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), expected);
            Ok(())
        }
    );

    test_instruction_r!(
        test_sltu,
        "SLTU",
        |cpu: &mut Cpu, rd, rs1, rs2, rs1_read_val: i32, rs2_read_val: i32| {
            let expected = if i32_to_u32(rs1_read_val) < i32_to_u32(rs2_read_val) {
                1
            } else {
                0
            };
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), expected);
            Ok(())
        }
    );

    test_instruction_r!(
        test_xor,
        "XOR",
        |cpu: &mut Cpu, rd, rs1, rs2, rs1_read_val: i32, rs2_read_val: i32| {
            let expected = rs1_read_val ^ rs2_read_val;
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), expected);
            Ok(())
        }
    );

    test_instruction_r!(
        test_or,
        "OR",
        |cpu: &mut Cpu, rd, rs1, rs2, rs1_read_val: i32, rs2_read_val: i32| {
            let expected = rs1_read_val | rs2_read_val;
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), expected);
            Ok(())
        }
    );

    test_instruction_r!(
        test_and,
        "AND",
        |cpu: &mut Cpu, rd, rs1, rs2, rs1_read_val: i32, rs2_read_val: i32| {
            let expected = rs1_read_val & rs2_read_val;
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), expected);
            Ok(())
        }
    );

    test_instruction_r!(
        test_sll,
        "SLL",
        |cpu: &mut Cpu, rd, rs1, rs2, rs1_read_val: i32, rs2_read_val: i32| {
            let expected = (i32_to_u32(rs1_read_val)) << (i32_to_u32(rs2_read_val) & 0b11111);
            prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), expected);
            Ok(())
        }
    );

    test_instruction_r!(
        test_srl,
        "SRL",
        |cpu: &mut Cpu, rd, rs1, rs2, rs1_read_val: i32, rs2_read_val: i32| {
            let expected = (i32_to_u32(rs1_read_val)) >> (i32_to_u32(rs2_read_val) & 0b11111);
            prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), expected);
            Ok(())
        }
    );

    test_instruction_r!(
        test_sra,
        "SRA",
        |cpu: &mut Cpu, rd, rs1, rs2, rs1_read_val: i32, rs2_read_val: i32| {
            let expected = (rs1_read_val) >> (i32_to_u32(rs2_read_val) & 0b11111);
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), expected);
            Ok(())
        }
    );

    proptest! {
        #[test]
        fn test_encode_decode_i16(rd in 1u8..30, rs1 in 1u8..30, immi16 in -2048i16..2047){
            println!();
            let imm = U12(i16_to_u16(immi16) & 0xFFF);
            let addi_instruction = IInstructionData {
                rd: U5(rd),
                rs1: U5(rs1),
                imm: imm,
                ..Default::default()
            };
            let op = encode_program_line("ADDI", InstructionData::I(addi_instruction)).unwrap();
            let decoded = decode_program_line(op).unwrap();
            // println!("{:#034b}", -2048 as i16);
            // println!("{:#034b}", i16_to_u16(-2048));
            // println!("{:#034b}", (-2048 as i32) << 20);
            // println!("{:#034b}", (i16_to_u16(-2048) as u32) << 20);
            // println!("{:#034b}", parse_instruction_i(&decoded.word).imm);
            prop_assert_eq!(parse_instruction_i(&decoded.word), addi_instruction);
        }

        #[test]
        fn test_encode_decode(rd in 1u8..30, rs1 in 1u8..30, imm_u16 in 0u16..4095){
            let imm = U12(imm_u16);
            let addi_instruction = IInstructionData {
                rd: U5(rd),
                rs1: U5(rs1),
                imm: imm,
                ..Default::default()
            };
            let op = encode_program_line("ADDI", InstructionData::I(addi_instruction)).unwrap();
            let decoded = decode_program_line(op).unwrap();
            prop_assert_eq!(parse_instruction_i(&decoded.word), addi_instruction);
        }

        #[test]
        fn test_addi(rd in 1u8..30, rs1 in 1u8..30, imm1 in -2048i16..2047, imm2 in -2048i16..2047) {
            let mut cpu = Cpu::new();
            let mut addi_instruction = IInstructionData {
                rd: U5(rd),
                rs1: U5(rs1),
                imm: U12(i16_to_u16(imm1)),
                ..Default::default()
            };
            let op = encode_program_line("ADDI", InstructionData::I(addi_instruction)).unwrap();
            prop_assert!(cpu.execute_word(op).is_ok());
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), imm1 as i32);

            addi_instruction.rs1 = U5(rd);
            addi_instruction.imm = U12(i16_to_u16(imm2));
            let rs1_val = cpu.read_x_i32(addi_instruction.rs1.value()).unwrap();

            let op = encode_program_line("ADDI", InstructionData::I(addi_instruction)).unwrap();
            prop_assert!(cpu.execute_word(op).is_ok());
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), imm2 as i32 + rs1_val);
        }

        #[test]
        fn test_stli(rd in 1u8..30, rs1 in 1u8..30, imm1 in -2048i16..2047, imm2 in -2048i16..2047){
            let mut cpu = Cpu::new();
            let mut slti_instruction = IInstructionData {
                rd: U5(rd),
                rs1: U5(rs1),
                imm: U12(i16_to_u16(imm1)),
                ..Default::default()
            };
            let rs1_val = cpu.read_x_i32(slti_instruction.rs1.value()).unwrap();
            let slti_op = encode_program_line("SLTI", InstructionData::I(slti_instruction)).unwrap();

            prop_assert!(cpu.execute_word(slti_op).is_ok());
            prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), if rs1_val < imm1 as i32 {1} else {0});


            slti_instruction.rs1 = U5(rd);
            slti_instruction.imm = U12(i16_to_u16(imm2));
            let rs1_val = cpu.read_x_i32(slti_instruction.rs1.value()).unwrap();

            let slti_op = encode_program_line("SLTI", InstructionData::I(slti_instruction)).unwrap();
            prop_assert!(cpu.execute_word(slti_op).is_ok());
            prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), if rs1_val < imm2 as i32 {1} else {0});
        }

        #[test] // TODO: Verify this test
        fn test_andi(rd in 1u8..30, rs1 in 1u8..30, imm1 in 0u16..0xFFFF, imm2 in 0u16..0xFFFF){
            let mut cpu = Cpu::new();
            let mut andi_instruction = IInstructionData {
                rd: U5(rd),
                rs1: U5(rs1),
                imm: U12(imm1),
                ..Default::default()
            };
            let rs1_val = cpu.read_x_u32(andi_instruction.rs1.value()).unwrap();
            let op = encode_program_line("ANDI", InstructionData::I(andi_instruction)).unwrap();
            prop_assert!(cpu.execute_word(op).is_ok());
            prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), rs1_val & i32_to_u32(sign_extend_12bit_to_32bit(imm1)));

            andi_instruction.rs1 = U5(rd);
            andi_instruction.imm = U12(imm2);
            let rs1_val = cpu.read_x_u32(andi_instruction.rs1.value()).unwrap();

            let op = encode_program_line("ANDI", InstructionData::I(andi_instruction)).unwrap();
            prop_assert!(cpu.execute_word(op).is_ok());
            prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), rs1_val & i32_to_u32(sign_extend_12bit_to_32bit(imm2)));
        }

        #[test]
        fn test_ori(rd in 1u8..30, rs1 in 1u8..30, imm1 in 0u16..0x0FFF, imm2 in 0u16..0x0FFF){
            let mut cpu = Cpu::new();
            let mut ori_instruction = IInstructionData {
                rd: U5(rd),
                rs1: U5(rs1),
                imm: U12(imm1),
                ..Default::default()
            };
            let rs1_val = cpu.read_x_u32(ori_instruction.rs1.value()).unwrap();
            let op = encode_program_line("ORI", InstructionData::I(ori_instruction)).unwrap();
            prop_assert!(cpu.execute_word(op).is_ok());
            prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), rs1_val | i32_to_u32(sign_extend_12bit_to_32bit(imm1)));

            ori_instruction.rs1 = U5(rd);
            ori_instruction.imm = U12(imm2);
            let rs1_val = cpu.read_x_u32(ori_instruction.rs1.value()).unwrap();

            let op = encode_program_line("ORI", InstructionData::I(ori_instruction)).unwrap();
            prop_assert!(cpu.execute_word(op).is_ok());
            prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), rs1_val | i32_to_u32(sign_extend_12bit_to_32bit(imm2)));
        }

        #[test]
        fn test_xori(rd in 1u8..30, rs1 in 1u8..30, imm1 in 0u16..0x0FFF, imm2 in 0u16..0x0FFF, rs1_val in i32::MIN..i32::MAX) {
            let mut cpu = Cpu::new();
            let mut xori_instruction = IInstructionData{
                rd: U5(rd),
                rs1: U5(rs1),
                imm: U12(imm1),
                ..Default::default()
            };

            cpu.write_x_i32(rs1, rs1_val).unwrap();

            let op = encode_program_line("XORI", InstructionData::I(xori_instruction)).unwrap();
            prop_assert!(cpu.execute_word(op).is_ok());
            prop_assert_eq!(
                cpu.read_x_i32(rd).unwrap(),
                rs1_val ^ sign_extend_12bit_to_32bit(imm1)
            );

            xori_instruction.rs1 = U5(rd);
            xori_instruction.imm = U12(imm2);
            let rs1_val = cpu.read_x_i32(xori_instruction.rs1.value()).unwrap();

            let op = encode_program_line("XORI", InstructionData::I(xori_instruction)).unwrap();
            prop_assert!(cpu.execute_word(op).is_ok());
            prop_assert_eq!(
                cpu.read_x_i32(rd).unwrap(),
                rs1_val ^ sign_extend_12bit_to_32bit(imm2)
            );
        }


        #[test]
        fn test_slli(rd in 1u8..30, rs1 in 1u8..30, shamt in 0u8..31, rs1_val in i32::MIN..i32::MAX) {
            let mut cpu = Cpu::new();
            let slli_instruction = IInstructionData {
                rd: U5(rd),
                rs1: U5(rs1),
                imm: U12(shamt as u16),
                ..Default::default()
            };

            cpu.write_x_i32(rs1, rs1_val).unwrap();

            let op = encode_program_line("SLLI", InstructionData::I(slli_instruction)).unwrap();

            prop_assert!(cpu.execute_word(op).is_ok());

            let expected = (rs1_val as u32).wrapping_shl(shamt as u32) as i32;
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), expected);
        }

        #[test]
        fn test_srli(rd in 1u8..30, rs1 in 1u8..30, shamt in 0u8..31, rs1_val in i32::MIN..i32::MAX) {
            let mut cpu = Cpu::new();
            let srli_instruction = IInstructionData {
                rd: U5(rd),
                rs1: U5(rs1),
                imm: U12(shamt as u16),
                ..Default::default()
            };

            cpu.write_x_u32(rs1, rs1_val as u32).unwrap();

            let op = encode_program_line("SRLI", InstructionData::I(srli_instruction)).unwrap();

            prop_assert!(cpu.execute_word(op).is_ok());

            let expected = ((rs1_val as u32) >> shamt) as i32;
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), expected);
        }

        #[test]
        fn test_srai(rd in 1u8..30, rs1 in 1u8..30, shamt in 0u8..31, rs1_val in i32::MIN..i32::MAX) {
            let mut cpu = Cpu::new();

            let srai_instruction = IInstructionData {
                rd: U5(rd),
                rs1: U5(rs1),
                imm: U12(0x400 | (shamt as u16)), // Set the 10th bit to differentiate from SRLI
                ..Default::default()
            };

            cpu.write_x_i32(rs1, rs1_val).unwrap();

            let op = encode_program_line("SRAI", InstructionData::I(srai_instruction)).unwrap();

            prop_assert!(cpu.execute_word(op).is_ok());

            let expected = rs1_val >> shamt;
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), expected);
        }
        #[test]
        fn test_lui(rd in 1u8..30, imm in 0u32..0xFFFFF) {
            let mut cpu = Cpu::new();

            let lui_instruction = UInstructionData {
                rd: U5(rd),
                imm: imm,
            };

            let op = encode_program_line("LUI", InstructionData::U(lui_instruction)).unwrap();

            prop_assert!(cpu.execute_word(op).is_ok());

            let expected = (imm << 12) as u32;
            prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), expected);
        }

        #[test]
        fn test_auipc(rd in 1u8..30, imm in 0u32..0xFFFFF, pc in 0u32..0xFFFFFFFF) {
            let mut cpu = Cpu::new();
            cpu.write_pc_u32(pc);

            let auipc_instruction = UInstructionData {
                rd: U5(rd),
                imm: imm,
            };

            let op = encode_program_line("AUIPC", InstructionData::U(auipc_instruction)).unwrap();

            prop_assert!(cpu.execute_word(op).is_ok());

            let expected = (imm << 12).wrapping_add(pc);
            prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), expected);
            prop_assert_eq!(cpu.read_pc_u32(), expected);
        }

        #[test]
        fn test_add(rd in 1u8..30, rs1 in 1u8..30, rs2 in 1u8..30, rs1_val in i32::MIN..i32::MAX, rs2_val in i32::MIN..i32::MAX) {
            let mut cpu = setup_cpu();

            let instruction = RInstructionData {
                rd: U5(rd),
                rs1: U5(rs1),
                rs2: U5(rs2),
                ..Default::default()
            };

            cpu.write_x_i32(rs1, rs1_val).unwrap();
            cpu.write_x_i32(rs2, rs2_val).unwrap();

            let rs1_read_val = cpu.read_x_i32(rs1).unwrap();
            let rs2_read_val = cpu.read_x_i32(rs2).unwrap();

            let op = encode_program_line("ADD", InstructionData::R(instruction)).unwrap();
            prop_assert!(cpu.execute_word(op).is_ok());

            let (expected, _) = rs1_read_val.overflowing_add(rs2_read_val);
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), expected);
        }

        #[test]
        fn test_12bit_sign_extend_to_16bit(imm in -2048i16..2047){
            let imm = sign_extend_12bit_to_16bit(i16_to_u16(imm));
            prop_assert_eq!(imm, imm as i16);
        }

        #[test]
        fn test_12bit_sign_extend_to_32bit(imm in -2048i16..2047){
            let imm = sign_extend_12bit_to_32bit(i16_to_u16(imm));
            prop_assert_eq!(imm, imm as i32);
        }
        //  */
    }
}
