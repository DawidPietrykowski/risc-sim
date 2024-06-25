use anyhow::{Ok, Result};

mod isa;
mod test;
mod utils;

fn main() -> Result<()> {
    // let mut cpu = Cpu::new();

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
    // let addi_instruction = IInstruction {
    //     rd: 1,
    //     func3: 0,
    //     rs1: 0,
    //     imm: i16_to_u16(10),
    // };
    // let addi_op = AddI::new(addi_instruction);
    // addi_op.execute(&mut cpu)?;
    // assert_eq!(cpu.read_x_i32(1)?, 10);

    // // Test ADDI -2 to 10 = 8
    // let addi_instruction = IInstruction {
    //     rd: 1,
    //     func3: 0,
    //     rs1: 1,
    //     imm: i16_to_u16(-2),
    // };
    // let addi_op = AddI::new(addi_instruction);
    // addi_op.execute(&mut cpu)?;
    // assert_eq!(cpu.read_x_i32(1)?, 8);

    // // Test SLTI
    // let slti_instruction = IInstruction {
    //     rd: 2,
    //     func3: 2,
    //     rs1: 1,
    //     imm: 15,
    // };
    // let slti_op = SLTI::new(slti_instruction);
    // slti_op.execute(&mut cpu)?;
    // assert_eq!(cpu.read_x_i32(2)?, 1);

    // // Test SLTI
    // let slti_instruction = IInstruction {
    //     rd: 2,
    //     func3: 2,
    //     rs1: 1,
    //     imm: 15,
    // };
    // let slti_op = SLTI::new(slti_instruction);
    // slti_op.execute(&mut cpu)?;
    // assert_eq!(cpu.read_x_i32(2)?, 1);

    // // Test ANDI
    // let andi_instruction = IInstruction {
    //     rd: 3,
    //     func3: 7,
    //     rs1: 1,
    //     imm: 0b1100,
    // };
    // let andi_op = ANDI::new(andi_instruction);
    // andi_op.execute(&mut cpu)?;
    // assert_eq!(cpu.read_x_u32(3)?, 0b1000);

    // // Test ORI
    // let ori_instruction = IInstruction {
    //     rd: 4,
    //     func3: 6,
    //     rs1: 1,
    //     imm: 0b0101,
    // };
    // let ori_op = ORI::new(ori_instruction);
    // ori_op.execute(&mut cpu)?;
    // assert_eq!(cpu.read_x_u32(4)?, 0b0101);

    // // Test XORI
    // let xori_instruction = IInstruction {
    //     rd: 5,
    //     func3: 4,
    //     rs1: 1,
    //     imm: 0b1111,
    // };
    // let xori_op = XORI::new(xori_instruction);
    // xori_op.execute(&mut cpu)?;
    // assert_eq!(cpu.read_x_u32(5)?, 0b0101);

    // println!("All tests passed successfully!");

    Ok(())
}
