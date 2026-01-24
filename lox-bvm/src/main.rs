use crate::{chunk::Chunk, opcode::OpCode};

mod chunk;
mod dynarray;
mod opcode;
mod value;

fn main() {
    let mut chunk = Chunk::default();

    let constant = chunk.add_constant(1.2);
    chunk.write(OpCode::OpConstant as u8, 123);
    chunk.write(constant as u8, 123);
    chunk.write(OpCode::OpReturn as u8, 123);

    chunk.disassemble("test chunk");

    chunk.free();
}
