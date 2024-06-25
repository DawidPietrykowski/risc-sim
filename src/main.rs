use anyhow::{anyhow, Context, Ok, Result};

mod test;

#[derive(Clone)]
struct RInstruction {
    rd: u8,
    func3: u8,
    rs1: u8,
    rs2: u8,
    func7: u8,
}

#[derive(Clone)]
struct IInstruction {
    rd: u8,
    func3: u8,
    rs1: u8,
    imm: u16,
}

#[derive(Clone)]
struct SInstruction {
    imm1: u8,
    func3: u8,
    rs1: u8,
    rs2: u8,
    imm2: u8,
}

#[derive(Clone)]
struct UInstruction {
    rd: u8,
    imm: u32,
}

#[derive(Clone)]
enum Instruction{
    R(RInstruction),
    I(IInstruction),
    S(SInstruction),
    U(UInstruction)
}

struct Cpu{
    reg_x32: [u32; 31],
    reg_pc: u32
}

impl Cpu { 
    pub fn new() -> Cpu {
        Cpu { 
            reg_x32: [0x0; 31],
            reg_pc: 0x0
        }
    }  

    pub fn execute_operation<I>(&mut self, operation: &impl Operation<I>) -> Result<()> {
        operation.execute(self)?;
        self.reg_pc += 1;
        Ok(())
    }

    pub fn read_x_u32(&self, id: u8) -> Result<u32> {
        let value = self.reg_x32.get(id as usize)
        .context(format!("Register x{} does not exist", id))?;
        
        Ok(*value)
    }

    pub fn read_x_i32(&self, id: u8) -> Result<i32> {
        Ok(u32_to_i32(self.read_x_u32(id)?))
    }

    pub fn write_x_i32(&mut self, id: u8, value: i32) -> Result<()> {
        let reg_value = self.reg_x32.get_mut(id as usize)
        .context(format!("Register x{} does not exist", id))?;

        *reg_value = i32_to_u32(value);
        Ok(())
    }
}

trait Operation<I>{
    fn new(instruction: I) -> Self;
    fn instruction(&self) -> &I;
    
    fn execute(&self, cpu: &mut Cpu) -> Result<()>;
}

fn u16_to_i16(value: u16) -> i16 {
    i16::from_ne_bytes(value.to_ne_bytes())
}

fn i16_to_u16(value: i16) -> u16 {
    u16::from_ne_bytes(value.to_ne_bytes())
}

fn u32_to_i32(value: u32) -> i32 {
    i32::from_ne_bytes(value.to_ne_bytes())
}

fn i32_to_u32(value: i32) -> u32 {
    u32::from_ne_bytes(value.to_ne_bytes())
}

fn sign_extend_12bit_to_16bit(value: u16) -> i16{
    let positive: bool = (value & (0b1 << 11)) == 0;
    match positive {
        true => u16_to_i16(value),
        false => u16_to_i16(value | (0b1111 << 12))
    }
}

fn sign_extend_12bit_to_32bit(value: u16) -> i32{
    let positive: bool = (value & (0b1 << 11)) == 0;
    let padding: u32 = match positive {
        true => 0x0,
        false => 0xFFFFF000,
    };
    u32_to_i32(padding | value as u32)
}

struct AddI{
    instruction: IInstruction
}

impl Operation<IInstruction> for AddI {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let imm = sign_extend_12bit_to_32bit(self.instruction.imm);
        let rs1 = cpu.read_x_i32(self.instruction.rs1)?;
        let (res, _) = imm.overflowing_add(rs1);
        cpu.write_x_i32(self.instruction.rd, res)?;
        Ok(())
    }
    
    fn new(instruction: IInstruction) -> Self {
        AddI { instruction: instruction }
    }
    
    fn instruction(&self) -> &IInstruction {
        &self.instruction
    }
}
struct SLTI{
    instruction: IInstruction
}

impl Operation<IInstruction> for SLTI {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let imm = sign_extend_12bit_to_32bit(self.instruction.imm);
        let rs1 = cpu.read_x_i32(self.instruction.rs1)?;
        cpu.write_x_i32(self.instruction.rd, if rs1 < imm {1} else {0})?;
        Ok(())
    }
    
    fn new(instruction: IInstruction) -> Self {
        SLTI { instruction: instruction }
    }
    
    fn instruction(&self) -> &IInstruction {
        &self.instruction
    }
}

struct ANDI{
    instruction: IInstruction
}

impl Operation<IInstruction> for ANDI {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let imm = sign_extend_12bit_to_32bit(self.instruction.imm);
        let rs1 = cpu.read_x_i32(self.instruction.rs1)?;
        cpu.write_x_i32(self.instruction.rd, rs1 & imm)?;
        Ok(())
    }
    
    fn new(instruction: IInstruction) -> Self {
        ANDI { instruction: instruction }
    }
    
    fn instruction(&self) -> &IInstruction {
        &self.instruction
    }
}
struct ORI{
    instruction: IInstruction
}

impl Operation<IInstruction> for ORI {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let imm = sign_extend_12bit_to_32bit(self.instruction.imm);
        let rs1 = cpu.read_x_i32(self.instruction.rs1)?;
        cpu.write_x_i32(self.instruction.rd, rs1 & imm)?;
        Ok(())
    }
    
    fn new(instruction: IInstruction) -> Self {
        ORI { instruction: instruction }
    }
    
    fn instruction(&self) -> &IInstruction {
        &self.instruction
    }
}
struct XORI{
    instruction: IInstruction
}

impl Operation<IInstruction> for XORI {
    fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        let imm = sign_extend_12bit_to_32bit(self.instruction.imm);
        let rs1 = cpu.read_x_i32(self.instruction.rs1)?;
        cpu.write_x_i32(self.instruction.rd, rs1 & imm)?;
        Ok(())
    }
    
    fn new(instruction: IInstruction) -> Self {
        XORI { instruction: instruction }
    }
    
    fn instruction(&self) -> &IInstruction {
        &self.instruction
    }
}


fn main() -> Result<()> {
    let mut cpu = Cpu::new();

    // let num: u16 = 42;
    // println!("{:08b}", num);
    // let bit_12: u16 = 0b101111111111;
    // let extended_12 = sign_extend_12bit_to_16bit(bit_12);

    // let signed_n10: u16 = 0b111111110110;
    // let signed_10: u16 = 0b000000001010;

    // println!("{:08b}", extended_12);

    // println!("{}", extended_12);

    // println!("{}", sign_extend_12bit_to_16bit(signed_10));
    // println!("{}", sign_extend_12bit_to_16bit(signed_n10));



    // Test ADDI 10 to 0 = 10
    let addi_instruction = IInstruction {
        rd: 1,
        func3: 0,
        rs1: 0,
        imm: i16_to_u16(10),
    };
    let addi_op = AddI::new(addi_instruction);
    addi_op.execute(&mut cpu)?;
    assert_eq!(cpu.read_x_i32(1)?, 10);

    // Test ADDI -2 to 10 = 8
    let addi_instruction = IInstruction {
        rd: 1,
        func3: 0,
        rs1: 1,
        imm: i16_to_u16(-2),
    };
    let addi_op = AddI::new(addi_instruction);
    addi_op.execute(&mut cpu)?;
    assert_eq!(cpu.read_x_i32(1)?, 8);

    // Test SLTI 
    let slti_instruction = IInstruction {
        rd: 2,
        func3: 2,
        rs1: 1,
        imm: 15,
    };
    let slti_op = SLTI::new(slti_instruction);
    slti_op.execute(&mut cpu)?;
    assert_eq!(cpu.read_x_i32(2)?, 1);

    // Test SLTI
    let slti_instruction = IInstruction {
        rd: 2,
        func3: 2,
        rs1: 1,
        imm: 15,
    };
    let slti_op = SLTI::new(slti_instruction);
    slti_op.execute(&mut cpu)?;
    assert_eq!(cpu.read_x_i32(2)?, 1);

    // Test ANDI
    let andi_instruction = IInstruction {
        rd: 3,
        func3: 7,
        rs1: 1,
        imm: 0b1100,
    };
    let andi_op = ANDI::new(andi_instruction);
    andi_op.execute(&mut cpu)?;
    assert_eq!(cpu.read_x_u32(3)?, 0b1000);

    // Test ORI
    let ori_instruction = IInstruction {
        rd: 4,
        func3: 6,
        rs1: 1,
        imm: 0b0101,
    };
    let ori_op = ORI::new(ori_instruction);
    ori_op.execute(&mut cpu)?;
    assert_eq!(cpu.read_x_u32(4)?, 0b0101);

    // Test XORI
    let xori_instruction = IInstruction {
        rd: 5,
        func3: 4,
        rs1: 1,
        imm: 0b1111,
    };
    let xori_op = XORI::new(xori_instruction);
    xori_op.execute(&mut cpu)?;
    assert_eq!(cpu.read_x_u32(5)?, 0b0101);

    println!("All tests passed successfully!");
    Ok(())

}
