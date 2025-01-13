use proptest::{prop_assert_eq, proptest};

use crate::{
    cpu::memory::{memory_core::Memory, page_storage::PAGE_SIZE, raw_memory::ContinuousMemory},
    tests::util::{execute_i_instruction, execute_s_instruction, setup_cpu, setup_cpu_64},
};

proptest! {
    #[test]
    fn test_memory_mapping_u32(addr in 0x0u64..(u32::MAX as u64 - 3 - 3)) {
        let mut cpu = setup_cpu();
        let value = 0x12345678u32;
        cpu.write_mem_u32(addr, value).unwrap();
        prop_assert_eq!(cpu.read_mem_u32(addr).unwrap(), value);
        prop_assert_eq!(cpu.read_mem_u32(addr + 1).unwrap(), 0x00123456);
        prop_assert_eq!(cpu.read_mem_u32(addr + 2).unwrap(), 0x00001234);
        prop_assert_eq!(cpu.read_mem_u32(addr + 3).unwrap(), 0x00000012);
    }

    #[test]
    fn test_memory_mapping_u64_continuous(addr in 0x1u64..0x1018u64, offset in 0x0..(u64::MAX - 0x2048 - 0x8)) {
        let value = 0x123456789abcdef0u64;
        let mut memory = ContinuousMemory::new(offset, 0x1024);
        let addr = addr + offset;
        memory.write_mem_u64(addr, value).unwrap();
        prop_assert_eq!(memory.read_mem_u64(addr).unwrap(), value);
        prop_assert_eq!(memory.read_mem_u64(addr + 1).unwrap(), 0x00123456789abcde);
        prop_assert_eq!(memory.read_mem_u64(addr + 2).unwrap(), 0x0000123456789abc);
        prop_assert_eq!(memory.read_mem_u64(addr + 3).unwrap(), 0x000000123456789a);
        prop_assert_eq!(memory.read_mem_u64(addr + 4).unwrap(), 0x0000000012345678);
        prop_assert_eq!(memory.read_mem_u64(addr + 5).unwrap(), 0x0000000000123456);
        prop_assert_eq!(memory.read_mem_u64(addr + 6).unwrap(), 0x0000000000001234);
        prop_assert_eq!(memory.read_mem_u64(addr + 7).unwrap(), 0x0000000000000012);
        prop_assert_eq!(memory.read_mem_u64(addr + 8).unwrap(), 0x0000000000000000);
        prop_assert_eq!(memory.read_mem_u64(addr - 1).unwrap(), 0x3456789abcdef000);
    }

    #[test]
    fn test_memory_mapping_u64(addr in 0x1u64..(u64::MAX - 8)) {
        let mut cpu = setup_cpu_64();
        let value = 0x123456789abcdef0u64;
        cpu.write_mem_u64(addr, value).unwrap();
        prop_assert_eq!(cpu.read_mem_u64(addr).unwrap(), value);
        prop_assert_eq!(cpu.read_mem_u64(addr + 1).unwrap(), 0x00123456789abcde);
        prop_assert_eq!(cpu.read_mem_u64(addr + 2).unwrap(), 0x0000123456789abc);
        prop_assert_eq!(cpu.read_mem_u64(addr + 3).unwrap(), 0x000000123456789a);
        prop_assert_eq!(cpu.read_mem_u64(addr + 4).unwrap(), 0x0000000012345678);
        prop_assert_eq!(cpu.read_mem_u64(addr + 5).unwrap(), 0x0000000000123456);
        prop_assert_eq!(cpu.read_mem_u64(addr + 6).unwrap(), 0x0000000000001234);
        prop_assert_eq!(cpu.read_mem_u64(addr + 7).unwrap(), 0x0000000000000012);
        prop_assert_eq!(cpu.read_mem_u64(addr + 8).unwrap(), 0x0000000000000000);
        prop_assert_eq!(cpu.read_mem_u64(addr - 1).unwrap(), 0x3456789abcdef000);
    }

    #[test]
    fn test_memory_mapping_u8(addr in 0x0u64..(u32::MAX as u64)) {
        let mut cpu = setup_cpu();
        let value = 0x12u8;
        cpu.write_mem_u8(addr, value).unwrap();
        prop_assert_eq!(cpu.read_mem_u8(addr).unwrap(), value);
    }

    #[test]
    fn test_memory_mapping_u16_edge_case(addr in 0x0u64..(u32::MAX as u64 - 1 - 3)) {
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
    fn test_memory_mapping_u16(addr in 0x0u64..(u32::MAX as u64 - 2)) {
        let mut cpu = setup_cpu();
        let value = 0x1234u16;
        cpu.write_mem_u16(addr, value).unwrap();
        prop_assert_eq!(cpu.read_mem_u16(addr).unwrap(), value);
    }

    #[test]
    fn test_memory_mapping_u16_misaligned(addr in 0x0u64..(u32::MAX as u64 - 3)) {
        let mut cpu = setup_cpu();
        let value = 0x1234u16;
        cpu.write_mem_u16(addr + 1, value).unwrap();
        prop_assert_eq!(cpu.read_mem_u16(addr + 1).unwrap(), value);
    }

    #[test]
    fn test_memory_mapping_mixed(addr in 0x0u64..(u32::MAX as u64 - 3)) {
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
    fn test_memory_cross_boundary_u32(addr_v in 1u64..10, offset in 0u64..4u64) {
        let mut cpu = setup_cpu();
        let value = 0x12345678u32;
        let addr = addr_v * PAGE_SIZE + offset - 4;
        cpu.write_mem_u32(addr, value).unwrap();
        prop_assert_eq!(cpu.read_mem_u32(addr).unwrap(), value);
        prop_assert_eq!(cpu.read_mem_u32(addr + 1).unwrap(), 0x00123456);
        prop_assert_eq!(cpu.read_mem_u32(addr + 2).unwrap(), 0x00001234);
        prop_assert_eq!(cpu.read_mem_u32(addr + 3).unwrap(), 0x00000012);
    }

    #[test]
    fn test_memory_cross_boundary_u16(addr_v in 1u64..10, offset in 0u64..2u64) {
        let mut cpu = setup_cpu();
        let value = 0x5678u16;
        let addr = addr_v * PAGE_SIZE + offset - 2;
        cpu.write_mem_u16(addr, value).unwrap();
        prop_assert_eq!(cpu.read_mem_u16(addr).unwrap(), value);
        prop_assert_eq!(cpu.read_mem_u16(addr + 1).unwrap(), 0x0056);
    }

    #[test]
    fn test_lb(rd in 1u8..31, rs1 in 1u8..31, imm in 0u16..0xF, value in i8::MIN..i8::MAX) {
        if rs1 == rd {
            return Ok(());
        }
        let mut cpu = setup_cpu();
        let addr = 1000u32;
        cpu.write_x_u32(rs1, addr).unwrap();
        cpu.write_mem_u8(addr.wrapping_add_signed(imm as i16 as i32) as u64, value as u8).unwrap();

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
        cpu.write_mem_u16(addr.wrapping_add_signed(imm as i16 as i32) as u64, value as u16).unwrap();

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
        cpu.write_mem_u32(addr.wrapping_add_signed(0) as u64, value as u32).unwrap();

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
        cpu.write_mem_u8(addr.wrapping_add_signed(imm as i16 as i32) as u64, value).unwrap();

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
        cpu.write_mem_u16(addr.wrapping_add_signed(imm as i16 as i32) as u64, value).unwrap();

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

        prop_assert_eq!(cpu.read_mem_u8(addr.wrapping_add_signed(imm as i16 as i32) as u64).unwrap(), value as u8);
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

        prop_assert_eq!(cpu.read_mem_u16(addr.wrapping_add_signed(imm as i16 as i32) as u64).unwrap(), value as u16);
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

        prop_assert_eq!(cpu.read_mem_u32(addr.wrapping_add_signed(imm as i16 as i32) as u64).unwrap(), value as u32);
    }
}
