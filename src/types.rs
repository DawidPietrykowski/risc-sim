use std::fmt;

use crate::{
    cpu::cpu_core::{Cpu, CpuMode},
    isa::{
        rv32_zicsr::zicsr::RV32_ZICSR_SET,
        rv32_zifencei::zifencei::RV32_ZIFENCEI_SET,
        rv32i::{
            control_transfer::RV32I_SET_UJ, environment::RV32I_SET_E, immediate::RV32I_SET_I,
            integer_reg_reg::RV32I_SET_R, load_store::RV32I_SET_LS,
        },
        rv32m::muldiv_reg_reg::RV32M_SET_R,
        rv64_privileged::privileged::RV64_PRIVILEGED_SET,
        rv64_zicsr::zicsr::RV64_ZICSR_SET,
        rv64_zifencei::zifencei::RV64_ZIFENCEI_SET,
        rv64a::amo::RV64A_SET_AMO,
        rv64fd::{
            conversion::RV64F_SET_CONVERSION, load_store::RV64F_SET_LS, mov::RV64F_SET_MOV,
            muldiv::RV64F_SET_MULDIV,
        },
        rv64i::{
            control_transfer::RV64I_SET_UJ, environment::RV64I_SET_E, immediate::RV64I_SET_I,
            integer_reg_reg::RV64I_SET_R, load_store::RV64I_SET_LS,
        },
        rv64m::muldiv_reg_reg::RV64M_SET_R,
    },
};
use anyhow::{anyhow, Context, Ok, Result};
use once_cell::sync::Lazy;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ProgramLine {
    pub instruction: Instruction,
    pub word: Word,
}

impl fmt::Display for ProgramLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data = parse_instruction(&self.word, self.instruction.instruction_type);
        write!(f, "{} {:?}", self.instruction, data)
    }
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

        let imm_20: bool = (raw_operation >> 31) & 1 == 1;
        let imm_10_1: u32 = (raw_operation >> 21) & 0x3FF;
        let imm_11: bool = (raw_operation >> 20) & 1 == 1;
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

    pub fn as_i64(&self) -> i64 {
        self.as_i32() as i64
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct SImmediate(pub U12);

impl SImmediate {
    pub fn from(raw_operation: u32) -> SImmediate {
        let mut filled: u32 = 0;

        let imm_11_5: u32 = (raw_operation >> 25) & (U7_MASK as u32);
        let imm_4_0: u32 = (raw_operation >> 7) & (U5_MASK as u32);

        filled |= imm_11_5 << 5;
        filled |= imm_4_0;

        SImmediate(U12::new((filled & 0xFFF) as u16))
    }

    pub fn as_i32(&self) -> i32 {
        self.0.as_i32()
    }

    pub fn as_i64(&self) -> i64 {
        self.0.as_i64()
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct SBImmediate(u32);

impl SBImmediate {
    pub fn from(raw_operation: u32) -> SBImmediate {
        let mut filled: u32 = 0;

        let imm_12: bool = (raw_operation >> 31) & 1 == 1;
        let imm_4_1: u32 = (raw_operation >> 8) & 0xF;
        let imm_11: bool = (raw_operation >> 7) & 1 == 1;
        let imm_10_5: u32 = (raw_operation >> 25) & 0x3F;

        filled |= (imm_12 as u32) << 12;
        filled |= imm_4_1 << 1;
        filled |= (imm_11 as u32) << 11;
        filled |= imm_10_5 << 5;
        filled &= !(0b111 << 13 | 0b1);

        SBImmediate(filled)
    }

    pub fn as_i32(&self) -> i32 {
        ((self.0 << 19) as i32) >> 19
    }

    pub fn as_i64(&self) -> i64 {
        self.as_i32() as i64
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }

    pub fn as_u64(&self) -> u64 {
        self.0 as u64
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

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (", self.name)?;
        match self.instruction_type {
            InstructionType::R => write!(f, "R-type"),
            InstructionType::I => write!(f, "I-type"),
            InstructionType::S => write!(f, "S-type"),
            InstructionType::SB => write!(f, "B-type"),
            InstructionType::U => write!(f, "U-type"),
            InstructionType::UJ => write!(f, "J-type"),
        }?;
        write!(f, ")")
    }
}

pub fn parse_instruction(word: &Word, instruction_type: InstructionType) -> InstructionData {
    match instruction_type {
        InstructionType::R => InstructionData::R(parse_instruction_r(word)),
        InstructionType::I => InstructionData::I(parse_instruction_i(word)),
        InstructionType::S => InstructionData::S(parse_instruction_s(word)),
        InstructionType::SB => InstructionData::SB(parse_instruction_sb(word)),
        InstructionType::U => InstructionData::U(parse_instruction_u(word)),
        InstructionType::UJ => InstructionData::UJ(parse_instruction_uj(word)),
    }
}

pub fn parse_instruction_i(word: &Word) -> IInstructionData {
    IInstructionData {
        rd: U5((word.0 >> 7) as u8 & U5_MASK),
        func3: U3((word.0 >> 12) as u8 & U3_MASK),
        rs1: U5((word.0 >> 15) as u8 & U5_MASK),
        imm: U12((word.0 >> 20) as u16 & 0xFFF),
    }
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
        imm: SImmediate::from(word.0),
    }
}

pub fn parse_instruction_sb(word: &Word) -> SBInstructionData {
    SBInstructionData {
        func3: U3((word.0 >> 12) as u8 & U3_MASK),
        rs1: U5((word.0 >> 15) as u8 & U5_MASK),
        rs2: U5((word.0 >> 20) as u8 & U5_MASK),
        imm: SBImmediate::from(word.0),
    }
}

pub fn parse_instruction_r(word: &Word) -> RInstructionData {
    RInstructionData {
        rd: U5((word.0 >> 7) as u8 & U5_MASK),
        func3: U3((word.0 >> 12) as u8 & U3_MASK),
        rs1: U5((word.0 >> 15) as u8 & U5_MASK),
        rs2: U5((word.0 >> 20) as u8 & U5_MASK),
        func7: U7((word.0 >> 25) as u8 & U7_MASK),
    }
}

pub fn decode_program_line(word: Word, mode: CpuMode) -> Result<ProgramLine> {
    #[cfg(not(feature = "maxperf"))]
    let context = format!("Instruction {:#x} not found", word.0);
    #[cfg(feature = "maxperf")]
    let context = "Instruction not found";
    let instruction = if mode == CpuMode::RV32 {
        *(ALL_INSTRUCTIONS_32
            .iter()
            .find(|ins| (word.0 & ins.mask) == ins.bits)
            .context(context)?)
    } else {
        *(ALL_INSTRUCTIONS_64
            .iter()
            .find(|ins| (word.0 & ins.mask) == ins.bits)
            .context(context)?)
    };
    Ok(ProgramLine { instruction, word })
}

pub fn decode_program_line_unchecked(word: &Word, mode: CpuMode) -> ProgramLine {
    let table = match mode {
        CpuMode::RV32 => &ALL_INSTRUCTIONS_32,
        CpuMode::RV64 => &ALL_INSTRUCTIONS_64,
    };
    let instruction = unsafe {
        table
            .iter()
            .find(|ins| (word.0 & ins.mask) == ins.bits)
            .unwrap_unchecked()
    };
    ProgramLine {
        instruction: *instruction,
        word: *word,
    }
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
                | (((data.imm.0 .0 as u8) & U5_MASK) as u32) << 7
                | (((data.imm.0 .0) & ((U12_MASK) & !(U5_MASK as u16))) as u32) << 20
        }
        InstructionData::SB(_) => todo!(),
        InstructionData::U(data) => (data.rd.value() as u32) << 7 | (data.imm) << 12,
        InstructionData::UJ(_) => todo!(),
    };
    word.0 |= instruction.mask & instruction.bits;
    Ok(word)
}

pub const OPCODE_MASK: u32 = U7_MASK as u32;
pub const FUNC3_MASK: u32 = (U3_MASK as u32) << FUNC3_POS;
pub const FUNC3_POS: u32 = 12;
pub const FUNC7_MASK: u32 = (U7_MASK as u32) << FUNC7_POS;
pub const FUNC7_POS: u32 = 25;
pub const TOP6_MASK: u32 = (U6_MASK as u32) << TOP6_POS;
pub const TOP6_POS: u32 = 26;
pub const FUNC12_MASK: u32 = (U12_MASK as u32) << 20;
pub const FUNC12_POS: u32 = 20;
pub const RS2_MASK: u32 = (U5_MASK as u32) << RS2_POS;
pub const RS2_POS: u32 = 20;
pub const FUNC2_MASK: u32 = (0b11 as u32) << FUNC2_POS;
pub const FUNC2_POS: u32 = 25;

pub const U7_MASK: u8 = 0b1111111;

pub const U6_MASK: u8 = 0b111111;

pub const U5_MASK: u8 = 0b11111;

pub const U3_MASK: u8 = 0b111;

pub const U12_MASK: u16 = 0b111111111111;

pub const FUNC3_ORI: u8 = 0b110;
pub const FUNC3_XORI: u8 = 0b100;

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct U3(pub u8);

impl BitValue<u8> for U3 {
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

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct U5(pub u8);

impl BitValue<u8> for U5 {
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

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct U7(pub u8);

impl BitValue<u8> for U7 {
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

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct U12(pub u16);

impl BitValue<u16> for U12 {
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

    pub fn as_i64(&self) -> i64 {
        self.as_i32() as i64
    }
}

pub trait BitValue<S> {
    fn new(value: S) -> Self;
    fn value(&self) -> S;
    fn max_value() -> Self;
    fn min_value() -> Self;
}

pub fn find_instruction_by_name(name: &str) -> Result<Instruction> {
    Ok(*ALL_INSTRUCTIONS_32
        .iter()
        .find(|ins| ins.name == name)
        .context("Function not found")?)
}

static ALL_INSTRUCTIONS_32: Lazy<Vec<Instruction>> = Lazy::new(|| {
    let mut all = Vec::new();
    all.extend_from_slice(&RV32I_SET_I);
    all.extend_from_slice(&RV32I_SET_LS);
    all.extend_from_slice(&RV32I_SET_R);
    all.extend_from_slice(&RV32I_SET_UJ);
    all.extend_from_slice(&RV32I_SET_E);
    all.extend_from_slice(&RV32M_SET_R);
    all.extend_from_slice(&RV32_ZIFENCEI_SET);
    all.extend_from_slice(&RV32_ZICSR_SET);
    all
});

static ALL_INSTRUCTIONS_64: Lazy<Vec<Instruction>> = Lazy::new(|| {
    let mut all = Vec::new();
    all.extend_from_slice(&RV64I_SET_I);
    all.extend_from_slice(&RV64I_SET_LS);
    all.extend_from_slice(&RV64I_SET_R);
    all.extend_from_slice(&RV64I_SET_UJ);
    all.extend_from_slice(&RV64I_SET_E);
    all.extend_from_slice(&RV64A_SET_AMO);
    all.extend_from_slice(&RV64M_SET_R);
    all.extend_from_slice(&RV64F_SET_LS);
    all.extend_from_slice(&RV64F_SET_CONVERSION);
    all.extend_from_slice(&RV64F_SET_MULDIV);
    all.extend_from_slice(&RV64F_SET_MOV);
    all.extend_from_slice(&RV64_ZIFENCEI_SET);
    all.extend_from_slice(&RV64_ZICSR_SET);
    all.extend_from_slice(&RV64_PRIVILEGED_SET);
    all
});

pub enum ABIRegister {
    Zero,
    RA,
    SP,
    QP,
    TP,
    A(u32),
    S(u32),
    T(u32),
}

impl ABIRegister {
    pub fn from(x_reg_id: u32) -> Result<ABIRegister> {
        match x_reg_id {
            0 => Ok(ABIRegister::Zero),
            1 => Ok(ABIRegister::RA),
            2 => Ok(ABIRegister::SP),
            3 => Ok(ABIRegister::QP),
            4 => Ok(ABIRegister::TP),
            5..=7 => Ok(ABIRegister::T(x_reg_id - 5)),
            8..=9 => Ok(ABIRegister::S(x_reg_id - 8)),
            10..=17 => Ok(ABIRegister::A(x_reg_id - 10)),
            18..=27 => Ok(ABIRegister::S(2 + x_reg_id - 18)),
            28..=31 => Ok(ABIRegister::T(3 + x_reg_id - 28)),
            32..=u32::MAX => Err(anyhow!("Cannot match register id {}", x_reg_id)),
        }
    }

    pub fn to_x_reg_id(&self) -> u32 {
        match self {
            ABIRegister::Zero => 0,
            ABIRegister::RA => 1,
            ABIRegister::SP => 2,
            ABIRegister::QP => 3,
            ABIRegister::TP => 4,
            ABIRegister::A(id) => 10 + id,
            ABIRegister::S(id) => {
                if *id < 2 {
                    8 + id
                } else {
                    18 + id - 2
                }
            }
            ABIRegister::T(id) => {
                if *id < 3 {
                    5 + id
                } else {
                    28 + id - 3
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum RoundingMode {
    RNE = 000, // Round to Nearest (Ties to Even) *approximated*
    RTZ = 001, // Round towards Zero
    RDN = 010, // Round Down (-∞)
    RUP = 011, // Round Up (+∞)
    RMM = 100, // Round to Max Magnitude (not implemented here)
}

impl From<u8> for RoundingMode {
    fn from(value: u8) -> Self {
        value.into()
    }
}

#[derive(Default)]
pub struct FCSR {
    pub invalid: bool,   // Invalid operation
    pub overflow: bool,  // Overflow
    pub underflow: bool, // Underflow
    pub inexact: bool,   // Inexact result
}
