#[cfg(test)]
mod tests {
    use crate::*;

    use crate::cpu::memory::memory_core::Memory;
    use anyhow::Result;

    use cpu::memory::vec_memory::VecMemory;
    use proptest::prelude::*;
    use std::result::Result::Ok;
    use types::*;
    use utils::binary_utils::*;

    fn setup_cpu() -> Cpu {
        Cpu::new()
    }

    const MAX_CYCLES: u32 = 1000000;

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
            let mut cpu = setup_cpu();
            cpu.load_program(program.memory, program.entry_point);
            let mut count = 0;

            loop {
                count += 1;
                if count > MAX_CYCLES {
                    break Err(anyhow::anyhow!("Too many cycles"));
                }
                match cpu.run_cycle() {
                    Ok(_) => continue,
                    Err(e) => {
                        break Ok(e);
                    }
                }
            }
            .unwrap();

            let expected_data = std::fs::read_to_string(file_path.with_extension("res")).unwrap();
            let cpu_stdout = cpu.read_and_clear_stdout_buffer();
            assert_eq!(expected_data, cpu_stdout);
        }
    }

    // Calculates n-th fibbonacci number and stores it in x5
    const FIB_PROGRAM_BIN: &[u32] = &[
        0x00100093, 0x00100113, 0x00002183, // lw x3, x0 - load n from memory
        0x00000213, 0x00010293, 0x00208133, 0x00028093, 0x00120213,
        0xfe3248e3, // blt x4, x3, -16
        0xfcdff06f,
    ];

    fn execute_s_instruction(
        cpu: &mut Cpu,
        opcode: &str,
        rs1: u8,
        rs2: u8,
        imm: u16,
    ) -> Result<()> {
        let instruction = SInstructionData {
            rs1: U5(rs1),
            rs2: U5(rs2),
            imm: SImmediate(U12::new(imm)),
            ..Default::default()
        };
        let op = encode_program_line(opcode, InstructionData::S(instruction))?;
        cpu.execute_word(op)?;
        Ok(())
    }

    fn fib(n1: u32) -> u32 {
        if n1 == 1 {
            1
        } else if n1 == 2 {
            2
        } else {
            fib(n1 - 1) + fib(n1 - 2)
        }
    }

    proptest! {
        #[test]
        fn test_memory_mapping_u32(addr in 0x0u32..(u32::MAX)) {
            let mut cpu = setup_cpu();
            let value = 0x12345678u32;
            cpu.write_mem_u32(addr, value).unwrap();
            prop_assert_eq!(cpu.read_mem_u32(addr).unwrap(), value);
            prop_assert_eq!(cpu.read_mem_u32(addr + 1).unwrap(), 0x00123456);
            prop_assert_eq!(cpu.read_mem_u32(addr + 2).unwrap(), 0x00001234);
            prop_assert_eq!(cpu.read_mem_u32(addr + 3).unwrap(), 0x00000012);
        }

        #[test]
        fn test_memory_mapping_u8(addr in 0x0u32..(u32::MAX)) {
            let mut cpu = setup_cpu();
            let value = 0x12u8;
            cpu.write_mem_u8(addr, value).unwrap();
            prop_assert_eq!(cpu.read_mem_u8(addr).unwrap(), value);
        }

        #[test]
        fn test_memory_mapping_u16_edge_case(addr in 0x0u32..(u32::MAX - 3)) {
            let mut cpu = setup_cpu();
            let value = u32::MAX;

            for offset in 0..4 {
                cpu.write_mem_u32(addr, value).unwrap();
                cpu.write_mem_u16(addr + offset, 0x0000).unwrap();

                let expected = match offset {
                    0 => 0xffff0000,
                    1 => 0xff0000ff,
                    2 => 0x0000ffff,
                    3 => 0x00ffffff,
                    _ => unreachable!(),
                };
                let result = cpu.read_mem_u32(addr).unwrap();
                prop_assert_eq!(result, expected);
            }
        }

        #[test]
        fn test_memory_mapping_u16(addr in 0x0u32..(u32::MAX)) {
            let mut cpu = setup_cpu();
            let value = 0x1234u16;
            cpu.write_mem_u16(addr, value).unwrap();
            prop_assert_eq!(cpu.read_mem_u16(addr).unwrap(), value);
        }

        #[test]
        fn test_memory_mapping_u16_misaligned(addr in 0x0u32..(u32::MAX)) {
            let mut cpu = setup_cpu();
            let value = 0x1234u16;
            cpu.write_mem_u16(addr + 1, value).unwrap();
            prop_assert_eq!(cpu.read_mem_u16(addr + 1).unwrap(), value);
        }

        #[test]
        fn test_memory_mapping_mixed(addr in 0x0u32..(u32::MAX)) {
            let mut cpu = setup_cpu();
            let value = 0x12345678u32;
            cpu.write_mem_u32(addr, value).unwrap();
            prop_assert_eq!(cpu.read_mem_u8(addr).unwrap(), 0x78);
            prop_assert_eq!(cpu.read_mem_u8(addr + 1).unwrap(), 0x56);
            prop_assert_eq!(cpu.read_mem_u8(addr + 2).unwrap(), 0x34);
            prop_assert_eq!(cpu.read_mem_u8(addr + 3).unwrap(), 0x12);
            prop_assert_eq!(cpu.read_mem_u16(addr).unwrap(), 0x5678);
            prop_assert_eq!(cpu.read_mem_u16(addr + 1).unwrap(), 0x3456);
            prop_assert_eq!(cpu.read_mem_u16(addr + 2).unwrap(), 0x1234);
            prop_assert_eq!(cpu.read_mem_u32(addr).unwrap(), value);

            cpu.write_mem_u16(addr, 0xabcd).unwrap();
            prop_assert_eq!(cpu.read_mem_u8(addr).unwrap(), 0xcd);
            prop_assert_eq!(cpu.read_mem_u8(addr + 1).unwrap(), 0xab);

            cpu.write_mem_u16(addr + 2, 0xdcba).unwrap();
            prop_assert_eq!(cpu.read_mem_u8(addr + 2).unwrap(), 0xba);
            prop_assert_eq!(cpu.read_mem_u8(addr + 3).unwrap(), 0xdc);

            prop_assert_eq!(cpu.read_mem_u32(addr).unwrap(), 0xdcbaabcd);

            cpu.write_mem_u8(addr, 0xef).unwrap();
            prop_assert_eq!(cpu.read_mem_u8(addr).unwrap(), 0xef);

            cpu.write_mem_u8(addr + 1, 0xab).unwrap();

            prop_assert_eq!(cpu.read_mem_u32(addr).unwrap(), 0xdcbaabef);
        }

        #[test]
        fn test_memory_cross_boundary_u32(addr_v in 0x0u32..(u32::MAX / (4096 * 16)), offset in 0u32..4u32) {
            let mut cpu = setup_cpu();
            let value = 0x12345678u32;
            let addr = addr_v * 4096 * 16 + offset;
            cpu.write_mem_u32(addr, value).unwrap();
            prop_assert_eq!(cpu.read_mem_u32(addr).unwrap(), value);
            prop_assert_eq!(cpu.read_mem_u32(addr + 1).unwrap(), 0x00123456);
            prop_assert_eq!(cpu.read_mem_u32(addr + 2).unwrap(), 0x00001234);
            prop_assert_eq!(cpu.read_mem_u32(addr + 3).unwrap(), 0x00000012);
        }

        #[test]
        fn test_fibbonaci_program(n in 1u32..15, entry_point in 0x1000u32..0xFFFFF) {
            let mut cpu = Cpu::new();

            let mut memory = VecMemory::new();
            for (id, val) in FIB_PROGRAM_BIN.iter().enumerate() {
                memory.write_mem_u32(entry_point + 4u32 * (id as u32), *val).unwrap();
            }
            cpu.load_program(memory, entry_point);

            cpu.write_mem_u32(0, n).unwrap();

            while cpu.run_cycle().is_ok() {
                // println!("{}", cpu);
            }

            prop_assert_eq!(cpu.read_x_u32(5).unwrap(), fib(n));
        }

        #[test]
        fn test_lb(rd in 1u8..31, rs1 in 1u8..31, imm in 0u16..0xF, value in i8::MIN..i8::MAX) {
            if rs1 == rd {
                return Ok(());
            }
            let mut cpu = setup_cpu();
            let addr = 1000u32;
            cpu.write_x_u32(rs1, addr).unwrap();
            cpu.write_mem_u8(addr.wrapping_add_signed(imm as i16 as i32), value as u8).unwrap();

            execute_i_instruction(&mut cpu, "LB", rd, rs1, imm).unwrap();

            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), value as i32);
        }

        #[test]
        fn test_lh(rd in 1u8..31, rs1 in 1u8..31, imm in 0u16..0xF, value in i16::MIN..i16::MAX) {
            if rs1 == rd {
                return Ok(());
            }
            let mut cpu = setup_cpu();
            let addr = 1000u32;
            cpu.write_x_u32(rs1, addr).unwrap();
            cpu.write_mem_u16(addr.wrapping_add_signed(imm as i16 as i32), value as u16).unwrap();

            execute_i_instruction(&mut cpu, "LH", rd, rs1, imm).unwrap();

            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), value as i32);
        }

        #[test]
        fn test_lw(rd in 1u8..31, rs1 in 1u8..31, _imm in 0u16..0xF, value in i32::MIN..i32::MAX) {
            if rs1 == rd {
                return Ok(());
            }
            let mut cpu = setup_cpu();
            let addr = 1000u32;
            cpu.write_x_u32(rs1, addr).unwrap();
            cpu.write_mem_u32(addr.wrapping_add_signed(0), value as u32).unwrap();

            execute_i_instruction(&mut cpu, "LW", rd, rs1, 0).unwrap();

            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), value);
        }

        #[test]
        fn test_lbu(rd in 1u8..31, rs1 in 1u8..31, imm in 0u16..0xF, value in u8::MIN..u8::MAX) {
            if rs1 == rd {
                return Ok(());
            }
            let mut cpu = setup_cpu();
            let addr = 1000u32;
            cpu.write_x_u32(rs1, addr).unwrap();
            cpu.write_mem_u8(addr.wrapping_add_signed(imm as i16 as i32), value).unwrap();

            execute_i_instruction(&mut cpu, "LBU", rd, rs1, imm).unwrap();

            prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), value as u32);
        }

        #[test]
        fn test_lhu(rd in 1u8..31, rs1 in 1u8..31, imm in 0u16..0xF, value in 10u16..u16::MAX) {
            if rs1 == rd {
                return Ok(());
            }
            let mut cpu = setup_cpu();
            let addr = 1000u32;

            cpu.write_x_u32(rs1, addr).unwrap();
            cpu.write_mem_u16(addr.wrapping_add_signed(imm as i16 as i32), value).unwrap();

            execute_i_instruction(&mut cpu, "LHU", rd, rs1, imm).unwrap();

            prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), value as u32);
        }

        #[test]
        fn test_sb(rs1 in 1u8..31, rs2 in 1u8..31, imm in 0u16..0xF, value in i8::MIN..i8::MAX) {
            if rs1 == rs2 {
                return Ok(());
            }
            let mut cpu = setup_cpu();
            let addr = 1000u32;
            cpu.write_x_u32(rs1, addr).unwrap();
            cpu.write_x_i32(rs2, value as i32).unwrap();

            execute_s_instruction(&mut cpu, "SB", rs1, rs2, imm).unwrap();

            prop_assert_eq!(cpu.read_mem_u8(addr.wrapping_add_signed(imm as i16 as i32)).unwrap(), value as u8);
        }

        #[test]
        fn test_sh(rs1 in 1u8..31, rs2 in 1u8..31, imm in 0u16..0xF, value in i16::MIN..i16::MAX) {
            if rs1 == rs2 {
                return Ok(());
            }
            let mut cpu = setup_cpu();
            let addr = 1000u32;
            cpu.write_x_u32(rs1, addr).unwrap();
            cpu.write_x_i32(rs2, value as i32).unwrap();

            execute_s_instruction(&mut cpu, "SH", rs1, rs2, imm).unwrap();

            prop_assert_eq!(cpu.read_mem_u16(addr.wrapping_add_signed(imm as i16 as i32)).unwrap(), value as u16);
        }

        #[test]
        fn test_sw(rs1 in 1u8..31, rs2 in 1u8..31, imm in 0u16..0xF, value in i32::MIN..i32::MAX) {
            if rs1 == rs2 {
                return Ok(());
            }
            let mut cpu = setup_cpu();
            let addr = 1000u32;
            cpu.write_x_u32(rs1, addr).unwrap();
            cpu.write_x_i32(rs2, value).unwrap();

            execute_s_instruction(&mut cpu, "SW", rs1, rs2, imm).unwrap();

            prop_assert_eq!(cpu.read_mem_u32(addr.wrapping_add_signed(imm as i16 as i32)).unwrap(), value as u32);
        }
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
        let instruction = UInstructionData { rd: U5(rd), imm };
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

    test_instruction_i!(test_addi2, "ADDI", |cpu: &mut Cpu, rd, _rs1, imm| {
        prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), imm as i32);
        Ok(())
    });

    test_instruction_u!(test_lui2, "LUI", |cpu: &mut Cpu, rd, imm| {
        let expected = imm << 12;
        prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), expected);
        Ok(())
    });

    test_instruction_r!(
        test_add2,
        "ADD",
        |cpu: &mut Cpu, rd, _rs1, _rs2, rs1_read_val: i32, rs2_read_val: i32| {
            let (expected, _) = rs1_read_val.overflowing_add(rs2_read_val);
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), expected);
            Ok(())
        }
    );

    test_instruction_r!(
        test_sub,
        "SUB",
        |cpu: &mut Cpu, rd, _rs1, _rs2, rs1_read_val: i32, rs2_read_val: i32| {
            let (expected, _) = rs1_read_val.overflowing_sub(rs2_read_val);
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), expected);
            Ok(())
        }
    );

    test_instruction_r!(
        test_slt,
        "SLT",
        |cpu: &mut Cpu, rd, _rs1, _rs2, rs1_read_val: i32, rs2_read_val: i32| {
            let expected = if rs1_read_val < rs2_read_val { 1 } else { 0 };
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), expected);
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
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), expected);
            Ok(())
        }
    );

    test_instruction_r!(
        test_xor,
        "XOR",
        |cpu: &mut Cpu, rd, _rs1, _rs2, rs1_read_val: i32, rs2_read_val: i32| {
            let expected = rs1_read_val ^ rs2_read_val;
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), expected);
            Ok(())
        }
    );

    test_instruction_r!(
        test_or,
        "OR",
        |cpu: &mut Cpu, rd, _rs1, _rs2, rs1_read_val: i32, rs2_read_val: i32| {
            let expected = rs1_read_val | rs2_read_val;
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), expected);
            Ok(())
        }
    );

    test_instruction_r!(
        test_and,
        "AND",
        |cpu: &mut Cpu, rd, _rs1, _rs2, rs1_read_val: i32, rs2_read_val: i32| {
            let expected = rs1_read_val & rs2_read_val;
            prop_assert_eq!(cpu.read_x_i32(rd).unwrap(), expected);
            Ok(())
        }
    );

    test_instruction_r!(
        test_sll,
        "SLL",
        |cpu: &mut Cpu, rd, _rs1, _rs2, rs1_read_val: i32, rs2_read_val: i32| {
            let expected = (i32_to_u32(rs1_read_val)) << (i32_to_u32(rs2_read_val) & 0b11111);
            prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), expected);
            Ok(())
        }
    );

    test_instruction_r!(
        test_srl,
        "SRL",
        |cpu: &mut Cpu, rd, _rs1, _rs2, rs1_read_val: i32, rs2_read_val: i32| {
            let expected = (i32_to_u32(rs1_read_val)) >> (i32_to_u32(rs2_read_val) & 0b11111);
            prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), expected);
            Ok(())
        }
    );

    test_instruction_r!(
        test_sra,
        "SRA",
        |cpu: &mut Cpu, rd, _rs1, _rs2, rs1_read_val: i32, rs2_read_val: i32| {
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
                imm,
                ..Default::default()
            };
            let op = encode_program_line("ADDI", InstructionData::I(addi_instruction)).unwrap();
            let decoded = decode_program_line(op).unwrap();
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
                imm,
            };

            let op = encode_program_line("LUI", InstructionData::U(lui_instruction)).unwrap();

            prop_assert!(cpu.execute_word(op).is_ok());

            let expected = imm << 12;
            prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), expected);
        }

        #[test]
        fn test_auipc(rd in 1u8..30, imm in 0u32..0xFFFFF, pc in 0u32..0xFFFFFFFF) {
            let mut cpu = Cpu::new();
            cpu.write_pc_u32(pc);
            cpu.current_instruction_pc = pc;

            let auipc_instruction = UInstructionData {
                rd: U5(rd),
                imm,
            };

            let op = encode_program_line("AUIPC", InstructionData::U(auipc_instruction)).unwrap();

            prop_assert!(cpu.execute_word(op).is_ok());

            let expected = (imm << 12).wrapping_add(pc);
            prop_assert_eq!(cpu.read_x_u32(rd).unwrap(), expected);
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
    }
}
