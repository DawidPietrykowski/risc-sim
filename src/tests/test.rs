use crate::*;

use cpu::cpu_core::{Cpu, CpuMode};
use elf::elf_loader::{decode_file, WordSize};

use proptest::prelude::*;
use std::result::Result::Ok;
use tests::util::*;
use types::*;
use utils::binary_utils::*;

#[test]
fn test_example_c_programs() {
    let test_programs = std::fs::read_dir("tests").unwrap().filter_map(|e| {
        let e = e.unwrap();
        let path = e.path();
        if path.extension().is_none() {
            Some(path)
        } else {
            None
        }
    });
    for file_path in test_programs {
        let program = decode_file(file_path.as_os_str().to_str().unwrap());
        let mut cpu;
        if program.header.word_size == WordSize::W32 {
            cpu = setup_cpu()
        } else {
            cpu = setup_cpu_64()
        }

        cpu.load_program_from_elf(program).unwrap();
        let mut count = 0;

        loop {
            count += 1;
            if count > MAX_CYCLES {
                break Err(
                    anyhow::anyhow!("Too many cycles").context(format!("File: {:?}", file_path))
                );
            }
            match cpu.run_cycles(1) {
                Ok(_) => continue,
                Err(e) => {
                    break Ok(e);
                }
            }
        }
        .unwrap();

        let expected_data = std::fs::read_to_string(file_path.with_extension("res")).unwrap();
        let cpu_stdout = cpu.kernel.read_and_clear_stdout_buffer();
        assert_eq!(expected_data, cpu_stdout);
    }
}

// Calculates n-th fibbonacci number and stores it in x5
const FIB_PROGRAM_BIN: &[u32] = &[
    0x00100093, 0x00100113, 0x00002183, // lw x3, x0 - load n from memory
    0x00000213, 0x00010293, 0x00208133, 0x00028093, 0x00120213, 0xfe3248e3, // blt x4, x3, -16
    0xfcdff06f,
];

fn fib(n1: u32) -> u32 {
    if n1 == 1 {
        1
    } else if n1 == 2 {
        2
    } else {
        fib(n1 - 1) + fib(n1 - 2)
    }
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

                    cpu.write_x_i32(rs1, rs1_val);
                    cpu.write_x_i32(rs2, rs2_val);

                    let rs1_read_val = cpu.read_x_i32(rs1);
                    let rs2_read_val = cpu.read_x_i32(rs2);

                    let op = encode_program_line($opcode, InstructionData::R(instruction)).unwrap();
                    prop_assert!(cpu.execute_word(op).is_ok());

                    $test_logic(&mut cpu, rd, rs1, rs2, rs1_read_val, rs2_read_val).unwrap();
                }
            }
        };
    }

test_instruction_i!(test_addi2, "ADDI", |cpu: &mut Cpu, rd, _rs1, imm| {
    prop_assert_eq!(cpu.read_x_i32(rd), imm as i32);
    Ok(())
});

test_instruction_u!(test_lui2, "LUI", |cpu: &mut Cpu, rd, imm| {
    let expected = imm << 12;
    prop_assert_eq!(cpu.read_x_u32(rd), expected);
    Ok(())
});

test_instruction_r!(
    test_add2,
    "ADD",
    |cpu: &mut Cpu, rd, _rs1, _rs2, rs1_read_val: i32, rs2_read_val: i32| {
        let (expected, _) = rs1_read_val.overflowing_add(rs2_read_val);
        prop_assert_eq!(cpu.read_x_i32(rd), expected);
        Ok(())
    }
);

test_instruction_r!(
    test_sub,
    "SUB",
    |cpu: &mut Cpu, rd, _rs1, _rs2, rs1_read_val: i32, rs2_read_val: i32| {
        let (expected, _) = rs1_read_val.overflowing_sub(rs2_read_val);
        prop_assert_eq!(cpu.read_x_i32(rd), expected);
        Ok(())
    }
);

test_instruction_r!(
    test_slt,
    "SLT",
    |cpu: &mut Cpu, rd, _rs1, _rs2, rs1_read_val: i32, rs2_read_val: i32| {
        let expected = if rs1_read_val < rs2_read_val { 1 } else { 0 };
        prop_assert_eq!(cpu.read_x_i32(rd), expected);
        Ok(())
    }
);

test_instruction_r!(
    test_sltu,
    "SLTU",
    |cpu: &mut Cpu, rd, _rs1, _rs2, rs1_read_val: i32, rs2_read_val: i32| {
        let expected = if i32_to_u32(rs1_read_val) < i32_to_u32(rs2_read_val) {
            1
        } else {
            0
        };
        prop_assert_eq!(cpu.read_x_i32(rd), expected);
        Ok(())
    }
);

test_instruction_r!(
    test_xor,
    "XOR",
    |cpu: &mut Cpu, rd, _rs1, _rs2, rs1_read_val: i32, rs2_read_val: i32| {
        let expected = rs1_read_val ^ rs2_read_val;
        prop_assert_eq!(cpu.read_x_i32(rd), expected);
        Ok(())
    }
);

test_instruction_r!(
    test_or,
    "OR",
    |cpu: &mut Cpu, rd, _rs1, _rs2, rs1_read_val: i32, rs2_read_val: i32| {
        let expected = rs1_read_val | rs2_read_val;
        prop_assert_eq!(cpu.read_x_i32(rd), expected);
        Ok(())
    }
);

test_instruction_r!(
    test_and,
    "AND",
    |cpu: &mut Cpu, rd, _rs1, _rs2, rs1_read_val: i32, rs2_read_val: i32| {
        let expected = rs1_read_val & rs2_read_val;
        prop_assert_eq!(cpu.read_x_i32(rd), expected);
        Ok(())
    }
);

test_instruction_r!(
    test_sll,
    "SLL",
    |cpu: &mut Cpu, rd, _rs1, _rs2, rs1_read_val: i32, rs2_read_val: i32| {
        let expected = (i32_to_u32(rs1_read_val)) << (i32_to_u32(rs2_read_val) & 0b11111);
        prop_assert_eq!(cpu.read_x_u32(rd), expected);
        Ok(())
    }
);

test_instruction_r!(
    test_srl,
    "SRL",
    |cpu: &mut Cpu, rd, _rs1, _rs2, rs1_read_val: i32, rs2_read_val: i32| {
        let expected = (i32_to_u32(rs1_read_val)) >> (i32_to_u32(rs2_read_val) & 0b11111);
        prop_assert_eq!(cpu.read_x_u32(rd), expected);
        Ok(())
    }
);

test_instruction_r!(
    test_sra,
    "SRA",
    |cpu: &mut Cpu, rd, _rs1, _rs2, rs1_read_val: i32, rs2_read_val: i32| {
        let expected = (rs1_read_val) >> (i32_to_u32(rs2_read_val) & 0b11111);
        prop_assert_eq!(cpu.read_x_i32(rd), expected);
        Ok(())
    }
);

proptest! {
    #[test]
    fn test_fibbonaci_program(n in 1u32..15, entry_point in 0x1000u64..0xFFFFF) {
        let mut cpu = Cpu::default();

        cpu.load_program_from_opcodes(FIB_PROGRAM_BIN.to_vec(), entry_point, cpu.arch_mode).unwrap();

        cpu.write_mem_u32(0, n).unwrap();

        while cpu.run_cycles(1).is_ok() {
        }

        prop_assert_eq!(cpu.read_x_u32(5), fib(n));
    }

    #[test]
    fn test_encode_decode_i16(rd in 1u8..30, rs1 in 1u8..30, immi16 in -2048i16..2047){
        let imm = U12(i16_to_u16(immi16) & 0xFFF);
        let addi_instruction = IInstructionData {
            rd: U5(rd),
            rs1: U5(rs1),
            imm,
            ..Default::default()
        };
        let op = encode_program_line("ADDI", InstructionData::I(addi_instruction)).unwrap();
        let decoded = decode_program_line(op, CpuMode::RV32).unwrap();
        prop_assert_eq!(parse_instruction_i(&decoded.word), addi_instruction);
    }

    #[test]
    fn test_encode_decode(rd in 1u8..30, rs1 in 1u8..30, imm_u16 in 0u16..4095){
        let imm = U12(imm_u16);
        let addi_instruction = IInstructionData {
            rd: U5(rd),
            rs1: U5(rs1),
            imm,
            ..Default::default()
        };
        let op = encode_program_line("ADDI", InstructionData::I(addi_instruction)).unwrap();
        let decoded = decode_program_line(op, CpuMode::RV32).unwrap();
        prop_assert_eq!(parse_instruction_i(&decoded.word), addi_instruction);
    }

    #[test]
    fn test_addi(rd in 1u8..30, rs1 in 1u8..30, imm1 in -2048i16..2047, imm2 in -2048i16..2047) {
        let mut cpu = Cpu::default();
        let mut addi_instruction = IInstructionData {
            rd: U5(rd),
            rs1: U5(rs1),
            imm: U12(i16_to_u16(imm1)),
            ..Default::default()
        };
        let op = encode_program_line("ADDI", InstructionData::I(addi_instruction)).unwrap();
        prop_assert!(cpu.execute_word(op).is_ok());
        prop_assert_eq!(cpu.read_x_i32(rd), imm1 as i32);

        addi_instruction.rs1 = U5(rd);
        addi_instruction.imm = U12(i16_to_u16(imm2));
        let rs1_val = cpu.read_x_i32(addi_instruction.rs1.value());

        let op = encode_program_line("ADDI", InstructionData::I(addi_instruction)).unwrap();
        prop_assert!(cpu.execute_word(op).is_ok());
        prop_assert_eq!(cpu.read_x_i32(rd), imm2 as i32 + rs1_val);
    }

    #[test]
    fn test_stli(rd in 1u8..30, rs1 in 1u8..30, imm1 in -2048i16..2047, imm2 in -2048i16..2047){
        let mut cpu = Cpu::default();
        let mut slti_instruction = IInstructionData {
            rd: U5(rd),
            rs1: U5(rs1),
            imm: U12(i16_to_u16(imm1)),
            ..Default::default()
        };
        let rs1_val = cpu.read_x_i32(slti_instruction.rs1.value());
        let slti_op = encode_program_line("SLTI", InstructionData::I(slti_instruction)).unwrap();

        prop_assert!(cpu.execute_word(slti_op).is_ok());
        prop_assert_eq!(cpu.read_x_u32(rd), if rs1_val < imm1 as i32 {1} else {0});


        slti_instruction.rs1 = U5(rd);
        slti_instruction.imm = U12(i16_to_u16(imm2));
        let rs1_val = cpu.read_x_i32(slti_instruction.rs1.value());

        let slti_op = encode_program_line("SLTI", InstructionData::I(slti_instruction)).unwrap();
        prop_assert!(cpu.execute_word(slti_op).is_ok());
        prop_assert_eq!(cpu.read_x_u32(rd), if rs1_val < imm2 as i32 {1} else {0});
    }

    #[test]
    fn test_andi(rd in 1u8..30, rs1 in 1u8..30, imm1 in 0u16..0xFFFF, imm2 in 0u16..0xFFFF){
        let mut cpu = Cpu::default();
        let mut andi_instruction = IInstructionData {
            rd: U5(rd),
            rs1: U5(rs1),
            imm: U12(imm1),
            ..Default::default()
        };
        let rs1_val = cpu.read_x_u32(andi_instruction.rs1.value());
        let op = encode_program_line("ANDI", InstructionData::I(andi_instruction)).unwrap();
        prop_assert!(cpu.execute_word(op).is_ok());
        prop_assert_eq!(cpu.read_x_u32(rd), rs1_val & i32_to_u32(sign_extend_12bit_to_32bit(imm1)));

        andi_instruction.rs1 = U5(rd);
        andi_instruction.imm = U12(imm2);
        let rs1_val = cpu.read_x_u32(andi_instruction.rs1.value());

        let op = encode_program_line("ANDI", InstructionData::I(andi_instruction)).unwrap();
        prop_assert!(cpu.execute_word(op).is_ok());
        prop_assert_eq!(cpu.read_x_u32(rd), rs1_val & i32_to_u32(sign_extend_12bit_to_32bit(imm2)));
    }

    #[test]
    fn test_ori(rd in 1u8..30, rs1 in 1u8..30, imm1 in 0u16..0x0FFF, imm2 in 0u16..0x0FFF){
        let mut cpu = Cpu::default();
        let mut ori_instruction = IInstructionData {
            rd: U5(rd),
            rs1: U5(rs1),
            imm: U12(imm1),
            ..Default::default()
        };
        let rs1_val = cpu.read_x_u32(ori_instruction.rs1.value());
        let op = encode_program_line("ORI", InstructionData::I(ori_instruction)).unwrap();
        prop_assert!(cpu.execute_word(op).is_ok());
        prop_assert_eq!(cpu.read_x_u32(rd), rs1_val | i32_to_u32(sign_extend_12bit_to_32bit(imm1)));

        ori_instruction.rs1 = U5(rd);
        ori_instruction.imm = U12(imm2);
        let rs1_val = cpu.read_x_u32(ori_instruction.rs1.value());

        let op = encode_program_line("ORI", InstructionData::I(ori_instruction)).unwrap();
        prop_assert!(cpu.execute_word(op).is_ok());
        prop_assert_eq!(cpu.read_x_u32(rd), rs1_val | i32_to_u32(sign_extend_12bit_to_32bit(imm2)));
    }

    #[test]
    fn test_xori(rd in 1u8..30, rs1 in 1u8..30, imm1 in 0u16..0x0FFF, imm2 in 0u16..0x0FFF, rs1_val in i32::MIN..i32::MAX) {
        let mut cpu = Cpu::default();
        let mut xori_instruction = IInstructionData{
            rd: U5(rd),
            rs1: U5(rs1),
            imm: U12(imm1),
            ..Default::default()
        };

        cpu.write_x_i32(rs1, rs1_val);

        let op = encode_program_line("XORI", InstructionData::I(xori_instruction)).unwrap();
        prop_assert!(cpu.execute_word(op).is_ok());
        prop_assert_eq!(
            cpu.read_x_i32(rd),
            rs1_val ^ sign_extend_12bit_to_32bit(imm1)
        );

        xori_instruction.rs1 = U5(rd);
        xori_instruction.imm = U12(imm2);
        let rs1_val = cpu.read_x_i32(xori_instruction.rs1.value());

        let op = encode_program_line("XORI", InstructionData::I(xori_instruction)).unwrap();
        prop_assert!(cpu.execute_word(op).is_ok());
        prop_assert_eq!(
            cpu.read_x_i32(rd),
            rs1_val ^ sign_extend_12bit_to_32bit(imm2)
        );
    }


    #[test]
    fn test_slli(rd in 1u8..30, rs1 in 1u8..30, shamt in 0u8..31, rs1_val in i32::MIN..i32::MAX) {
        let mut cpu = Cpu::default();
        let slli_instruction = IInstructionData {
            rd: U5(rd),
            rs1: U5(rs1),
            imm: U12(shamt as u16),
            ..Default::default()
        };

        cpu.write_x_i32(rs1, rs1_val);

        let op = encode_program_line("SLLI", InstructionData::I(slli_instruction)).unwrap();

        prop_assert!(cpu.execute_word(op).is_ok());

        let expected = (rs1_val as u32).wrapping_shl(shamt as u32) as i32;
        prop_assert_eq!(cpu.read_x_i32(rd), expected);
    }

    #[test]
    fn test_srli(rd in 1u8..30, rs1 in 1u8..30, shamt in 0u8..31, rs1_val in i32::MIN..i32::MAX) {
        let mut cpu = Cpu::default();
        let srli_instruction = IInstructionData {
            rd: U5(rd),
            rs1: U5(rs1),
            imm: U12(shamt as u16),
            ..Default::default()
        };

        cpu.write_x_u32(rs1, rs1_val as u32);

        let op = encode_program_line("SRLI", InstructionData::I(srli_instruction)).unwrap();

        prop_assert!(cpu.execute_word(op).is_ok());

        let expected = ((rs1_val as u32) >> shamt) as i32;
        prop_assert_eq!(cpu.read_x_i32(rd), expected);
    }

    #[test]
    fn test_srai(rd in 1u8..30, rs1 in 1u8..30, shamt in 0u8..31, rs1_val in i32::MIN..i32::MAX) {
        let mut cpu = Cpu::default();

        let srai_instruction = IInstructionData {
            rd: U5(rd),
            rs1: U5(rs1),
            imm: U12(0x400 | (shamt as u16)), // Set the 10th bit to differentiate from SRLI
            ..Default::default()
        };

        cpu.write_x_i32(rs1, rs1_val);

        let op = encode_program_line("SRAI", InstructionData::I(srai_instruction)).unwrap();

        prop_assert!(cpu.execute_word(op).is_ok());

        let expected = rs1_val >> shamt;
        prop_assert_eq!(cpu.read_x_i32(rd), expected);
    }
    #[test]
    fn test_lui(rd in 1u8..30, imm in 0u32..0xFFFFF) {
        let mut cpu = Cpu::default();

        let lui_instruction = UInstructionData {
            rd: U5(rd),
            imm,
        };

        let op = encode_program_line("LUI", InstructionData::U(lui_instruction)).unwrap();

        prop_assert!(cpu.execute_word(op).is_ok());

        let expected = imm << 12;
        prop_assert_eq!(cpu.read_x_u32(rd), expected);
    }

    #[test]
    fn test_auipc(rd in 1u8..30, imm in 0u32..0xFFFFF) {
        let mut cpu = Cpu::default();

        let auipc_instruction = UInstructionData {
            rd: U5(rd),
            imm,
        };

        let op = encode_program_line("AUIPC", InstructionData::U(auipc_instruction)).unwrap();

        prop_assert!(cpu.execute_word(op).is_ok());

        let expected = imm << 12;
        prop_assert_eq!(cpu.read_x_u32(rd), expected);
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

        cpu.write_x_i32(rs1, rs1_val);
        cpu.write_x_i32(rs2, rs2_val);

        let rs1_read_val = cpu.read_x_i32(rs1);
        let rs2_read_val = cpu.read_x_i32(rs2);

        let op = encode_program_line("ADD", InstructionData::R(instruction)).unwrap();
        prop_assert!(cpu.execute_word(op).is_ok());

        let (expected, _) = rs1_read_val.overflowing_add(rs2_read_val);
        prop_assert_eq!(cpu.read_x_i32(rd), expected);
    }

    #[test]
    fn test_add_64(rd in 1u8..30, rs1 in 1u8..30, rs2 in 1u8..30, rs1_val in i64::MIN..i64::MAX, rs2_val in i64::MIN..i64::MAX) {
        let mut cpu = setup_cpu_64();
        let instruction = RInstructionData {
            rd: U5(rd),
            rs1: U5(rs1),
            rs2: U5(rs2),
            ..Default::default()
        };

        cpu.write_x_i64(rs1, rs1_val);
        cpu.write_x_i64(rs2, rs2_val);

        let rs1_read_val = cpu.read_x_i64(rs1);
        let rs2_read_val = cpu.read_x_i64(rs2);

        let op = encode_program_line("ADD", InstructionData::R(instruction)).unwrap();

        prop_assert!(cpu.execute_word(op).is_ok());

        let (expected, _) = rs1_read_val.overflowing_add(rs2_read_val);
        prop_assert_eq!(cpu.read_x_i64(rd), expected);
    }


    #[test]
    fn test_csrrw(rd in 1u8..30, rs1 in 1u8..30, csr in 0u16..0xFFF, rs1_val in u32::MIN..u32::MAX, csr_val in u32::MIN..u32::MAX) {
        let mut cpu = Cpu::default();
        let csrrw_instruction = IInstructionData {
            rd: U5(rd),
            rs1: U5(rs1),
            imm: U12(csr),
            ..Default::default()
        };

        cpu.write_x_u32(rs1, rs1_val);
        cpu.csr_table.write32(U12::new(csr), csr_val);

        let op = encode_program_line("CSRRW", InstructionData::I(csrrw_instruction)).unwrap();

        prop_assert!(cpu.execute_word(op).is_ok());

        prop_assert_eq!(cpu.read_x_u32(rd), csr_val);
        prop_assert_eq!(cpu.csr_table.read32(U12::new(csr)), rs1_val);
    }


    #[test]
    fn test_csrrs(rd in 1u8..30, rs1 in 1u8..30, csr in 0u16..0xFFF, rs1_val in u32::MIN..u32::MAX, csr_val in u32::MIN..u32::MAX) {
        let mut cpu = Cpu::default();
        let csrrs_instruction = IInstructionData {
            rd: U5(rd),
            rs1: U5(rs1),
            imm: U12(csr),
            ..Default::default()
        };

        cpu.write_x_u32(rs1, rs1_val);
        cpu.csr_table.write32(U12::new(csr), csr_val);

        let op = encode_program_line("CSRRS", InstructionData::I(csrrs_instruction)).unwrap();

        prop_assert!(cpu.execute_word(op).is_ok());

        prop_assert_eq!(cpu.read_x_u32(rd), csr_val);
        prop_assert_eq!(cpu.csr_table.read32(U12::new(csr)), csr_val | rs1_val);
    }

    #[test]
    fn test_csrrc(rd in 1u8..30, rs1 in 1u8..30, csr in 0u16..0xFFF, rs1_val in u32::MIN..u32::MAX, csr_val in u32::MIN..u32::MAX) {
        let mut cpu = Cpu::default();
        let csrrc_instruction = IInstructionData {
            rd: U5(rd),
            rs1: U5(rs1),
            imm: U12(csr),
            ..Default::default()
        };

        cpu.write_x_u32(rs1, rs1_val);
        cpu.csr_table.write32(U12::new(csr), csr_val);

        let op = encode_program_line("CSRRC", InstructionData::I(csrrc_instruction)).unwrap();

        prop_assert!(cpu.execute_word(op).is_ok());

        prop_assert_eq!(cpu.read_x_u32(rd), csr_val);
        prop_assert_eq!(cpu.csr_table.read32(U12::new(csr)), csr_val & !rs1_val);
    }

    #[test]
    fn test_csrrwi(rd in 1u8..30, zimm in 0i8..31, csr in 0u16..0xFFF, csr_val in u32::MIN..u32::MAX) {
        let mut cpu = Cpu::default();
        let csrrwi_instruction = IInstructionData {
            rd: U5(rd),
            rs1: U5(zimm as u8), // Using rs1 field for zimm
            imm: U12(csr),
            ..Default::default()
        };

        cpu.csr_table.write32(U12::new(csr), csr_val);

        let op = encode_program_line("CSRRWI", InstructionData::I(csrrwi_instruction)).unwrap();

        prop_assert!(cpu.execute_word(op).is_ok());

        prop_assert_eq!(cpu.read_x_u32(rd), csr_val);
        prop_assert_eq!(cpu.csr_table.read32(U12::new(csr)), zimm as u32);
    }

    #[test]
    fn test_csrrsi(rd in 1u8..30, zimm in 0u8..31, csr in 0u16..0xFFF, csr_val in u32::MIN..u32::MAX) {
        let mut cpu = Cpu::default();
        let csrrsi_instruction = IInstructionData {
            rd: U5(rd),
            rs1: U5(zimm), // Using rs1 field for zimm
            imm: U12(csr),
            ..Default::default()
        };

        cpu.csr_table.write32(U12::new(csr), csr_val);

        let op = encode_program_line("CSRRSI", InstructionData::I(csrrsi_instruction)).unwrap();

        prop_assert!(cpu.execute_word(op).is_ok());

        prop_assert_eq!(cpu.read_x_u32(rd), csr_val);
        prop_assert_eq!(cpu.csr_table.read32(U12::new(csr)), csr_val | (zimm as u32));
    }

    #[test]
    fn test_csrrci(rd in 1u8..30, zimm in 0u8..31, csr in 0u16..0xFFF, csr_val in u32::MIN..u32::MAX) {
        let mut cpu = Cpu::default();
        let csrrci_instruction = IInstructionData {
            rd: U5(rd),
            rs1: U5(zimm), // Using rs1 field for zimm
            imm: U12(csr),
            ..Default::default()
        };

        cpu.csr_table.write32(U12::new(csr), csr_val);

        let op = encode_program_line("CSRRCI", InstructionData::I(csrrci_instruction)).unwrap();

        prop_assert!(cpu.execute_word(op).is_ok());

        prop_assert_eq!(cpu.read_x_u32(rd), csr_val);
        prop_assert_eq!(cpu.csr_table.read32(U12::new(csr)), csr_val & !(zimm as u32));
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
