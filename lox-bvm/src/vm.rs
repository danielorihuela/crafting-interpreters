use crate::{
    AsciiChar,
    collections::stack::Stack,
    compiler::Parser,
    scanner::Scanner,
    types::{chunk::Chunk, opcode::OpCode, value::Value},
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
                    self.stack.push(value.clone());
                }
                OpCode::Add
                | OpCode::Subtract
                | OpCode::Multiply
                | OpCode::Divide
                | OpCode::Greater
                | OpCode::Less => {
                    if !Value::is_number(self.stack.peek_at(0))
                        || !Value::is_number(self.stack.peek_at(1))
                    {
                        self.runtime_error("Operands must be numbers.");
                        return InterpretResult::RuntimeError;
                    }

                    if let Some(op) = instruction.maybe_binary_op() {
                        let b = self.stack.pop();
                        let a = self.stack.pop();
                        self.stack.push(op(a, b));
                    } else {
                        panic!("Unsupported binary operation");
                    }
                }
                OpCode::Negate => {
                    if !Value::is_number(self.stack.peek_at(0)) {
                        self.runtime_error("Operand must be a number.");
                        return InterpretResult::RuntimeError;
                    }
                    let value = self.stack.pop();
                    self.stack.push(Value::from(-value.as_number()));
                }
                OpCode::Not => {
                    let value = self.stack.pop();
                    self.stack.push(Value::from(value.is_falsey()));
                }
                OpCode::Equal => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    self.stack.push(Value::from(a == b));
                }
                OpCode::False => self.stack.push(Value::from(false)),
                OpCode::True => self.stack.push(Value::from(true)),
                OpCode::Nil => self.stack.push(Value::from(())),
                OpCode::Return => {
                    let value = self.stack.pop();
                    println!("{}", value);
                    return InterpretResult::Ok;
                }
                OpCode::Unknown => panic!("Something went wrong running the bytecode"),
            }
        }
    }

    pub fn free(&mut self) {}

    fn runtime_error(&mut self, message: &str) {
        eprintln!("{message}");

        let instruction = unsafe { self.ip.offset_from_unsigned((*self.chunk).code.data) - 1 };
        let line = unsafe { (&(*self.chunk).lines)[instruction] };
        eprintln!("[line {line}] in script");
        self.reset_stack();
    }

    fn reset_stack(&mut self) {
        self.stack = Stack::default();
    }
}

#[derive(Debug, PartialEq)]
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

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use super::*;

    #[test]
    fn test_interpret() {
        let mut vm = VM::new();
        let source = CString::new("1 > 5").expect("Input doesn't contain null bytes");

        let result = vm.interpret(source.as_bytes_with_nul().as_ptr() as *const AsciiChar);
        assert_eq!(result, InterpretResult::Ok);
    }

    #[test]
    fn test_interpret_compile_error() {
        let mut vm = VM::new();
        let source = CString::new("1>").expect("Input doesn't contain null bytes");
        let result = vm.interpret(source.as_bytes_with_nul().as_ptr() as *const AsciiChar);
        assert_eq!(result, InterpretResult::CompileError);
    }

    #[test]
    fn test_interpret_runtime_error() {
        let mut vm = VM::new();
        let source = CString::new("true + false").expect("Input doesn't contain null bytes");
        let result = vm.interpret(source.as_bytes_with_nul().as_ptr() as *const AsciiChar);
        assert_eq!(result, InterpretResult::RuntimeError);
    }
}
