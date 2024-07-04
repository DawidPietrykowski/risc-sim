use super::{
    cpu::Cpu,
    rv32i::{
        control_transfer::RV32I_SET_UJ, environment::RV32I_SET_E, immediate::RV32I_SET_I,
        integer_reg_reg::RV32I_SET_R, load_store::RV32I_SET_LS,
    },
};
use anyhow::{Context, Ok, Result};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ProgramLine {
    pub instruction: Instruction,
    pub word: Word,
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct RInstructionData {
    pub rd: U5,
    pub func3: U3,
    pub rs1: U5,
    pub rs2: U5,
    pub func7: U7,
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct IInstructionData {
    pub rd: U5,
    pub func3: U3,
    pub rs1: U5,
    pub imm: U12,
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct SInstructionData {
    pub func3: U3,
    pub rs1: U5,
    pub rs2: U5,
    pub imm: SImmediate,
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct SBInstructionData {
    pub func3: U3,
    pub rs1: U5,
    pub rs2: U5,
    pub imm: SBImmediate,
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct UInstructionData {
    pub rd: U5,
    pub imm: u32,
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct UJInstructionData {
    pub rd: U5,
    pub imm: UJImmediate,
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct UJImmediate(u32);

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

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct SImmediate(pub u32);

impl SImmediate {
    pub fn from(raw_operation: u32) -> SImmediate {
        let mut filled: u32 = 0;

        let imm_11_5: u32 = (raw_operation >> 25) & (U7_MASK as u32);
        let imm_4_0: u32 = (raw_operation >> 7) & (U5_MASK as u32);

        filled |= imm_11_5 << 5;
        filled |= imm_4_0;

        SImmediate(filled & 0xFFF)
    }

    pub fn as_i32(&self) -> i32 {
        ((self.0 << 11) as i32) >> 11
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct SBImmediate(u32);

impl SBImmediate {
    pub fn from(raw_operation: u32) -> SBImmediate {
        let mut filled: u32 = 0;

        let imm_12: bool = (raw_operation >> 31) == 1;
        let imm_4_1: u32 = (raw_operation >> 8) & 0xF;
        let imm_11: bool = (raw_operation >> 7) == 1;
        let imm_10_5: u32 = (raw_operation >> 25) & 0x3F;

        filled |= (imm_12 as u32) << 12;
        filled |= imm_4_1 << 1;
        filled |= (imm_11 as u32) << 11;
        filled |= imm_10_5 << 5;

        SBImmediate(filled & !(0b1111 << 12 | 0b1))
    }

    pub fn as_i32(&self) -> i32 {
        ((self.0 << 11) as i32) >> 11
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug)]
pub enum InstructionData {
    R(RInstructionData),
    I(IInstructionData),
    S(SInstructionData),
    SB(SBInstructionData),
    U(UInstructionData),
    UJ(UJInstructionData),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum InstructionType {
    R,
    I,
    S,
    SB,
    U,
    UJ,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Word(pub u32);

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Instruction {
    pub mask: u32,
    pub bits: u32,
    pub name: &'static str,
    pub instruction_type: InstructionType,
    pub operation: fn(cpu: &mut Cpu, word: &Word) -> Result<()>,
}

pub fn parse_instruction_i(word: &Word) -> IInstructionData {
    let data = IInstructionData {
        rd: U5((word.0 >> 7) as u8 & U5_MASK),
        func3: U3((word.0 >> 12) as u8 & U3_MASK),
        rs1: U5((word.0 >> 15) as u8 & U5_MASK),
        imm: U12((word.0 >> 20) as u16 & 0xFFF),
    };
    println!("{:#034b} {:?}", word.0, data);
    data
}

pub fn parse_instruction_u(word: &Word) -> UInstructionData {
    UInstructionData {
        rd: U5((word.0 >> 7) as u8 & U5_MASK),
        imm: word.0 & (0xFFFFFFFF << 12),
    }
}

pub fn parse_instruction_uj(word: &Word) -> UJInstructionData {
    UJInstructionData {
        rd: U5((word.0 >> 7) as u8 & U5_MASK),
        imm: UJImmediate::from(word.0),
    }
}

pub fn parse_instruction_s(word: &Word) -> SInstructionData {
    SInstructionData {
        func3: U3((word.0 >> 12) as u8 & U3_MASK),
        rs1: U5((word.0 >> 15) as u8 & U5_MASK),
        rs2: U5((word.0 >> 20) as u8 & U5_MASK),
        imm: SImmediate::from(word.0 as u32),
    }
}

pub fn parse_instruction_sb(word: &Word) -> SBInstructionData {
    SBInstructionData {
        func3: U3((word.0 >> 12) as u8 & U3_MASK),
        rs1: U5((word.0 >> 15) as u8 & U5_MASK),
        rs2: U5((word.0 >> 20) as u8 & U5_MASK),
        imm: SBImmediate::from(word.0 as u32),
    }
}

pub fn parse_instruction_r(word: &Word) -> RInstructionData {
    let data = RInstructionData {
        rd: U5((word.0 >> 7) as u8 & U5_MASK),
        func3: U3((word.0 >> 12) as u8 & U3_MASK),
        rs1: U5((word.0 >> 15) as u8 & U5_MASK),
        rs2: U5((word.0 >> 20) as u8 & U5_MASK),
        func7: U7((word.0 >> 25) as u8 & U7_MASK),
    };
    println!("{:#034b} {:?}", word.0, data);
    data
}

pub fn decode_program_line(word: Word) -> Result<ProgramLine> {
    let instruction = *ALL_INSTRUCTIONS
        .iter()
        .find(|ins| (word.0 & ins.mask) == ins.bits)
        .context("Instruction not found")?;
    Ok(ProgramLine { instruction, word })
}

pub fn encode_program_line(name: &str, instruction_data: InstructionData) -> Result<Word> {
    let instruction = find_instruction_by_name(name)?;
    let mut word = Word(0);
    word.0 |= match instruction_data {
        InstructionData::R(data) => {
            (data.rd.value() as u32) << 7
                | (data.func3.value() as u32) << 12
                | (data.rs1.value() as u32) << 15
                | (data.rs2.value() as u32) << 20
                | (data.func7.value() as u32) << 25
        }
        InstructionData::I(data) => {
            (data.rd.value() as u32) << 7
                | (data.func3.value() as u32) << 12
                | (data.rs1.value() as u32) << 15
                | (data.imm.value() as u32) << 20
        }
        InstructionData::S(data) => {
            (data.func3.value() as u32) << 12
                | (data.rs1.value() as u32) << 15
                | (data.rs2.value() as u32) << 20
                | ((data.imm.0) & U5_MASK as u32) << 7
                | ((data.imm.0) & ((U12_MASK as u32) & !(U5_MASK as u32))) << 20
        }
        InstructionData::SB(_) => todo!(),
        InstructionData::U(data) => (data.rd.value() as u32) << 7 | (data.imm as u32) << 12,
        InstructionData::UJ(_) => todo!(),
    };
    word.0 |= instruction.mask & instruction.bits;
    // println!(
    //     "{:#034b} {:?} {}",
    //     word.0, instruction_data, instruction.name
    // );
    Ok(word)
}

pub const OPCODE_MASK: u32 = U7_MASK as u32;
pub const FUNC3_MASK: u32 = (U3_MASK as u32) << FUNC3_POS;
pub const FUNC3_POS: u32 = 12;
pub const FUNC7_MASK: u32 = (U7_MASK as u32) << FUNC7_POS;
pub const FUNC7_POS: u32 = 25;
pub const FUNC12_MASK: u32 = (U12_MASK as u32) << 20;
pub const FUNC12_POS: u32 = 20;

const U7_MASK: u8 = 0b1111111;
const U7_SHIFT: u8 = 7;

const U5_MASK: u8 = 0b11111;
const U5_SHIFT: u8 = 5;

const U3_MASK: u8 = 0b111;
const U3_SHIFT: u8 = 3;

const U12_MASK: u16 = 0b111111111111;
const U12_SHIFT: u8 = 12;

pub const FUNC3_ORI: u8 = 0b110;
pub const FUNC3_XORI: u8 = 0b100;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct U3(pub u8);

impl BitValue<u8, U3> for U3 {
    fn new(value: u8) -> U3 {
        assert_eq!(value & !U3_MASK, 0);
        U3(value & U3_MASK)
    }

    fn value(&self) -> u8 {
        self.0
    }

    fn max_value() -> U3 {
        U3(U3_MASK)
    }

    fn min_value() -> U3 {
        U3(0)
    }
}

impl Default for U3 {
    fn default() -> Self {
        U3(0)
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct U5(pub u8);

impl BitValue<u8, U5> for U5 {
    fn new(value: u8) -> U5 {
        assert_eq!(value & !U5_MASK, 0);
        U5(value & U5_MASK)
    }

    fn value(&self) -> u8 {
        self.0
    }

    fn max_value() -> U5 {
        U5(U5_MASK)
    }

    fn min_value() -> U5 {
        U5(0)
    }
}

impl Default for U5 {
    fn default() -> Self {
        U5(0)
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct U7(pub u8);

impl BitValue<u8, U7> for U7 {
    fn new(value: u8) -> U7 {
        assert_eq!(value & !U7_MASK, 0);
        U7(value & U7_MASK)
    }

    fn value(&self) -> u8 {
        self.0
    }

    fn max_value() -> U7 {
        U7(U7_MASK)
    }

    fn min_value() -> U7 {
        U7(0)
    }
}

impl Default for U7 {
    fn default() -> Self {
        U7(0)
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct U12(pub u16);

impl BitValue<u16, U12> for U12 {
    fn new(value: u16) -> U12 {
        assert_eq!(value & (0b1111 << 12), 0);
        U12(value & !(0b1111 << 12))
    }

    fn value(&self) -> u16 {
        self.0
    }

    fn max_value() -> U12 {
        U12(!(0b1111 << 12))
    }

    fn min_value() -> U12 {
        U12(0)
    }
}

impl U12 {
    pub fn as_i32(&self) -> i32 {
        (((self.0 << 4) as i16) >> 4) as i32
    }
}

impl Default for U12 {
    fn default() -> Self {
        U12(0)
    }
}

pub trait BitValue<S, T> {
    fn new(value: S) -> T;
    fn value(&self) -> S;
    fn max_value() -> T;
    fn min_value() -> T;
}

pub fn find_instruction_by_name(name: &str) -> Result<Instruction> {
    Ok(*ALL_INSTRUCTIONS
        .iter()
        .find(|ins| ins.name == name)
        .context("Function not found")?)
}
use lazy_static::lazy_static;

lazy_static! {
    static ref ALL_INSTRUCTIONS: Vec<Instruction> = {
        let mut all = Vec::new();
        all.extend_from_slice(&RV32I_SET_I);
        all.extend_from_slice(&RV32I_SET_R);
        all.extend_from_slice(&RV32I_SET_UJ);
        all.extend_from_slice(&RV32I_SET_LS);
        all.extend_from_slice(&RV32I_SET_E);
        all
    };
}
