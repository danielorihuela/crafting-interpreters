use crate::{
    dynarray::DynArray,
    opcode::OpCode,
    value::{Value, Values},
};

#[derive(Default)]
pub struct Chunk {
    dyn_array: DynArray<u8>,
    values: Values,
    lines: DynArray<usize>,
}

impl Chunk {
    pub fn write(&mut self, byte: u8, line: usize) {
        self.dyn_array.write(byte);
        self.lines.write(line);
    }

    pub fn free(&mut self) {
        self.dyn_array.free();
        self.values.free();
        self.lines.free();
    }

    pub fn disassemble(&self, name: &str) {
        #[cfg(debug_assertions)]
        {
            println!("== {name} ==");
            let mut offset = 0;
            while offset != self.dyn_array.count {
                offset = disassemble_instruction(self, offset);
            }
        }
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.values.write(value);
        self.values.count() - 1
    }
}

#[cfg(debug_assertions)]
fn disassemble_instruction(chunk: &Chunk, offset: usize) -> usize {
    use crate::opcode::OpCode;

    print!("{offset:04} ");
    print_line_number(chunk, offset);

    let instruction = unsafe { *(chunk.dyn_array.data as *const u8).add(offset) };
    match OpCode::try_from(instruction) {
        Ok(opcode) => match opcode {
            OpCode::OpConstant => {
                print_constant_instructions(chunk, offset, opcode);
                offset + 2
            }
            OpCode::OpReturn => {
                println!("{}", opcode);
                offset + 1
            }
        },
        _ => {
            println!("Unknown opcode {}", instruction);
            offset + 1
        }
    }
}

fn print_line_number(chunk: &Chunk, offset: usize) {
    if offset > 0 && unsafe { *chunk.lines.data.add(offset) == *chunk.lines.data.add(offset - 1) } {
        print!("   | ");
    } else {
        print!("{:4} ", unsafe { *chunk.lines.data.add(offset) });
    }
}

fn print_constant_instructions(chunk: &Chunk, offset: usize, opcode: OpCode) {
    let constant = unsafe { *(chunk.dyn_array.data).add(offset + 1) };
    let value = chunk.values.get_at(constant as usize);
    println!("{:<16} {constant:4} '{value}'", opcode.to_string());
}
