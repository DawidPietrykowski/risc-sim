use crate::{
    cpu::cpu_core::PrivilegeMode,
    isa::csr::csr_types::{CSRAddress, MstatusCSR},
    types::{
        Instruction, InstructionType, FUNC3_MASK, FUNC7_MASK, FUNC7_POS, OPCODE_MASK, RS2_MASK,
        RS2_POS,
    },
};

pub const RV64_PRIVILEGED_SET: [Instruction; 4] = [
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK | RS2_MASK,
        bits: 0b0001000 << FUNC7_POS | 0b1110011 | 0b00010 << RS2_POS,
        name: "SRET",
        instruction_type: InstructionType::R,
        operation: |cpu, _word| {
            let mut sstatus = MstatusCSR(
                cpu.csr_table
                    .read_xlen(CSRAddress::Sstatus.as_u12(), cpu.arch_mode),
            );

            //println!("Running SRET");
            //println!("SSTATUS: {:?}", sstatus);

            if !sstatus.spp() {
                cpu.privilege_mode = PrivilegeMode::User;
            }

            sstatus.set_sie(sstatus.spie());
            sstatus.set_spie(true);
            sstatus.set_spp(false);

            cpu.csr_table
                .write_xlen(CSRAddress::Sstatus.as_u12(), sstatus.0, cpu.arch_mode);

            let sepc = cpu
                .csr_table
                .read_xlen(CSRAddress::Sepc.as_u12(), cpu.arch_mode);

            let current_pc = cpu.read_current_instruction_addr_u64();

            //println!("current PC: {:x}", current_pc);
            //println!("SEPC: {:x}", sepc);
            assert_ne!(sepc, current_pc);

            cpu.write_pc_u64(sepc);
            cpu.current_instruction_pc_64 = sepc;

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK | RS2_MASK,
        bits: 0b0011000 << FUNC7_POS | 0b1110011 | 0b00010 << RS2_POS,
        name: "MRET",
        instruction_type: InstructionType::R,
        operation: |cpu, _word| {
            let mut mstatus = MstatusCSR(
                cpu.csr_table
                    .read_xlen(CSRAddress::Mstatus.as_u12(), cpu.arch_mode),
            );

            //println!("Running MRET");
            //println!("MSTATUS: {:?}", mstatus);

            match mstatus.mpp() {
                1 => {
                    cpu.privilege_mode = PrivilegeMode::Supervisor;
                }
                0 => {
                    cpu.privilege_mode = PrivilegeMode::User;
                }
                _ => {
                    panic!("Invalid MPP value");
                }
            }

            mstatus.set_mie(mstatus.mpie());
            mstatus.set_mpie(true);
            mstatus.set_mpp(0);

            cpu.csr_table
                .write_xlen(CSRAddress::Mstatus.as_u12(), mstatus.0, cpu.arch_mode);

            let mepc = cpu
                .csr_table
                .read_xlen(CSRAddress::Mepc.as_u12(), cpu.arch_mode);

            //println!("current PC: {:x}", cpu.read_current_instruction_addr_u64());
            //println!("MEPC: {:x}", mepc);

            cpu.write_pc_u64(mepc);

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0001001 << FUNC7_POS | 0b1110011,
        name: "SFENCE.VMA",
        instruction_type: InstructionType::R,
        operation: |_cpu, _word| Ok(()),
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK | RS2_MASK,
        bits: 0b0001000 << FUNC7_POS | 0b1110011 | 0b00101 << RS2_POS,
        name: "WFI",
        instruction_type: InstructionType::R,
        operation: |_cpu, _word| {
            //bail!("not implemented");

            Ok(())
        },
    },
];
