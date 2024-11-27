use crate::{
    cpu::cpu_core::PrivilegeMode,
    isa::csr::csr_types::{CSRAddress, MstatusCSR},
    types::{Instruction, InstructionType, FUNC3_MASK, FUNC7_MASK, FUNC7_POS, OPCODE_MASK},
};

pub const RV64_PRIVILEGED_SET: [Instruction; 3] = [
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0001000 << FUNC7_POS | 0b1110011,
        name: "SRET",
        instruction_type: InstructionType::R,
        operation: |cpu, _word| {
            let mut sstatus = MstatusCSR(
                cpu.csr_table
                    .read_xlen(CSRAddress::Mstatus.as_u12(), cpu.arch_mode),
            );

            if !sstatus.spp() {
                cpu.privilege_mode = PrivilegeMode::User;
            }

            sstatus.set_spie(sstatus.spie());
            sstatus.set_spie(true);
            sstatus.set_spp(false);

            cpu.write_pc_u64(
                cpu.csr_table
                    .read_xlen(CSRAddress::Sepc.as_u12(), cpu.arch_mode),
            );

            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC7_MASK,
        bits: 0b0011000 << FUNC7_POS | 0b1110011,
        name: "MRET",
        instruction_type: InstructionType::R,
        operation: |cpu, _word| {
            let mut mstatus = MstatusCSR(
                cpu.csr_table
                    .read_xlen(CSRAddress::Mstatus.as_u12(), cpu.arch_mode),
            );

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

            cpu.write_pc_u64(
                cpu.csr_table
                    .read_xlen(CSRAddress::Mepc.as_u12(), cpu.arch_mode),
            );

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
];
