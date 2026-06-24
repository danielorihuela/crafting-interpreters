use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    AsciiChar,
    collections::{hashtable::HashTable, stack::Stack},
    compiler::{Compiler, FunctionType, Parser},
    scanner::Scanner,
    types::{
        opcode::OpCode,
        value::{
            Value,
            function::ObjFunction,
            native::{NativeFn, ObjNative},
            obj::{Obj, free_object},
            string::ObjString,
        },
    },
};

pub const FRAMES_MAX: usize = 64;

pub struct VM {
    frames: [CallFrame; FRAMES_MAX],
    frame_count: u8,

    stack: Stack<Value>,
    objects: *mut Obj,
    strings: HashTable,
    globals: HashTable,

    #[cfg(test)]
    output: Vec<String>,
}

impl VM {
    pub fn new() -> Self {
        let call_frame = CallFrame {
            function: std::ptr::null_mut(),
            ip: std::ptr::null_mut(),
            slots: std::ptr::null_mut(),
        };
        let mut vm = VM {
            frames: [(); FRAMES_MAX].map(|_| call_frame.clone()),
            frame_count: 0,
            stack: Stack::default(),
            objects: std::ptr::null_mut(),
            strings: HashTable::new(),
            globals: HashTable::new(),

            #[cfg(test)]
            output: Vec::new(),
        };

        vm.define_native(
            "clock".as_ptr() as *mut AsciiChar,
            "clock".len(),
            clock_native,
        );

        vm
    }

    pub fn interpret(&mut self, source: *const AsciiChar) -> InterpretResult {
        let scanner = &mut Scanner::new(source);
        let compiler = &mut Compiler::new(
            FunctionType::Script,
            &mut self.objects,
            &mut self.strings,
            std::ptr::null_mut(),
            std::ptr::null(),
            0,
        );
        let parser = &mut Parser::new(scanner, compiler, &mut self.objects, &mut self.strings);

        let function = parser.compile();
        if function.is_null() {
            return InterpretResult::CompileError;
        }

        self.stack.push(Value::from(function));
        self.call(function, 0);

        self.run()
    }

    fn run(&mut self) -> InterpretResult {
        #[cfg(debug_assertions)]
        {
            println!("\n=== Running bytecode ===");
        }

        let mut frame = &mut self.frames[self.frame_count as usize - 1];

        loop {
            #[cfg(debug_assertions)]
            {
                use crate::collections::stack::debug::show_stack;
                show_stack(&self.stack);

                use crate::types::chunk::debug::disassemble_instruction;
                let offset =
                    unsafe { frame.ip.offset_from((*frame.function).chunk.code.data) } as usize;
                let _ = disassemble_instruction(unsafe { &(*frame.function).chunk }, offset);
            }

            let instruction = OpCode::from(unsafe { *frame.ip });
            frame.ip = unsafe { frame.ip.add(1) };

            match instruction {
                OpCode::Constant => {
                    let position = unsafe { *frame.ip } as usize;
                    frame.ip = unsafe { frame.ip.add(1) };

                    let value = &unsafe { &(*frame.function).chunk }.values[position];
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
                OpCode::Return => {
                    let result = self.stack.pop();
                    self.frame_count -= 1;
                    if self.frame_count == 0 {
                        self.stack.pop();
                        return InterpretResult::Ok;
                    }

                    unsafe { self.stack.truncate_to_ptr(frame.slots) };
                    self.stack.push(result);
                    frame = &mut self.frames[self.frame_count as usize - 1];
                }
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
                    let position = unsafe { *frame.ip } as usize;
                    frame.ip = unsafe { frame.ip.add(1) };

                    let name = unsafe { &(*frame.function).chunk }.values[position].as_string();
                    self.globals.set(name, self.stack.peek(0).clone());
                    let _ = self.stack.pop();
                }
                OpCode::GetGlobal => {
                    let position = unsafe { *frame.ip } as usize;
                    frame.ip = unsafe { frame.ip.add(1) };

                    let name = unsafe { &(*frame.function).chunk }.values[position].as_string();
                    match self.globals.get(name) {
                        Some(value) => self.stack.push(unsafe { (*value).clone() }),
                        None => {
                            self.runtime_error(&format!(
                                "Undefined variable '{}'.",
                                Value::from(name)
                            ));
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::SetGlobal => {
                    let position = unsafe { *frame.ip } as usize;
                    frame.ip = unsafe { frame.ip.add(1) };

                    let name = unsafe { &(*frame.function).chunk }.values[position].as_string();
                    if self.globals.set(name, self.stack.peek(0).clone()) {
                        self.globals.delete(name);
                        self.runtime_error(&format!("Undefined variable '{}'.", Value::from(name)));
                        return InterpretResult::RuntimeError;
                    };
                }
                OpCode::GetLocal => {
                    let slot = unsafe { *frame.ip } as usize;
                    frame.ip = unsafe { frame.ip.add(1) };

                    self.stack.push(unsafe { (*frame.slots.add(slot)).clone() });
                }
                OpCode::SetLocal => {
                    let slot = unsafe { *frame.ip } as usize;
                    frame.ip = unsafe { frame.ip.add(1) };

                    unsafe { *frame.slots.add(slot) = self.stack.peek(0).clone() };
                }
                OpCode::JumpIfFalse => {
                    let offset_0 = unsafe { *frame.ip } as usize;
                    let offset_1 = unsafe { *frame.ip.add(1) } as usize;
                    frame.ip = unsafe { frame.ip.add(2) };

                    let offset = (offset_0 << 8) | offset_1;

                    if self.stack.peek(0).is_falsey() {
                        frame.ip = unsafe { frame.ip.add(offset) };
                    }
                }
                OpCode::Jump => {
                    let offset_0 = unsafe { *frame.ip } as usize;
                    let offset_1 = unsafe { *frame.ip.add(1) } as usize;
                    frame.ip = unsafe { frame.ip.add(2) };

                    let offset = (offset_0 << 8) | offset_1;

                    frame.ip = unsafe { frame.ip.add(offset) };
                }
                OpCode::Loop => {
                    let offset_0 = unsafe { *frame.ip } as usize;
                    let offset_1 = unsafe { *frame.ip.add(1) } as usize;
                    frame.ip = unsafe { frame.ip.add(2) };

                    let offset = (offset_0 << 8) | offset_1;

                    frame.ip = unsafe { frame.ip.sub(offset) };
                }
                OpCode::Call => {
                    let arg_count = unsafe { *frame.ip } as usize;
                    frame.ip = unsafe { frame.ip.add(1) };

                    let callee = self.stack.peek(arg_count).clone();
                    if !self.call_value(callee, arg_count) {
                        return InterpretResult::RuntimeError;
                    }
                    frame = &mut self.frames[self.frame_count as usize - 1];
                }
                OpCode::Unknown => panic!("Something went wrong running the bytecode"),
            }
        }
    }

    fn call_value(&mut self, callee: Value, arg_count: usize) -> bool {
        if callee.is_function() {
            let function = callee.as_function();
            return self.call(function, arg_count);
        } else if callee.is_native() {
            let native = callee.as_native();
            let stack_base = self.stack.len() - arg_count;
            let args = unsafe { self.stack.as_mut_ptr().add(stack_base) };
            let result = unsafe { (*native).function }(arg_count, args);
            self.stack.truncate(stack_base - 1);
            self.stack.push(result);
            return true;
        }

        self.runtime_error("Can only call functions and classes.");
        false
    }

    fn call(&mut self, function: *mut ObjFunction, arg_count: usize) -> bool {
        if arg_count != unsafe { (*function).arity } {
            self.runtime_error(&format!(
                "Expected {} arguments but got {}.",
                unsafe { (*function).arity },
                arg_count
            ));
            return false;
        }

        if self.frame_count as usize == FRAMES_MAX {
            self.runtime_error("Stack overflow.");
            return false;
        }

        let frame = &mut self.frames[self.frame_count as usize];
        self.frame_count += 1;

        frame.function = function;
        frame.ip = unsafe { (*function).chunk.code.data };
        let stack_base = self.stack.len() - arg_count - 1;
        frame.slots = unsafe { self.stack.as_mut_ptr().add(stack_base) };

        true
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

        self.objects = std::ptr::null_mut();

        self.strings.free();
        self.globals.free();
    }

    fn runtime_error(&mut self, message: &str) {
        eprintln!("{message}");

        for i in (0..self.frame_count).rev() {
            let frame = &self.frames[i as usize];
            let function = frame.function;
            let instruction =
                unsafe { frame.ip.offset_from_unsigned((*function).chunk.code.data) - 1 };
            let line = unsafe { (&(*function).chunk.lines)[instruction] };
            eprintln!(
                "[line {line}] in {}",
                if unsafe { (*function).name }.is_null() {
                    "script".to_string()
                } else {
                    unsafe { Value::from((*function).name).to_string() }
                }
            );
        }

        self.reset_stack();
    }

    fn reset_stack(&mut self) {
        self.stack = Stack::default();
    }

    fn define_native(&mut self, name: *mut AsciiChar, length: usize, function: NativeFn) {
        let name = Value::from(ObjString::new(
            name,
            length,
            &mut self.objects,
            &mut self.strings,
        ));
        self.stack.push(name);

        let native = Value::from(ObjNative::new(function, &mut self.objects));
        self.stack.push(native);

        self.globals
            .set(self.stack.peek(1).as_string(), self.stack.peek(0).clone());

        self.stack.pop();
        self.stack.pop();
    }
}

fn clock_native(_: usize, _: *mut Value) -> Value {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
    Value::from(elapsed)
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

#[derive(Clone)]
struct CallFrame {
    function: *mut ObjFunction,
    ip: *mut u8,
    slots: *mut Value,
}

impl Drop for VM {
    fn drop(&mut self) {
        self.free();
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
