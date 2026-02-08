use crate::{chunk::Chunk, opcode::OpCode, vm::VM};

mod chunk;
mod collections;
mod opcode;
mod value;
mod vm;

fn main() {
    let mut vm = VM::init();

    let mut chunk = Chunk::default();

    let constant = chunk.add_constant(1.2);
    chunk.write(OpCode::Constant as u8, 123);
    chunk.write(constant as u8, 123);

    let constant = chunk.add_constant(3.4);
    chunk.write(OpCode::Constant as u8, 123);
    chunk.write(constant as u8, 123);

    chunk.write(OpCode::Add as u8, 123);

    let constant = chunk.add_constant(5.6);
    chunk.write(OpCode::Constant as u8, 123);
    chunk.write(constant as u8, 123);

    chunk.write(OpCode::Divide as u8, 123);
    chunk.write(OpCode::Negate as u8, 123);

    chunk.write(OpCode::Return as u8, 123);

    vm.interpret(&mut chunk);
    vm.free();

    chunk.free();
}
