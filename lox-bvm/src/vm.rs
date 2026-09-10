use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    AsciiChar, COMPILER_INSTANCE,
    collections::{dynarray::DynArray, hashtable::HashTable, stack::Stack},
    compiler::{Compiler, FunctionType, Parser},
    scanner::Scanner,
    types::value::string::copy_string,
    types::{
        opcode::OpCode,
        value::{
            Value,
            class::{ObjBoundMethod, ObjClass, ObjInstance},
            closure::ObjClosure,
            native::{NativeFn, ObjNative},
            obj::{Obj, free_object},
            string::ObjString,
            upvalue::ObjUpvalue,
        },
    },
};

pub const FRAMES_MAX: usize = 64;

pub struct VM {
    pub frames: [CallFrame; FRAMES_MAX],
    pub frame_count: u8,

    pub stack: Stack<Value>,
    pub open_upvalues: *mut ObjUpvalue,
    pub objects: *mut Obj,
    pub strings: HashTable,
    pub globals: HashTable,

    pub gray_stack: DynArray<*mut Obj>,

    pub bytes_allocated: usize,
    pub next_gc: usize,

    pub init_string: *mut ObjString,

    #[cfg(test)]
    output: Vec<String>,
}

impl VM {
    pub fn new() -> Self {
        let call_frame = CallFrame {
            closure: std::ptr::null_mut(),
            ip: std::ptr::null_mut(),
            slots: std::ptr::null_mut(),
        };

        let mut vm = VM {
            frames: [(); FRAMES_MAX].map(|_| call_frame.clone()),
            frame_count: 0,
            stack: Stack::default(),
            open_upvalues: std::ptr::null_mut(),
            objects: std::ptr::null_mut(),
            strings: HashTable::new(),
            globals: HashTable::new(),

            gray_stack: DynArray::default(),

            bytes_allocated: 0,
            next_gc: 1024 * 1024,

            init_string: std::ptr::null_mut(),

            #[cfg(test)]
            output: Vec::new(),
        };

        vm.init_string = copy_string(
            "init".as_ptr() as *const AsciiChar,
            4,
            &mut vm.objects,
            &mut vm.strings,
        );

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

        unsafe {
            COMPILER_INSTANCE = compiler as *mut Compiler;
        }

        let parser = &mut Parser::new(scanner, compiler, &mut self.objects, &mut self.strings);

        let function = parser.compile();

        unsafe {
            COMPILER_INSTANCE = compiler as *mut Compiler;
        }

        if function.is_null() {
            return InterpretResult::CompileError;
        }

        self.stack.push(Value::from(function));

        let closure = ObjClosure::new(&mut self.objects, function);
        self.stack.pop();
        self.stack.push(Value::from(closure));

        self.call(closure, 0);

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
                let offset = unsafe {
                    frame
                        .ip
                        .offset_from((*(*frame.closure).function).chunk.code.data)
                } as usize;
                let _ =
                    disassemble_instruction(unsafe { &(*(*frame.closure).function).chunk }, offset);
            }

            let instruction = OpCode::from(unsafe { *frame.ip });
            frame.ip = unsafe { frame.ip.add(1) };

            match instruction {
                OpCode::Constant => {
                    let position = unsafe { *frame.ip } as usize;
                    frame.ip = unsafe { frame.ip.add(1) };

                    let value = &unsafe { &(*(*frame.closure).function).chunk }.values[position];
                    self.stack.push(value.clone());
                }
                OpCode::Add => {
                    let b = self.stack.peek(0).clone();
                    let a = self.stack.peek(1).clone();
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
                    self.stack.pop();
                    self.stack.pop();
                    match result {
                        Ok(v) => self.stack.push(v),
                        Err(e) => {
                            self.runtime_error(&e.to_string());
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
                                self.runtime_error(&e.to_string());
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
                    close_upvalues(&mut self.open_upvalues, frame.slots);
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

                    let name =
                        unsafe { &(*(*frame.closure).function).chunk }.values[position].as_string();
                    self.globals.set(name, self.stack.peek(0).clone());
                    let _ = self.stack.pop();
                }
                OpCode::GetGlobal => {
                    let position = unsafe { *frame.ip } as usize;
                    frame.ip = unsafe { frame.ip.add(1) };

                    let name =
                        unsafe { &(*(*frame.closure).function).chunk }.values[position].as_string();
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

                    let name =
                        unsafe { &(*(*frame.closure).function).chunk }.values[position].as_string();
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
                OpCode::Closure => {
                    let position = unsafe { *frame.ip } as usize;
                    frame.ip = unsafe { frame.ip.add(1) };

                    let function = unsafe { &(*(*frame.closure).function).chunk }.values[position]
                        .as_function();
                    let closure = ObjClosure::new(&mut self.objects, function);
                    self.stack.push(Value::from(closure));

                    for i in 0..unsafe { (*function).upvalue_count } {
                        let is_local = unsafe { *frame.ip } != 0;
                        frame.ip = unsafe { frame.ip.add(1) };

                        let index = unsafe { *frame.ip } as usize;
                        frame.ip = unsafe { frame.ip.add(1) };

                        if is_local {
                            unsafe {
                                (*closure).upvalues.add(i).write(capture_upvalue(
                                    &mut self.open_upvalues,
                                    &mut self.objects,
                                    frame.slots.add(index),
                                ));
                            }
                        } else {
                            unsafe {
                                (*closure)
                                    .upvalues
                                    .add(i)
                                    .write(*(*frame.closure).upvalues.add(index));
                            }
                        }
                    }
                }
                OpCode::GetUpvalue => {
                    let slot = unsafe { *frame.ip } as usize;
                    frame.ip = unsafe { frame.ip.add(1) };

                    let upvalue = unsafe { (*(*(*frame.closure).upvalues.add(slot))).location };
                    self.stack.push(unsafe { (*upvalue).clone() });
                }
                OpCode::SetUpvalue => {
                    let slot = unsafe { *frame.ip } as usize;
                    frame.ip = unsafe { frame.ip.add(1) };

                    let upvalue = unsafe { (*(*(*frame.closure).upvalues.add(slot))).location };
                    unsafe { *upvalue = self.stack.peek(0).clone() };
                }
                OpCode::CloseUpvalue => {
                    let last = unsafe { self.stack.as_mut_ptr().add(self.stack.len() - 1) };
                    close_upvalues(&mut self.open_upvalues, last);
                    self.stack.pop();
                }
                OpCode::Class => {
                    let position = unsafe { *frame.ip } as usize;
                    frame.ip = unsafe { frame.ip.add(1) };

                    let name =
                        unsafe { &(*(*frame.closure).function).chunk }.values[position].as_string();
                    let class = ObjClass::new(&mut self.objects, name);
                    self.stack.push(Value::from(class));
                }
                OpCode::GetProperty => {
                    if !self.stack.peek(0).is_instance() {
                        self.runtime_error("Only instances have properties.");
                        return InterpretResult::RuntimeError;
                    }

                    let instance = self.stack.peek(0).as_instance();

                    let name_position = unsafe { *frame.ip } as usize;
                    frame.ip = unsafe { frame.ip.add(1) };

                    let name = unsafe { &(*(*frame.closure).function).chunk }.values[name_position]
                        .as_string();

                    let value = unsafe { (*instance).fields.get(name) };
                    if let Some(v) = value {
                        self.stack.pop();
                        self.stack.push(unsafe { (*v).clone() });
                    } else {
                        if let Err(message) = bind_method(
                            &mut self.objects,
                            &mut self.stack,
                            unsafe { (*instance).class },
                            name,
                        ) {
                            self.runtime_error(&message);
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::SetProperty => {
                    if !self.stack.peek(1).is_instance() {
                        self.runtime_error("Only instances have fields.");
                        return InterpretResult::RuntimeError;
                    }

                    let instance = self.stack.peek(1).as_instance();

                    let name_position = unsafe { *frame.ip } as usize;
                    frame.ip = unsafe { frame.ip.add(1) };

                    let name = unsafe { &(*(*frame.closure).function).chunk }.values[name_position]
                        .as_string();

                    let value = self.stack.peek(0).clone();
                    unsafe { (*instance).fields.set(name, value.clone()) };
                    self.stack.pop();
                    self.stack.pop();
                    self.stack.push(value);
                }
                OpCode::Method => {
                    let name_position = unsafe { *frame.ip } as usize;
                    frame.ip = unsafe { frame.ip.add(1) };

                    let name = unsafe { &(*(*frame.closure).function).chunk }.values[name_position]
                        .as_string();

                    let class = self.stack.peek(1).as_class();
                    let method = self.stack.peek(0).clone();
                    unsafe { (*class).methods.set(name, method) };
                    self.stack.pop();
                }
                OpCode::Invoke => {
                    let method_position = unsafe { *frame.ip } as usize;
                    frame.ip = unsafe { frame.ip.add(1) };
                    let method = unsafe { &(*(*frame.closure).function).chunk }.values
                        [method_position]
                        .as_string();

                    let arg_count = unsafe { *frame.ip } as usize;
                    frame.ip = unsafe { frame.ip.add(1) };

                    if !self.invoke(method, arg_count) {
                        return InterpretResult::RuntimeError;
                    }

                    frame = &mut self.frames[self.frame_count as usize - 1];
                }
                OpCode::Inherit => {
                    let superclass = self.stack.peek(1).clone();
                    if !superclass.is_class() {
                        self.runtime_error("Superclass must be a class.");
                        return InterpretResult::RuntimeError;
                    }

                    let subclass = self.stack.peek(0).as_class();
                    unsafe {
                        (*subclass)
                            .methods
                            .add_all(&(*superclass.as_class()).methods);
                    };
                    self.stack.pop();
                }
                OpCode::GetSuper => {
                    let name_position = unsafe { *frame.ip } as usize;
                    frame.ip = unsafe { frame.ip.add(1) };

                    let name = unsafe { &(*(*frame.closure).function).chunk }.values[name_position]
                        .as_string();

                    let superclass = self.stack.pop().as_class();

                    if bind_method(&mut self.objects, &mut self.stack, superclass, name).is_err() {
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::SuperInvoke => {
                    let method_position = unsafe { *frame.ip } as usize;
                    frame.ip = unsafe { frame.ip.add(1) };
                    let method = unsafe { &(*(*frame.closure).function).chunk }.values
                        [method_position]
                        .as_string();

                    let arg_count = unsafe { *frame.ip } as usize;
                    frame.ip = unsafe { frame.ip.add(1) };

                    let superclass = self.stack.pop().as_class();

                    if !self.invoke_from_class(superclass, method, arg_count) {
                        return InterpretResult::RuntimeError;
                    }

                    frame = &mut self.frames[self.frame_count as usize - 1];
                }
                OpCode::Unknown => panic!("Something went wrong running the bytecode"),
            }
        }
    }

    fn invoke(&mut self, name: *mut ObjString, arg_count: usize) -> bool {
        let receiver = self.stack.peek(arg_count).clone();
        if !receiver.is_instance() {
            self.runtime_error("Only instances have methods.");
            return false;
        }

        let instance = receiver.as_instance();
        if let Some(value) = unsafe { (*instance).fields.get(name) } {
            let stack_len = self.stack.len();
            self.stack[stack_len - arg_count - 1] = unsafe { (*value).clone() };
            return self.call_value(unsafe { (*value).clone() }, arg_count);
        }

        self.invoke_from_class(unsafe { (*instance).class }, name, arg_count)
    }

    fn invoke_from_class(
        &mut self,
        class: *mut ObjClass,
        name: *mut ObjString,
        arg_count: usize,
    ) -> bool {
        let method = unsafe { (*class).methods.get(name) };
        if let Some(m) = method {
            return self.call(unsafe { (*m).as_closure() }, arg_count);
        }

        self.runtime_error(&format!("Undefined property '{}'.", Value::from(name)));
        false
    }

    fn call_value(&mut self, callee: Value, arg_count: usize) -> bool {
        if callee.is_function() {
            unreachable!("eventhing is wrapper up in a closure")
        } else if callee.is_class() {
            let class = callee.as_class();
            let instance = Value::from(ObjInstance::new(&mut self.objects, class));
            let stack_len = self.stack.len();
            unsafe {
                *self.stack.as_mut_ptr().add(stack_len - arg_count - 1) = instance.clone();
            }

            if let Some(initializer) = unsafe { (*class).methods.get(self.init_string) } {
                return self.call(unsafe { (*initializer).as_closure() }, arg_count);
            } else if arg_count != 0 {
                self.runtime_error(&format!("Expected 0 arguments but got {}.", arg_count));
                return false;
            }

            return true;
        } else if callee.is_native() {
            let native = callee.as_native();
            let stack_base = self.stack.len() - arg_count;
            let args = unsafe { self.stack.as_mut_ptr().add(stack_base) };
            let result = unsafe { (*native).function }(arg_count, args);
            self.stack.truncate(stack_base - 1);
            self.stack.push(result);
            return true;
        } else if callee.is_closure() {
            let closure = callee.as_closure();
            return self.call(closure, arg_count);
        } else if callee.is_bound_method() {
            let bound_method = callee.as_bound_method();
            let method = unsafe { (*bound_method).method };
            let stack_len = self.stack.len();
            self.stack[stack_len - arg_count - 1] = unsafe { (*bound_method).receiver.clone() };
            return self.call(method, arg_count);
        }

        self.runtime_error("Can only call functions and classes.");
        false
    }

    fn call(&mut self, closure: *mut ObjClosure, arg_count: usize) -> bool {
        if arg_count != unsafe { (*(*closure).function).arity } {
            self.runtime_error(&format!(
                "Expected {} arguments but got {}.",
                unsafe { (*(*closure).function).arity },
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

        frame.closure = closure;
        frame.ip = unsafe { (*(*closure).function).chunk.code.data };
        let stack_base = self.stack.len() - arg_count - 1;
        frame.slots = unsafe { self.stack.as_mut_ptr().add(stack_base) };

        true
    }

    #[cfg(test)]
    fn output(&self) -> &[String] {
        &self.output
    }

    pub fn free(&mut self) {
        self.init_string = std::ptr::null_mut();

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
            let function = unsafe { (*frame.closure).function };
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
        self.frame_count = 0;
        self.open_upvalues = std::ptr::null_mut();
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

fn bind_method(
    objects: &mut *mut Obj,
    stack: &mut Stack<Value>,
    class: *mut ObjClass,
    name: *mut ObjString,
) -> Result<(), String> {
    let method = unsafe { (*class).methods.get(name) };
    if let Some(m) = method {
        let bound_method =
            ObjBoundMethod::new(objects, stack.peek(0).clone(), unsafe { (*m).as_closure() });
        stack.pop();
        stack.push(Value::from(bound_method));
        Ok(())
    } else {
        Err(format!("Undefined property '{}'.", Value::from(name)))
    }
}

fn clock_native(_: usize, _: *mut Value) -> Value {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
    Value::from(elapsed)
}

fn capture_upvalue(
    upvalue: &mut *mut ObjUpvalue,
    objects: *mut *mut Obj,
    local: *mut Value,
) -> *mut ObjUpvalue {
    let mut prev_upvalue = std::ptr::null_mut();
    let mut curr_upvalue = *upvalue;
    while !curr_upvalue.is_null() && unsafe { (*curr_upvalue).location } > local {
        prev_upvalue = curr_upvalue;
        curr_upvalue = unsafe { (*curr_upvalue).next };
    }

    if !curr_upvalue.is_null() && unsafe { (*curr_upvalue).location } == local {
        return curr_upvalue;
    }

    let created_upvalue = ObjUpvalue::new(objects, local);
    unsafe { (*created_upvalue).next = curr_upvalue };

    if prev_upvalue.is_null() {
        *upvalue = created_upvalue;
    } else {
        unsafe { (*prev_upvalue).next = created_upvalue };
    }

    created_upvalue
}

fn close_upvalues(upvalue: &mut *mut ObjUpvalue, last: *mut Value) {
    while !(*upvalue).is_null() && unsafe { (**upvalue).location } >= last {
        let curr = *upvalue;
        unsafe { (*curr).closed = (*(*curr).location).clone() };
        unsafe { (*curr).location = std::ptr::addr_of_mut!((*curr).closed) };
        *upvalue = unsafe { (*curr).next };
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

#[derive(Clone)]
pub struct CallFrame {
    pub closure: *mut ObjClosure,
    pub ip: *mut u8,
    pub slots: *mut Value,
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

    #[test]
    fn print_class_name() {
        let mut vm = VM::new();
        let source = CString::new("class Brioche {} print Brioche;")
            .expect("Input doesn't contain null bytes");
        let result = vm.interpret(source.as_bytes_with_nul().as_ptr() as *const AsciiChar);

        assert_eq!(result, InterpretResult::Ok);
        assert_eq!(vm.output(), &["Brioche".to_string()]);
    }
}
