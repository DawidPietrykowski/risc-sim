use super::{cpu::{self, Cpu, Operation}, rv32i::immediate::{AddI, RV32I_SET}};
use anyhow::{Context, Ok, Result};


#[derive(Clone, Copy)]
pub struct ProgramLine{
    pub instruction: Instruction,
    pub word: Word
}


#[derive(Clone, Copy)]
pub struct RInstructionData {
    pub rd: u8,
    pub func3: u8,
    pub rs1: u8,
    pub rs2: u8,
    pub func7: u8,
}

#[derive(Clone, Copy)]
pub struct IInstructionData {
    pub rd: u8,
    pub func3: u8,
    pub rs1: u8,
    pub imm: u16,
}

#[derive(Clone, Copy)]
pub struct SInstructionData {
    pub imm1: u8,
    pub func3: u8,
    pub rs1: u8,
    pub rs2: u8,
    pub imm2: u8,
}

#[derive(Clone, Copy)]
pub struct SBInstructionData {
    pub imm1: u8,
    pub func3: u8,
    pub rs1: u8,
    pub rs2: u8,
    pub imm2: u8,
}

#[derive(Clone, Copy)]
pub struct UInstructionData {
    pub rd: u8,
    pub imm: u32,
}

#[derive(Clone, Copy)]
pub struct UJInstructionData {
    pub rd: u8,
    pub imm: UJImmediate,
}

#[derive(Clone, Copy)]
pub struct UJImmediate (u32);

impl UJImmediate {
    pub fn from(raw_operation: u32) -> UJImmediate {
        let mut filled: u32 = 0;

        let imm_20: bool = (raw_operation >> 31) == 1;
        let imm_10_1: u32 = (raw_operation >> 21) & 0x3FF;
        let imm_11: bool = (raw_operation >> 20) == 1;
        let imm_19_12: u32 = (raw_operation >> 12) & 0xFF;

        filled |= (imm_20 as u32) << 20;
        filled |= imm_10_1 << 1;
        filled |= (imm_11 as u32) << 11;
        filled |= imm_19_12 << 12;

        UJImmediate(filled & 0x3FFFFE)
    }

    pub fn as_i32(&self) -> i32 {
        ((self.0 << 11) as i32) >> 11
    }
}

#[derive(Clone, Copy)]
pub enum InstructionData {
    R(RInstructionData),
    I(IInstructionData),
    S(SInstructionData),
    SB(SBInstructionData),
    U(UInstructionData),
    UJ(UJInstructionData),
}

pub trait InstructionType {}

impl InstructionType for RInstructionData {}
impl InstructionType for IInstructionData {}
impl InstructionType for SInstructionData {}
impl InstructionType for SBInstructionData {}
impl InstructionType for UInstructionData {}
impl InstructionType for UJInstructionData {}


#[derive(Clone, Copy)]
pub struct Word(u32);

#[derive(Clone, Copy)]
pub struct Instruction{
    pub mask: u32,
    pub bits: u32,
    pub name: &'static str,
    pub operation: fn(cpu: &mut Cpu, word: &Word) -> Result<()>
}

pub fn parse_instruction_i(word: &Word) -> IInstructionData {
    IInstructionData {
        rd: (word.0 >> 7) as u8 & 0b00011111,
        func3: (word.0 >> 12) as u8 & 0b00000111,
        rs1: (word.0 >> 15) as u8 & 0b00011111,
        imm: (word.0 >> 20) as u16
    }
}

pub fn parse_instruction_u(word: &Word) -> UInstructionData {
    UInstructionData {
        rd: (word.0 >> 7) as u8 & 0b00011111,
        imm: word.0 & (0xFFFFFFFF << 12)
    }
}

pub fn decode_program_line(word: Word) -> Result<ProgramLine> {
    let instruction = *RV32I_SET.iter().find(|ins| {
        (word.0 & ins.mask) == ins.bits
    }).context("Instruction not found")?;
    Ok(ProgramLine {
        instruction,
        word
    })
}

pub fn encode_program_line(name: &str, instruction_data: InstructionData) -> Result<Word> {
    let instruction = find_instruction_by_name(name)?;
    let mut word = Word(0);
    word.0 |= instruction.mask & instruction.bits;
    word.0 |= match instruction_data {
        InstructionData::R(data) => {
            (data.rd as u32) << 7 | 
            (data.func3 as u32) << 12 | 
            (data.rs1 as u32) << 15 | 
            (data.rs2 as u32) << 20 | 
            (data.func7 as u32) << 25
        },
        InstructionData::I(data) => {
            (data.rd as u32) << 7 | 
            (data.func3 as u32) << 12 | 
            (data.rs1 as u32) << 15 | 
            (data.imm as u32) << 20
        },
        InstructionData::S(_) => todo!(),
        InstructionData::SB(_) => todo!(),
        InstructionData::U(data) => {
            (data.rd as u32) << 7 | 
            (data.imm as u32) << 12
        },
        InstructionData::UJ(_) => todo!(),
    };
    Ok(word)
}

pub const OPCODE_MASK: u32 = 0b01111111;
pub const FUNC3_MASK: u32 = (0b111 as u32) << 12;


pub fn find_instruction_by_name(name: &str) -> Result<Instruction> {
    Ok(*RV32I_SET.iter().find(|ins| {ins.name == name}).context("Function not found")?)
}