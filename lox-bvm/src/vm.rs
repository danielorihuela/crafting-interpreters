use crate::{
    AsciiChar,
    collections::{hashtable::HashTable, stack::Stack},
    compiler::{Compiler, Parser},
    scanner::Scanner,
    types::{
        chunk::Chunk,
        opcode::OpCode,
        value::{
            Value,
            obj::{Obj, free_object},
        },
    },
};

pub struct VM {
    chunk: *mut Chunk,
    ip: *mut u8,

    stack: Stack<Value>,
    objects: *mut Obj,
    strings: HashTable,
    globals: HashTable,

    #[cfg(test)]
    output: Vec<String>,
}

impl VM {
    pub fn new() -> Self {
        VM {
            chunk: std::ptr::null_mut(),
            ip: std::ptr::null_mut(),
            stack: Stack::default(),
            objects: std::ptr::null_mut(),
            strings: HashTable::new(),
            globals: HashTable::new(),

            #[cfg(test)]
            output: Vec::new(),
        }
    }

    pub fn interpret(&mut self, source: *const AsciiChar) -> InterpretResult {
        let chunk = &mut Chunk::default();

        let compiler = &mut Compiler::new();

        let scanner = &mut Scanner::new(source);
        let parser = &mut Parser::new(scanner, compiler, &mut self.objects, &mut self.strings);
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
                OpCode::Add => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    let result = if a.is_string() && b.is_string() {
                        Ok(Value::from(unsafe {
                            (*a.as_string()).add(
                                b.as_string(),
                                &mut self.objects,
                                &mut self.strings,
                            )
                        }))
                    } else {
                        a + b
                    };
                    match result {
                        Ok(v) => self.stack.push(v),
                        Err(e) => {
                            self.runtime_error(&e);
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::Subtract
                | OpCode::Multiply
                | OpCode::Divide
                | OpCode::Greater
                | OpCode::Less => {
                    if let Some(op) = instruction.maybe_binary_op() {
                        let b = self.stack.pop();
                        let a = self.stack.pop();
                        match op(a, b) {
                            Ok(result) => self.stack.push(result),
                            Err(e) => {
                                self.runtime_error(&e);
                                return InterpretResult::RuntimeError;
                            }
                        }
                    } else {
                        panic!("Unsupported binary operation");
                    }
                }
                OpCode::Negate => {
                    if !self.stack.peek(0).is_number() {
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
                OpCode::Return => return InterpretResult::Ok,
                OpCode::Print => {
                    let value = self.stack.pop();
                    #[cfg(test)]
                    self.output.push(value.to_string());
                    println!("{}", value);
                }
                OpCode::Pop => {
                    self.stack.pop();
                }
                OpCode::DefineGlobal => {
                    let position = unsafe { *self.ip } as usize;
                    self.ip = unsafe { self.ip.add(1) };

                    let name = unsafe { &*self.chunk }.values[position].as_string();
                    self.globals.set(name, self.stack.peek(0).clone());
                    let _ = self.stack.pop();
                }
                OpCode::GetGlobal => {
                    let position = unsafe { *self.ip } as usize;
                    self.ip = unsafe { self.ip.add(1) };

                    let name = unsafe { &*self.chunk }.values[position].as_string();
                    match self.globals.get(name) {
                        Some(value) => self.stack.push(unsafe { (*value).clone() }),
                        None => {
                            self.runtime_error(&format!(
                                "Undefined variable '{}'.",
                                Value::from(name).to_string()
                            ));
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::SetGlobal => {
                    let position = unsafe { *self.ip } as usize;
                    self.ip = unsafe { self.ip.add(1) };

                    let name = unsafe { &*self.chunk }.values[position].as_string();
                    if self.globals.set(name, self.stack.peek(0).clone()) {
                        self.globals.delete(name);
                        self.runtime_error(&format!(
                            "Undefined variable '{}'.",
                            Value::from(name).to_string()
                        ));
                        return InterpretResult::RuntimeError;
                    };
                }
                OpCode::GetLocal => {
                    let slot = unsafe { *self.ip } as usize;
                    self.ip = unsafe { self.ip.add(1) };

                    self.stack.push(self.stack[slot].clone());
                }
                OpCode::SetLocal => {
                    let slot = unsafe { *self.ip } as usize;
                    self.ip = unsafe { self.ip.add(1) };

                    self.stack[slot] = self.stack.peek(0).clone();
                }
                OpCode::Unknown => panic!("Something went wrong running the bytecode"),
            }
        }
    }

    #[cfg(test)]
    fn output(&self) -> &[String] {
        &self.output
    }

    pub fn free(&mut self) {
        let mut object = self.objects;
        while !object.is_null() {
            let next = unsafe { (*object).next.0 };
            unsafe { free_object(object) };
            object = next;
        }

        self.strings.free();
        self.globals.free();
    }

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
    fn test_interpret_compile_error() {
        let mut vm = VM::new();
        let source = CString::new("print 1>;").expect("Input doesn't contain null bytes");
        let result = vm.interpret(source.as_bytes_with_nul().as_ptr() as *const AsciiChar);
        assert_eq!(result, InterpretResult::CompileError);
    }

    #[test]
    fn test_interpret_runtime_error() {
        let mut vm = VM::new();
        let source = CString::new("print true + false;").expect("Input doesn't contain null bytes");
        let result = vm.interpret(source.as_bytes_with_nul().as_ptr() as *const AsciiChar);
        assert_eq!(result, InterpretResult::RuntimeError);
    }

    macro_rules! test_interpret {
        ($($name:ident: $data:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (source, expected) = $data;

                    let mut vm = VM::new();
                    let source = CString::new(format!("print {};", source)).expect("Input doesn't contain null bytes");
                    let result = vm.interpret(source.as_bytes_with_nul().as_ptr() as *const AsciiChar);
                    assert_eq!(result, InterpretResult::Ok);
                    assert_eq!(vm.output(), &[expected.to_string()]);

                    vm.free();
                }
            )*
        };
    }

    test_interpret! {
        not: ("!false", "true"),

        unary_negate: ("-123", "-123"),
        addition_numbers: ("1 + 2", "3"),
        subtraction: ("5 - 3", "2"),
        multiplication: ("4 * 2", "8"),
        division: ("10 / 2", "5"),
        precedence: ("1 + 2 * 3", "7"),
        parentheses: ("(1 + 2) * 3", "9"),
        equal_numbers: ("1 == 1", "true"),
        equal_numbers_2: ("1 == 2", "false"),
        not_equal_numbers: ("1 != 1", "false"),
        not_equal_numbers_2: ("1 != 2", "true"),
        greater_numbers: ("3 > 2", "true"),
        greater_numbers_2: ("2 > 3", "false"),
        greater_equal_numbers: ("3 >= 3", "true"),
        greater_equal_numbers_2: ("2 >= 3", "false"),
        less_numbers: ("2 < 3", "true"),
        less_numbers_2: ("3 < 2", "false"),
        less_equal_numbers: ("3 <= 3", "true"),
        less_equal_numbers_2: ("3 <= 2", "false"),

        addition_strings: ("\"Hello, \" + \"world!\"", "Hello, world!"),
        equal_strings: ("\"hello\" == \"hello\"", "true"),
        equal_strings_2: ("\"hello\" == \"world\"", "false"),
        not_equal_strings: ("\"hello\" != \"hello\"", "false"),
        not_equal_strings_2: ("\"hello\" != \"world\"", "true"),

        equal_different_types: ("1 == \"1\"", "false"),
    }
}
