#[derive(Clone)]
pub struct RInstruction {
    pub rd: u8,
    pub func3: u8,
    pub rs1: u8,
    pub rs2: u8,
    pub func7: u8,
}

#[derive(Clone)]
pub struct IInstruction {
    pub rd: u8,
    pub func3: u8,
    pub rs1: u8,
    pub imm: u16,
}

#[derive(Clone)]
pub struct SInstruction {
    pub imm1: u8,
    pub func3: u8,
    pub rs1: u8,
    pub rs2: u8,
    pub imm2: u8,
}

#[derive(Clone)]
pub struct UInstruction {
    pub rd: u8,
    pub imm: u32,
}

#[derive(Clone)]
pub enum Instruction {
    R(RInstruction),
    I(IInstruction),
    S(SInstruction),
    U(UInstruction),
}
