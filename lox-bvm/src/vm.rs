use crate::{chunk::Chunk, collections::stack::Stack, opcode::OpCode, value::Value};

pub struct VM {
    chunk: *mut Chunk,
    ip: *mut u8,

    stack: Stack<Value>,
}

impl VM {
    pub fn init() -> Self {
        VM {
            chunk: std::ptr::null_mut(),
            ip: std::ptr::null_mut(),
            stack: Stack::default(),
        }
    }

    pub fn interpret(&mut self, chunk: &mut Chunk) {
        self.chunk = chunk;
        self.ip = chunk.code.data;

        let _ = self.run();
    }

    fn run(&mut self) -> InterpretResult {
        loop {
            #[cfg(debug_assertions)]
            {
                use crate::collections::stack::debug::show_stack;
                show_stack(&self.stack);

                use crate::chunk::debug::disassemble_instruction;
                let offset = unsafe { self.ip.offset_from((*self.chunk).code.data) } as usize;
                let _ = disassemble_instruction(unsafe { &*self.chunk }, offset);
            }

            let instruction = OpCode::from(unsafe { *self.ip });
            self.ip = unsafe { self.ip.add(1) };

            match instruction {
                OpCode::Constant => {
                    let position = unsafe { *self.ip } as usize;
                    self.ip = unsafe { self.ip.add(1) };

                    let value = &unsafe { &*self.chunk }.values.get_at(position);
                    self.stack.push(*value);
                }
                OpCode::Add => {
                    self.binary_op(|a, b| a + b);
                }
                OpCode::Subtract => {
                    self.binary_op(|a, b| a - b);
                }
                OpCode::Multiply => {
                    self.binary_op(|a, b| a * b);
                }
                OpCode::Divide => {
                    self.binary_op(|a, b| a / b);
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

pub enum InterpretResult {
    Ok,
    _CompileError,
    _RuntimeError,
}
