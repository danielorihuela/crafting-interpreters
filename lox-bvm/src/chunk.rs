use crate::{
    collections::dynarray::DynArray,
    value::{Value, Values},
};

#[derive(Default)]
pub struct Chunk {
    pub code: DynArray<u8>,
    pub values: Values,
    lines: DynArray<usize>,
}

impl Chunk {
    pub fn write(&mut self, byte: u8, line: usize) {
        self.code.write(byte);
        self.lines.write(line);
    }

    pub fn free(&mut self) {
        self.code.free();
        self.values.free();
        self.lines.free();
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.values.write(value);
        self.values.count() - 1
    }
}

#[cfg(debug_assertions)]
pub mod debug {
    use super::*;
    use crate::OpCode;

    pub fn disassemble_instruction(chunk: &Chunk, offset: usize) -> usize {
        use crate::opcode::OpCode;

        print!("{offset:04} ");
        print_line_number(chunk, offset);

        let instruction = *chunk.code.get_at(offset);
        let opcode = OpCode::from(instruction);
        match opcode {
            OpCode::Constant => {
                print_constant_instructions(chunk, offset, opcode);
                offset + 2
            }
            OpCode::Add
            | OpCode::Subtract
            | OpCode::Multiply
            | OpCode::Divide
            | OpCode::Negate
            | OpCode::Return => {
                println!("{}", opcode);
                offset + 1
            }
            OpCode::Unknown => {
                println!("Unknown opcode {}", instruction);
                offset + 1
            }
        }
    }

    fn print_line_number(chunk: &Chunk, offset: usize) {
        if offset > 0 && *chunk.lines.get_at(offset) == *chunk.lines.get_at(offset - 1) {
            print!("   | ");
        } else {
            print!("{:4} ", *chunk.lines.get_at(offset));
        }
    }

    fn print_constant_instructions(chunk: &Chunk, offset: usize, opcode: OpCode) {
        let constant = unsafe { *(chunk.code.data).add(offset + 1) };
        let value = chunk.values.get_at(constant as usize);
        println!("{:<16} {constant:4} '{value}'", opcode.to_string());
    }
}
