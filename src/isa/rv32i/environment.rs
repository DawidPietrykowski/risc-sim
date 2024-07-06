use crate::isa::types::*;

pub const RV32I_SET_E: [Instruction; 2] = [
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC12_MASK,
        bits: 0b0 << FUNC12_POS | 0b1110011,
        name: "ECALL",
        instruction_type: InstructionType::I,
        operation: |cpu, _word| {
            let syscall_num = cpu.read_x_u32(ABIRegister::A(7).to_x_reg_id() as u8)?;
            match syscall_num {
                64 => {
                    // Write syscall
                    let fd = cpu.read_x_u32(ABIRegister::A(0).to_x_reg_id() as u8)?;
                    let buffer_addr = cpu.read_x_u32(ABIRegister::A(1).to_x_reg_id() as u8)?;
                    let len = cpu.read_x_u32(ABIRegister::A(2).to_x_reg_id() as u8)?;

                    if fd != 1 {
                        todo!()
                    }

                    for i in 0..len {
                        let byte = cpu.read_mem_u8(buffer_addr + i)?;
                        cpu.push_stdout(byte);
                    }
                }
                _ => todo!(),
            };
            Ok(())
        },
    },
    Instruction {
        mask: OPCODE_MASK | FUNC3_MASK | FUNC12_MASK,
        bits: 0b1 << FUNC12_POS | 0b1110011,
        name: "EBREAK",
        instruction_type: InstructionType::I,
        operation: |_cpu, _word| {
            todo!();
        },
    },
];
