use crate::collections::{
    dynarray::DynArray,
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

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.values.write(value);
        self.values.count() - 1
    }
}

#[cfg(debug_assertions)]
pub mod debug {
    use crate::types::opcode::OpCode;

    use super::*;

    impl Chunk {
        pub fn disassemble(&self, name: &str) {
            println!("== {name} ==");
            let mut offset = 0;
            while offset != self.code.count {
                offset = disassemble_instruction(self, offset);
            }
        }
    }

    pub fn disassemble_instruction(chunk: &Chunk, offset: usize) -> usize {
        print!("{offset:04} ");
        print_line_number(chunk, offset);

        let instruction = chunk.code[offset];
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
        if offset > 0 && chunk.lines[offset] == chunk.lines[offset - 1] {
            print!("   | ");
        } else {
            print!("{:4} ", chunk.lines[offset]);
        }
    }

    fn print_constant_instructions(chunk: &Chunk, offset: usize, opcode: OpCode) {
        let constant = unsafe { *(chunk.code.data).add(offset + 1) };
        let value = chunk.values[constant as usize];
        println!("{:<16} {constant:4} '{value}'", opcode.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_write() {
        let mut chunk = Chunk::default();

        chunk.write(1, 1);
        assert_eq!(chunk.code[0], 1);
        assert_eq!(chunk.lines[0], 1);

        chunk.write(2, 2);
        assert_eq!(chunk.code[1], 2);
        assert_eq!(chunk.lines[1], 2);

        chunk.write(3, 3);
        assert_eq!(chunk.code[2], 3);
        assert_eq!(chunk.lines[2], 3);
    }

    #[test]
    fn test_chunk_add_constant() {
        let mut chunk = Chunk::default();

        let index = chunk.add_constant(42.0);
        assert_eq!(index, 0);
        assert_eq!(chunk.values[0], 42.0);

        let index = chunk.add_constant(84.0);
        assert_eq!(index, 1);
        assert_eq!(chunk.values[1], 84.0);
    }
}
