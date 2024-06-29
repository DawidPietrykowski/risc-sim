#[cfg(test)]
mod tests {
    use crate::*;

    use isa::cpu::*;
    use isa::rv32i::integer_reg_reg::*;
    use isa::types::*;
    use utils::binary_utils::*;

    use core::result::Result::Ok;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_encode_decode_i16(rd in 1u8..30, rs1 in 1u8..30, imm in -2048i16..2047){
            println!();
            let addi_instruction = IInstructionData {
                rd,
                func3: 0,
                rs1,
                imm: i16_to_u16(imm) & 0xFFF,
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
        fn test_encode_decode(rd in 1u8..30, rs1 in 1u8..30, imm1 in 0u16..4095){
            let addi_instruction = IInstructionData {
                rd,
                func3: 0,
                rs1,
                imm: imm1,
            };
            let op = encode_program_line("ADDI", InstructionData::I(addi_instruction)).unwrap();
            let decoded = decode_program_line(op).unwrap();
            prop_assert_eq!(parse_instruction_i(&decoded.word), addi_instruction);
        }

        #[test]
        fn test_addi(rd in 1u8..30, rs1 in 1u8..30, imm1 in -2048i16..2047, imm2 in -2048i16..2047) {
            let mut cpu = Cpu::new();
            let mut addi_instruction = IInstructionData {
                rd,
                func3: 0,
                rs1,
                imm: i16_to_u16(imm1),
            };
            let op = encode_program_line("ADDI", InstructionData::I(addi_instruction)).unwrap();
            prop_assert!(cpu.execute_word(op).is_ok());
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), imm1 as i32);

            addi_instruction.rs1 = rd;
            addi_instruction.imm = i16_to_u16(imm2);
            let rs1_val = cpu.read_x_i32(addi_instruction.rs1).unwrap();

            let op = encode_program_line("ADDI", InstructionData::I(addi_instruction)).unwrap();
            prop_assert!(cpu.execute_word(op).is_ok());
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), imm2 as i32 + rs1_val);
        }

        #[test]
        fn test_stli(rd in 1u8..30, rs1 in 1u8..30, imm1 in -2048i16..2047, imm2 in -2048i16..2047){
            let mut cpu = Cpu::new();

            let mut slti_instruction = IInstructionData {
                rd: rd,
                func3: 0,
                rs1: rs1,
                imm: i16_to_u16(imm1),
            };
            let rs1_val = cpu.read_x_i32(slti_instruction.rs1).unwrap();
            let slti_op = encode_program_line("SLTI", InstructionData::I(slti_instruction)).unwrap();

            prop_assert!(cpu.execute_word(slti_op).is_ok());
            prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), if rs1_val < imm1 as i32 {1} else {0});


            slti_instruction.rs1 = rd;
            slti_instruction.imm = i16_to_u16(imm2);
            let rs1_val = cpu.read_x_i32(slti_instruction.rs1).unwrap();

            let slti_op = encode_program_line("SLTI", InstructionData::I(slti_instruction)).unwrap();
            prop_assert!(cpu.execute_word(slti_op).is_ok());
            prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), if rs1_val < imm2 as i32 {1} else {0});
        }

        #[test] // TODO: Verify this test
        fn test_andi(rd in 1u8..30, rs1 in 1u8..30, imm1 in 0u16..0xFFFF, imm2 in 0u16..0xFFFF){
            let mut cpu = Cpu::new();

            let mut andi_instruction = IInstructionData {
                rd: rd,
                func3: 0,
                rs1: rs1,
                imm: imm1,
            };
            let rs1_val = cpu.read_x_u32(andi_instruction.rs1).unwrap();
            let op = encode_program_line("ANDI", InstructionData::I(andi_instruction)).unwrap();
            prop_assert!(cpu.execute_word(op).is_ok());
            prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), rs1_val & i32_to_u32(sign_extend_12bit_to_32bit(imm1)));

            andi_instruction.rs1 = rd;
            andi_instruction.imm = imm2;
            let rs1_val = cpu.read_x_u32(andi_instruction.rs1).unwrap();

            let op = encode_program_line("ANDI", InstructionData::I(andi_instruction)).unwrap();
            prop_assert!(cpu.execute_word(op).is_ok());
            prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), rs1_val & i32_to_u32(sign_extend_12bit_to_32bit(imm2)));
        }

        #[test]
        fn test_ori(rd in 1u8..30, rs1 in 1u8..30, imm1 in 0u16..0x0FFF, imm2 in 0u16..0x0FFF){
            let mut cpu = Cpu::new();

            let mut ori_instruction = IInstructionData {
                rd: rd,
                func3: 0,
                rs1: rs1,
                imm: imm1,
            };
            let rs1_val = cpu.read_x_u32(ori_instruction.rs1).unwrap();
            let op = encode_program_line("ORI", InstructionData::I(ori_instruction)).unwrap();
            prop_assert!(cpu.execute_word(op).is_ok());
            prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), rs1_val | i32_to_u32(sign_extend_12bit_to_32bit(imm1)));

            ori_instruction.rs1 = rd;
            ori_instruction.imm = imm2;
            let rs1_val = cpu.read_x_u32(ori_instruction.rs1).unwrap();

            let op = encode_program_line("ORI", InstructionData::I(ori_instruction)).unwrap();
            prop_assert!(cpu.execute_word(op).is_ok());
            prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), rs1_val | i32_to_u32(sign_extend_12bit_to_32bit(imm2)));
        }

        #[test]
        fn test_xori(rd in 1u8..30, rs1 in 1u8..30, imm1 in 0u16..0x0FFF, imm2 in 0u16..0x0FFF, rs1_val in i32::MIN..i32::MAX) {
            let mut cpu = Cpu::new();

            let mut xori_instruction = IInstructionData{
                rd,
                func3: 0,
                rs1,
                imm: imm1,
            };

            cpu.write_x_i32(rs1, rs1_val).unwrap();

            let op = encode_program_line("XORI", InstructionData::I(xori_instruction)).unwrap();
            prop_assert!(cpu.execute_word(op).is_ok());
            prop_assert_eq!(
                cpu.read_x_i32(rd).unwrap(),
                rs1_val ^ sign_extend_12bit_to_32bit(imm1)
            );

            xori_instruction.rs1 = rd;
            xori_instruction.imm = imm2;
            let rs1_val = cpu.read_x_i32(xori_instruction.rs1).unwrap();

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
                rd,
                func3: 0,
                rs1,
                imm: shamt as u16,
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
                rd,
                func3: 0,
                rs1,
                imm: shamt as u16,
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
                rd,
                func3: 0,
                rs1,
                imm: (0x400 | (shamt as u16)), // Set the 10th bit to differentiate from SRLI
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
                rd,
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
                rd,
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
            let mut cpu = Cpu::new();

            let add_instruction = RInstructionData {
                rd,
                func3: 0,
                rs1,
                rs2,
                func7: 0,
            };

            cpu.write_x_i32(rs1, rs1_val).unwrap();
            cpu.write_x_i32(rs2, rs2_val).unwrap();

            let rs1_read_val = cpu.read_x_i32(rs1).unwrap();
            let rs2_read_val = cpu.read_x_i32(rs2).unwrap();

            let add_op = Add::new(add_instruction);
            prop_assert!(cpu.execute_operation(&add_op).is_ok());

            let (expected, _) = rs1_read_val.overflowing_add(rs2_read_val);
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), expected);
        }

        #[test]
        fn test_sub(rd in 1u8..30, rs1 in 1u8..30, rs2 in 1u8..30, rs1_val in i32::MIN..i32::MAX, rs2_val in i32::MIN..i32::MAX) {
            let mut cpu = Cpu::new();

            let sub_instruction = RInstructionData {
                rd,
                func3: 0,
                rs1,
                rs2,
                func7: 0,
            };

            cpu.write_x_i32(rs1, rs1_val).unwrap();
            cpu.write_x_i32(rs2, rs2_val).unwrap();

            let rs1_read_val = cpu.read_x_i32(rs1).unwrap();
            let rs2_read_val = cpu.read_x_i32(rs2).unwrap();

            let sub_op = Sub::new(sub_instruction);
            prop_assert!(cpu.execute_operation(&sub_op).is_ok());

            let (expected, _) = rs1_read_val.overflowing_sub(rs2_read_val);
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), expected);
        }

        #[test]
        fn test_slt(rd in 1u8..30, rs1 in 1u8..30, rs2 in 1u8..30, rs1_val in i32::MIN..i32::MAX, rs2_val in i32::MIN..i32::MAX) {
            let mut cpu = Cpu::new();

            let slt_instruction = RInstructionData {
                rd,
                func3: 0,
                rs1,
                rs2,
                func7: 0,
            };

            cpu.write_x_i32(rs1, rs1_val).unwrap();
            cpu.write_x_i32(rs2, rs2_val).unwrap();

            let rs1_read_val = cpu.read_x_i32(rs1).unwrap();
            let rs2_read_val = cpu.read_x_i32(rs2).unwrap();

            let slt_op = SLT::new(slt_instruction);
            prop_assert!(cpu.execute_operation(&slt_op).is_ok());

            let expected = if rs1_read_val < rs2_read_val {1} else {0};
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), expected);
        }

        #[test]
        fn test_sltu(rd in 1u8..30, rs1 in 1u8..30, rs2 in 1u8..30, rs1_val in i32::MIN..i32::MAX, rs2_val in i32::MIN..i32::MAX) {
            let mut cpu = Cpu::new();

            let sltu_instruction = RInstructionData {
                rd,
                func3: 0,
                rs1,
                rs2,
                func7: 0,
            };

            cpu.write_x_u32(rs1, rs1_val as u32).unwrap();
            cpu.write_x_u32(rs2, rs2_val as u32).unwrap();

            let rs1_read_val = cpu.read_x_u32(rs1).unwrap();
            let rs2_read_val = cpu.read_x_u32(rs2).unwrap();

            let sltu_op = SLTU::new(sltu_instruction);
            prop_assert!(cpu.execute_operation(&sltu_op).is_ok());

            let expected = if rs1_read_val < rs2_read_val {1} else {0};
            prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), expected);
        }

        #[test]
        fn test_xor(rd in 1u8..30, rs1 in 1u8..30, rs2 in 1u8..30, rs1_val in i32::MIN..i32::MAX, rs2_val in i32::MIN..i32::MAX) {
            let mut cpu = Cpu::new();

            let xor_instruction = RInstructionData {
                rd,
                func3: 0,
                rs1,
                rs2,
                func7: 0,
            };

            cpu.write_x_i32(rs1, rs1_val).unwrap();
            cpu.write_x_i32(rs2, rs2_val).unwrap();

            let rs1_read_val = cpu.read_x_i32(rs1).unwrap();
            let rs2_read_val = cpu.read_x_i32(rs2).unwrap();

            let xor_op = XOR::new(xor_instruction);
            prop_assert!(cpu.execute_operation(&xor_op).is_ok());

            let expected = rs1_read_val ^ rs2_read_val;
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), expected);
        }

        #[test]
        fn test_or(rd in 1u8..30, rs1 in 1u8..30, rs2 in 1u8..30, rs1_val in i32::MIN..i32::MAX, rs2_val in i32::MIN..i32::MAX) {
            let mut cpu = Cpu::new();

            let or_instruction = RInstructionData {
                rd,
                func3: 0,
                rs1,
                rs2,
                func7: 0,
            };

            cpu.write_x_i32(rs1, rs1_val).unwrap();
            cpu.write_x_i32(rs2, rs2_val).unwrap();

            let rs1_read_val = cpu.read_x_i32(rs1).unwrap();
            let rs2_read_val = cpu.read_x_i32(rs2).unwrap();

            let or_op = OR::new(or_instruction);
            prop_assert!(cpu.execute_operation(&or_op).is_ok());

            let expected = rs1_read_val | rs2_read_val;
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), expected);
        }

        #[test]
        fn test_and(rd in 1u8..30, rs1 in 1u8..30, rs2 in 1u8..30, rs1_val in i32::MIN..i32::MAX, rs2_val in i32::MIN..i32::MAX) {
            let mut cpu = Cpu::new();

            let and_instruction = RInstructionData {
                rd,
                func3: 0,
                rs1,
                rs2,
                func7: 0,
            };

            cpu.write_x_i32(rs1, rs1_val).unwrap();
            cpu.write_x_i32(rs2, rs2_val).unwrap();

            let rs1_read_val = cpu.read_x_i32(rs1).unwrap();
            let rs2_read_val = cpu.read_x_i32(rs2).unwrap();

            let and_op = AND::new(and_instruction);
            prop_assert!(cpu.execute_operation(&and_op).is_ok());

            let expected = rs1_read_val & rs2_read_val;
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), expected);
        }

        #[test]
        fn test_sll(rd in 1u8..30, rs1 in 1u8..30, rs2 in 1u8..30, rs1_val in u32::MIN..u32::MAX, rs2_val in 0u32..31) {
            let mut cpu = Cpu::new();

            let sll_instruction = RInstructionData {
                rd,
                func3: 0,
                rs1,
                rs2,
                func7: 0,
            };

            cpu.write_x_u32(rs1, rs1_val).unwrap();
            cpu.write_x_u32(rs2, rs2_val).unwrap();

            let rs1_read_val = cpu.read_x_u32(rs1).unwrap();
            let rs2_read_val = cpu.read_x_u32(rs2).unwrap();

            let sll_op = SLL::new(sll_instruction);
            prop_assert!(cpu.execute_operation(&sll_op).is_ok());

            let expected = rs1_read_val << (rs2_read_val & 0b11111);
            prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), expected);
        }

        #[test]
        fn test_srl(rd in 1u8..30, rs1 in 1u8..30, rs2 in 1u8..30, rs1_val in u32::MIN..u32::MAX, rs2_val in 0u32..31) {
            let mut cpu = Cpu::new();

            let sll_instruction = RInstructionData {
                rd,
                func3: 0,
                rs1,
                rs2,
                func7: 0,
            };

            cpu.write_x_u32(rs1, rs1_val).unwrap();
            cpu.write_x_u32(rs2, rs2_val).unwrap();

            let rs1_read_val = cpu.read_x_u32(rs1).unwrap();
            let rs2_read_val = cpu.read_x_u32(rs2).unwrap();

            let sll_op = SRL::new(sll_instruction);
            prop_assert!(cpu.execute_operation(&sll_op).is_ok());

            let expected = rs1_read_val >> (rs2_read_val & 0b11111);
            prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), expected);
        }

        #[test]
        fn test_sra(rd in 1u8..30, rs1 in 1u8..30, rs2 in 1u8..30, rs1_val in i32::MIN..i32::MAX, rs2_val in 0u32..31) {
            let mut cpu = Cpu::new();

            let sll_instruction = RInstructionData {
                rd,
                func3: 0,
                rs1,
                rs2,
                func7: 0,
            };

            cpu.write_x_i32(rs1, rs1_val).unwrap();
            cpu.write_x_u32(rs2, rs2_val).unwrap();

            let rs1_read_val = cpu.read_x_i32(rs1).unwrap();
            let rs2_read_val = cpu.read_x_u32(rs2).unwrap();

            let sll_op = SRA::new(sll_instruction);
            prop_assert!(cpu.execute_operation(&sll_op).is_ok());

            let expected = rs1_read_val >> (rs2_read_val & 0b11111);
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
