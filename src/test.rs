#[cfg(test)]
mod tests {
    use crate::*;
    use proptest::prelude::*;
    use core::result::Result::Ok;

    proptest! {
        #[test]
        fn test_addi(rd in 1u8..30, rs1 in 1u8..30, imm1 in -2048i16..2047, imm2 in -2048i16..2047) {
            let mut cpu = Cpu::new();
            let mut addi_instruction = IInstruction {
                rd,
                func3: 0,
                rs1,
                imm: i16_to_u16(imm1),
            };
            let addi_op = AddI::new(addi_instruction.clone());
            prop_assert!(cpu.execute_operation(&addi_op).is_ok());
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), imm1 as i32);

            addi_instruction.rs1 = rd;
            addi_instruction.imm = i16_to_u16(imm2);
            let rs1_val = cpu.read_x_i32(addi_instruction.rs1).unwrap();

            let addi_op = AddI::new(addi_instruction);
            prop_assert!(cpu.execute_operation(&addi_op).is_ok());
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), imm2 as i32 + rs1_val);
        }

        #[test]
        fn test_stli(rd in 1u8..30, rs1 in 1u8..30, imm1 in -2048i16..2047, imm2 in -2048i16..2047){
            let mut cpu = Cpu::new();

            let mut slti_instruction = IInstruction {
                rd: rd,
                func3: 2,
                rs1: rs1,
                imm: i16_to_u16(imm1),
            };
            let rs1_val = cpu.read_x_i32(slti_instruction.rs1).unwrap();
            let slti_op = SLTI::new(slti_instruction.clone());
            prop_assert!(cpu.execute_operation(&slti_op).is_ok());
            prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), if rs1_val < imm1 as i32 {1} else {0});
        

            slti_instruction.rs1 = rd;
            slti_instruction.imm = i16_to_u16(imm2);
            let rs1_val = cpu.read_x_i32(slti_instruction.rs1).unwrap();

            let slti_op = SLTI::new(slti_instruction);
            prop_assert!(cpu.execute_operation(&slti_op).is_ok());
            prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), if rs1_val < imm2 as i32 {1} else {0});
        }

        #[test] // TODO: Verify this test
        fn test_andi(rd in 1u8..30, rs1 in 1u8..30, imm1 in 0u16..0xFFFF, imm2 in 0u16..0xFFFF){
            let mut cpu = Cpu::new();

            let mut andi_instruction = IInstruction {
                rd: rd,
                func3: 7,
                rs1: rs1,
                imm: imm1,
            };
            let rs1_val = cpu.read_x_u32(andi_instruction.rs1).unwrap();
            let andi_op = ANDI::new(andi_instruction.clone());
            prop_assert!(cpu.execute_operation(&andi_op).is_ok());
            prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), rs1_val & i32_to_u32(sign_extend_12bit_to_32bit(imm1)));

            andi_instruction.rs1 = rd;
            andi_instruction.imm = imm2;
            let rs1_val = cpu.read_x_u32(andi_instruction.rs1).unwrap();

            let andi_op = ANDI::new(andi_instruction);
            prop_assert!(cpu.execute_operation(&andi_op).is_ok());
            prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), rs1_val & i32_to_u32(sign_extend_12bit_to_32bit(imm2)));
        }

        #[test]
        fn test_ori(rd in 1u8..30, rs1 in 1u8..30, imm1 in 0u16..0xFFFF, imm2 in 0u16..0xFFFF){
            let mut cpu = Cpu::new();

            let mut ori_instruction = IInstruction {
                rd: rd,
                func3: 6,
                rs1: rs1,
                imm: imm1,
            };
            let rs1_val = cpu.read_x_u32(ori_instruction.rs1).unwrap();
            let ori_op = ORI::new(ori_instruction.clone());
            prop_assert!(cpu.execute_operation(&ori_op).is_ok());
            prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), rs1_val | i32_to_u32(sign_extend_12bit_to_32bit(imm1)));

            ori_instruction.rs1 = rd;
            ori_instruction.imm = imm2;
            let rs1_val = cpu.read_x_u32(ori_instruction.rs1).unwrap();

            let ori_op = ORI::new(ori_instruction);
            prop_assert!(cpu.execute_operation(&ori_op).is_ok());
            prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), rs1_val | i32_to_u32(sign_extend_12bit_to_32bit(imm2)));
        }

        #[test]
        fn test_xori(rd in 1u8..30, rs1 in 1u8..30, imm1 in 0u16..0xFFFF, imm2 in 0u16..0xFFFF, rs1_val in i32::MIN..i32::MAX) {
            let mut cpu = Cpu::new();

            let mut xori_instruction = IInstruction {
                rd,
                func3: 4,
                rs1,
                imm: imm1,
            };

            cpu.write_x_i32(rs1, rs1_val).unwrap();

            let xori_op = XORI::new(xori_instruction.clone());
            prop_assert!(cpu.execute_operation(&xori_op).is_ok());
            prop_assert_eq!(
                cpu.read_x_i32(rd).unwrap(),
                rs1_val ^ sign_extend_12bit_to_32bit(imm1)
            );

            xori_instruction.rs1 = rd;
            xori_instruction.imm = imm2;
            let rs1_val = cpu.read_x_i32(xori_instruction.rs1).unwrap();

            let xori_op = XORI::new(xori_instruction);
            prop_assert!(cpu.execute_operation(&xori_op).is_ok());
            prop_assert_eq!(
                cpu.read_x_i32(rd).unwrap(),
                rs1_val ^ sign_extend_12bit_to_32bit(imm2)
            );
        }


        #[test]
        fn test_slli(rd in 1u8..30, rs1 in 1u8..30, shamt in 0u8..31, rs1_val in i32::MIN..i32::MAX) {
            let mut cpu = Cpu::new();
            
            let slli_instruction = IInstruction {
                rd,
                func3: 1,
                rs1,
                imm: shamt as u16,
            };
            
            cpu.write_x_i32(rs1, rs1_val).unwrap();
            
            let slli_op = SLLI::new(slli_instruction);
            prop_assert!(cpu.execute_operation(&slli_op).is_ok());
            
            let expected = (rs1_val as u32).wrapping_shl(shamt as u32) as i32;
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), expected);
        }

        #[test]
        fn test_srli(rd in 1u8..30, rs1 in 1u8..30, shamt in 0u8..31, rs1_val in i32::MIN..i32::MAX) {
            let mut cpu = Cpu::new();
            
            let srli_instruction = IInstruction {
                rd,
                func3: 5,
                rs1,
                imm: shamt as u16,
            };
            
            cpu.write_x_u32(rs1, rs1_val as u32).unwrap();
            
            let srli_op = SRLI::new(srli_instruction);
            prop_assert!(cpu.execute_operation(&srli_op).is_ok());
            
            let expected = ((rs1_val as u32) >> shamt) as i32;
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), expected);
        }

        #[test]
        fn test_srai(rd in 1u8..30, rs1 in 1u8..30, shamt in 0u8..31, rs1_val in i32::MIN..i32::MAX) {
            let mut cpu = Cpu::new();
            
            let srai_instruction = IInstruction {
                rd,
                func3: 5,
                rs1,
                imm: (0x400 | (shamt as u16)), // Set the 10th bit to differentiate from SRLI
            };
            
            cpu.write_x_i32(rs1, rs1_val).unwrap();
            
            let srai_op = SRAI::new(srai_instruction);
            prop_assert!(cpu.execute_operation(&srai_op).is_ok());
            
            let expected = rs1_val >> shamt;
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), expected);
        }
        #[test]
        fn test_lui(rd in 1u8..30, imm in 0u32..0xFFFFF) {
            let mut cpu = Cpu::new();
            
            let lui_instruction = UInstruction {
                rd,
                imm: imm,
            };
            
            let lui_op = LUI::new(lui_instruction);
            prop_assert!(cpu.execute_operation(&lui_op).is_ok());
            
            let expected = (imm << 12) as u32;
            prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), expected);
        }

        #[test]
        fn test_auipc(rd in 1u8..30, imm in 0u32..0xFFFFF, pc in 0u32..0xFFFFFFFF) {
            let mut cpu = Cpu::new();
            cpu.reg_pc = pc;
            
            let auipc_instruction = UInstruction {
                rd,
                imm: imm,
            };
            
            let auipc_op = AUIPC::new(auipc_instruction);
            prop_assert!(cpu.execute_operation(&auipc_op).is_ok());
            
            let expected = (imm << 12).wrapping_add(pc);
            prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), expected);
            prop_assert_eq!(cpu.read_pc_u32(), expected);
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
    }
}
