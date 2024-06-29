use super::{cpu::Cpu, rv32i::immediate::RV32I_SET};
use anyhow::{Context, Ok, Result};


#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ProgramLine{
    pub instruction: Instruction,
    pub word: Word
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

// impl IInstructionData {
//     pub fn new(rd: u8, func3: U3, rs1: u8, imm: u16) -> IInstructionData {
//         assert_eq!(rd & 0b11100000, 0);
//         assert_eq!(func3 & 0b11111000, 0);
//         assert_eq!(rs1 & 0b11100000, 0);
//         assert_eq!(imm & 0xF000, 0);
//         IInstructionData { rd, func3, rs1, imm }
//     }
// }

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct SInstructionData {
    pub func3: U3,
    pub rs1: U5,
    pub rs2: U5,
    pub imm: u32,
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct SBInstructionData {
    pub func3: U3,
    pub rs1: U5,
    pub rs2: U5,
    pub imm: u32,
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

#[derive(Clone, Copy, Debug)]
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


#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Word(pub u32);

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Instruction{
    pub mask: u32,
    pub bits: u32,
    pub name: &'static str,
    pub operation: fn(cpu: &mut Cpu, word: &Word) -> Result<()>
}

pub fn parse_instruction_i(word: &Word) -> IInstructionData {
    let data = IInstructionData {
        rd: U5((word.0 >> 7) as u8 & 0b11111),
        func3: U3((word.0 >> 12) as u8 & 0b111),
        rs1: U5((word.0 >> 15) as u8 & 0b11111),
        imm: U12((word.0 >> 20) as u16 & 0xFFF)
    };
    println!("{:#034b} {:?}", word.0, data);
    data
}

pub fn parse_instruction_u(word: &Word) -> UInstructionData {
    UInstructionData {
        rd: U5((word.0 >> 7) as u8 & 0b00011111),
        imm: word.0 & (0xFFFFFFFF << 12)
    }
}

pub fn parse_instruction_r(word: &Word) -> RInstructionData {
    let data = RInstructionData {
        rd: U5((word.0 >> 7) as u8 & 0b11111),
        func3: U3((word.0 >> 12) as u8 & 0b111),
        rs1: U5((word.0 >> 15) as u8 & 0b11111),
        rs2: U5((word.0 >> 20) as u8 & 0b11111),
        func7: U7((word.0 >> 25) as u8 & 0x7F)
    };
    println!("{:#034b} {:?}", word.0, data);
    data
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
    word.0 |= match instruction_data {
        InstructionData::R(data) => {
            (data.rd.value() as u32) << 7 | 
            (data.func3.value() as u32) << 12 | 
            (data.rs1.value() as u32) << 15 | 
            (data.rs2.value() as u32) << 20 | 
            (data.func7.value() as u32) << 25
        },
        InstructionData::I(data) => {
            (data.rd.value() as u32) << 7 | 
            (data.func3.value() as u32) << 12 | 
            (data.rs1.value() as u32) << 15 | 
            (data.imm.value() as u32) << 20
        },
        InstructionData::S(_) => todo!(),
        InstructionData::SB(_) => todo!(),
        InstructionData::U(data) => {
            (data.rd.value() as u32) << 7 | 
            (data.imm as u32) << 12
        },
        InstructionData::UJ(_) => todo!(),
    };
    word.0 |= instruction.mask & instruction.bits;
    println!("{:#034b} {:?} {}", word.0, instruction_data, instruction.name);
    Ok(word)
}

pub const OPCODE_MASK: u32 = 0b01111111;
pub const FUNC3_MASK: u32 = (0b111 as u32) << FUNC3_POS;
pub const FUNC3_POS: u32 = 12;
pub const FUNC7_MASK: u32 = (0b111111 as u32) << FUNC7_POS;
pub const FUNC7_POS: u32 = 25;

pub const FUNC3_ORI: u8 = 0b110;
pub const FUNC3_XORI: u8 = 0b100;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct U3 (pub u8);

impl BitValue<u8, U3> for U3 {
    fn new(value: u8) -> U3 {
        assert_eq!(value & (0b11111 << 3), 0);
        U3(value & 0b111)
    }

    fn value(&self) -> u8 {
        self.0
    }

    fn max_value() -> U3 {
        U3(0b111)
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
pub struct U5 (pub u8);

impl BitValue<u8, U5> for U5 {
    fn new(value: u8) -> U5 {
        assert_eq!(value & (0b111 << 5), 0);
        U5(value & !(0b111 << 5))
    }

    fn value(&self) -> u8 {
        self.0
    }

    fn max_value() -> U5 {
        U5(0b11111)
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
pub struct U7 (pub u8);

impl BitValue<u8, U7> for U7 {
    fn new(value: u8) -> U7 {
        assert_eq!(value & (0b1 << 7), 0);
        U7(value & !(0b1 << 7))
    }

    fn value(&self) -> u8 {
        self.0
    }

    fn max_value() -> U7 {
        U7(!(0b1 << 7))
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
pub struct U12 (pub u16);

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
    Ok(*RV32I_SET.iter().find(|ins| {ins.name == name}).context("Function not found")?)
}