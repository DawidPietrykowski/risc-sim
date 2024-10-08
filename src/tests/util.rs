use crate::{
    cpu::{self, cpu_core::Cpu, memory::raw_vec_memory::RawVecMemory},
    system::passthrough_kernel::PassthroughKernel,
    types::{
        encode_program_line, BitValue, IInstructionData, InstructionData, SImmediate,
        SInstructionData, UInstructionData, U12, U5,
    },
};

use anyhow::Result;

pub fn setup_cpu() -> Cpu {
    Cpu::default()
}

pub fn setup_cpu_64() -> Cpu {
    Cpu::new(
        RawVecMemory::default(),
        PassthroughKernel::default(),
        cpu::cpu_core::CpuMode::RV64,
    )
}

pub fn execute_s_instruction(
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
pub fn execute_i_instruction(cpu: &mut Cpu, opcode: &str, rd: u8, rs1: u8, imm: u16) -> Result<()> {
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

pub fn execute_u_instruction(cpu: &mut Cpu, opcode: &str, rd: u8, imm: u32) -> Result<()> {
    let instruction = UInstructionData { rd: U5(rd), imm };
    let op = encode_program_line(opcode, InstructionData::U(instruction))?;
    cpu.execute_word(op)?;
    Ok(())
}

pub const MAX_CYCLES: u32 = 1000000;
