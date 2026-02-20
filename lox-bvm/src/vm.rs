use crate::{
    AsciiChar,
    collections::{stack::Stack, value::Value},
    compiler::Parser,
    scanner::Scanner,
    types::{chunk::Chunk, opcode::OpCode},
};

pub struct VM {
    chunk: *mut Chunk,
    ip: *mut u8,

    stack: Stack<Value>,
}

impl VM {
    pub fn new() -> Self {
        VM {
            chunk: std::ptr::null_mut(),
            ip: std::ptr::null_mut(),
            stack: Stack::default(),
        }
    }

    pub fn interpret(&mut self, source: *const AsciiChar) -> InterpretResult {
        let chunk = &mut Chunk::default();

        let scanner = &mut Scanner::new(source);
        let parser = &mut Parser::new(scanner);
        if !parser.compile(chunk) {
            return InterpretResult::CompileError;
        }

        self.chunk = chunk;
        self.ip = unsafe { (*self.chunk).code.data };

        self.run()
    }

    fn run(&mut self) -> InterpretResult {
        loop {
            #[cfg(debug_assertions)]
            {
                use crate::collections::stack::debug::show_stack;
                show_stack(&self.stack);

                use crate::types::chunk::debug::disassemble_instruction;
                let offset = unsafe { self.ip.offset_from((*self.chunk).code.data) } as usize;
                let _ = disassemble_instruction(unsafe { &*self.chunk }, offset);
            }

            let instruction = OpCode::from(unsafe { *self.ip });
            self.ip = unsafe { self.ip.add(1) };

            match instruction {
                OpCode::Constant => {
                    let position = unsafe { *self.ip } as usize;
                    self.ip = unsafe { self.ip.add(1) };

                    let value = &unsafe { &*self.chunk }.values[position];
                    self.stack.push(*value);
                }
                OpCode::Add | OpCode::Subtract | OpCode::Multiply | OpCode::Divide => {
                    if let Some(op) = instruction.maybe_binary_op() {
                        self.binary_op(op);
                    } else {
                        panic!("Unsupported binary operation");
                    }
                }
                OpCode::Negate => {
                    let value = self.stack.pop();
                    self.stack.push(-value);
                }
                OpCode::Return => {
                    let value = self.stack.pop();
                    println!("{}", value);
                    return InterpretResult::Ok;
                }
                OpCode::Unknown => panic!("Something went wrong running the bytecode"),
            }
        }
    }

    fn binary_op(&mut self, op: impl FnOnce(Value, Value) -> Value) {
        let b = self.stack.pop();
        let a = self.stack.pop();
        self.stack.push(op(a, b));
    }

    pub fn free(&mut self) {}
}

#[derive(PartialEq)]
pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

impl InterpretResult {
    pub fn to_exit_code(&self) -> i32 {
        match self {
            InterpretResult::Ok => 0,
            InterpretResult::CompileError => 65,
            InterpretResult::RuntimeError => 70,
        }
    }
}
