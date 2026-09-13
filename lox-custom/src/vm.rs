use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    DEBUG_LOG_GC,
    collections::{dynarray::DynArray, hashtable::HashTable, stack::Stack},
    compiler::{Compiler, FunctionType, Parser},
    memory::gc::{GcCollector, GcContext},
    scanner::Scanner,
    types::{
        opcode::OpCode,
        value::{
            Value,
            class::{ObjBoundMethod, ObjClass, ObjInstance},
            closure::ObjClosure,
            function::ObjFunction,
            native::{NativeFn, ObjNative},
            obj::{Obj, ObjType, free_object},
            string::{ObjString, copy_string},
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
    pub compiler: *mut Compiler,
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
            compiler: std::ptr::null_mut(),
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

        vm.init_string = copy_string("init", &mut vm.objects, &mut vm.strings, &mut vm);

        vm.define_native("clock", clock_native);

        vm
    }

    pub fn interpret(&mut self, source: &str) -> InterpretResult {
        let scanner = &mut Scanner::new(source);
        let mut compiler = Compiler::new(
            FunctionType::Script,
            &mut self.objects,
            &mut self.strings,
            std::ptr::null_mut(),
            "",
            self,
        );

        self.compiler = &mut compiler;

        let parser = &mut Parser::new(
            scanner,
            self.compiler,
            &mut self.objects,
            &mut self.strings,
            &mut self.stack,
            self,
        );

        let function = parser.compile();

        if function.is_null() {
            return InterpretResult::CompileError;
        }

        self.stack.push(Value::from(function));

        let closure = ObjClosure::new(&mut self.objects, function, self);
        self.stack.pop();
        self.stack.push(Value::from(closure));

        self.call(closure, 0);

        self.run()
    }

    #[allow(unused_unsafe)]
    fn run(&mut self) -> InterpretResult {
        #[cfg(debug_assertions)]
        {
            println!("\n=== Running bytecode ===");
        }

        let mut frame: *mut CallFrame = &mut self.frames[self.frame_count as usize - 1];

        loop {
            #[cfg(debug_assertions)]
            {
                use crate::collections::stack::debug::show_stack;
                show_stack(&self.stack);

                use crate::types::chunk::debug::disassemble_instruction;
                let offset = unsafe {
                    (*frame)
                        .ip
                        .offset_from((*(*(*frame).closure).function).chunk.code.data)
                } as usize;
                let _ = disassemble_instruction(
                    unsafe { &(*(*(*frame).closure).function).chunk },
                    offset,
                );
            }

            unsafe {
                let instruction = OpCode::from(*(*frame).ip);
                (*frame).ip = (*frame).ip.add(1);

                match instruction {
                    OpCode::Constant => {
                        let position = unsafe { *(*frame).ip } as usize;
                        (*frame).ip = unsafe { (*frame).ip.add(1) };

                        let value =
                            &unsafe { &(*(*(*frame).closure).function).chunk }.values[position];
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
                                    self,
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
                        self.stack.push(Value::Number(-value.as_number()));
                    }
                    OpCode::Not => {
                        let value = self.stack.pop();
                        self.stack.push(Value::Bool(value.is_falsey()));
                    }
                    OpCode::Equal => {
                        let b = self.stack.pop();
                        let a = self.stack.pop();
                        self.stack.push(Value::Bool(a == b));
                    }
                    OpCode::False => self.stack.push(Value::Bool(false)),
                    OpCode::True => self.stack.push(Value::Bool(true)),
                    OpCode::Nil => self.stack.push(Value::Nil),
                    OpCode::Return => {
                        let result = self.stack.pop();
                        close_upvalues(&mut self.open_upvalues, (*frame).slots);
                        self.frame_count -= 1;
                        if self.frame_count == 0 {
                            self.stack.pop();
                            return InterpretResult::Ok;
                        }

                        unsafe { self.stack.truncate_to_ptr((*frame).slots) };
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
                        let position = unsafe { *(*frame).ip } as usize;
                        (*frame).ip = unsafe { (*frame).ip.add(1) };

                        let name = unsafe { &(*(*(*frame).closure).function).chunk }.values
                            [position]
                            .as_string();
                        self.globals_set(name, self.stack.peek(0).clone());
                        let _ = self.stack.pop();
                    }
                    OpCode::GetGlobal => {
                        let position = unsafe { *(*frame).ip } as usize;
                        (*frame).ip = unsafe { (*frame).ip.add(1) };

                        let name = unsafe { &(*(*(*frame).closure).function).chunk }.values
                            [position]
                            .as_string();
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
                        let position = unsafe { *(*frame).ip } as usize;
                        (*frame).ip = unsafe { (*frame).ip.add(1) };

                        let name = unsafe { &(*(*(*frame).closure).function).chunk }.values
                            [position]
                            .as_string();
                        if self.globals_set(name, self.stack.peek(0).clone()) {
                            self.globals.delete(name);
                            self.runtime_error(&format!(
                                "Undefined variable '{}'.",
                                Value::from(name)
                            ));
                            return InterpretResult::RuntimeError;
                        };
                    }
                    OpCode::GetLocal => {
                        let slot = unsafe { *(*frame).ip } as usize;
                        (*frame).ip = unsafe { (*frame).ip.add(1) };

                        self.stack
                            .push(unsafe { (*(*frame).slots.add(slot)).clone() });
                    }
                    OpCode::SetLocal => {
                        let slot = unsafe { *(*frame).ip } as usize;
                        (*frame).ip = unsafe { (*frame).ip.add(1) };

                        unsafe { *(*frame).slots.add(slot) = self.stack.peek(0).clone() };
                    }
                    OpCode::JumpIfFalse => {
                        let offset_0 = unsafe { *(*frame).ip } as usize;
                        let offset_1 = unsafe { *(*frame).ip.add(1) } as usize;
                        (*frame).ip = unsafe { (*frame).ip.add(2) };

                        let offset = (offset_0 << 8) | offset_1;

                        if self.stack.peek(0).is_falsey() {
                            (*frame).ip = unsafe { (*frame).ip.add(offset) };
                        }
                    }
                    OpCode::Jump => {
                        let offset_0 = unsafe { *(*frame).ip } as usize;
                        let offset_1 = unsafe { *(*frame).ip.add(1) } as usize;
                        (*frame).ip = unsafe { (*frame).ip.add(2) };

                        let offset = (offset_0 << 8) | offset_1;

                        (*frame).ip = unsafe { (*frame).ip.add(offset) };
                    }
                    OpCode::Loop => {
                        let offset_0 = unsafe { *(*frame).ip } as usize;
                        let offset_1 = unsafe { *(*frame).ip.add(1) } as usize;
                        (*frame).ip = unsafe { (*frame).ip.add(2) };

                        let offset = (offset_0 << 8) | offset_1;

                        (*frame).ip = unsafe { (*frame).ip.sub(offset) };
                    }
                    OpCode::Call => {
                        let arg_count = unsafe { *(*frame).ip } as usize;
                        (*frame).ip = unsafe { (*frame).ip.add(1) };

                        let callee = self.stack.peek(arg_count).clone();
                        if !self.call_value(callee, arg_count) {
                            return InterpretResult::RuntimeError;
                        }
                        frame = &mut self.frames[self.frame_count as usize - 1];
                    }
                    OpCode::Closure => {
                        let position = unsafe { *(*frame).ip } as usize;
                        (*frame).ip = unsafe { (*frame).ip.add(1) };

                        let function = unsafe { &(*(*(*frame).closure).function).chunk }.values
                            [position]
                            .as_function();
                        let closure = ObjClosure::new(&mut self.objects, function, self);
                        self.stack.push(Value::from(closure));

                        for i in 0..unsafe { (*function).upvalue_count } {
                            let is_local = unsafe { *(*frame).ip } != 0;
                            (*frame).ip = unsafe { (*frame).ip.add(1) };

                            let index = unsafe { *(*frame).ip } as usize;
                            (*frame).ip = unsafe { (*frame).ip.add(1) };

                            if is_local {
                                unsafe {
                                    (*closure)
                                        .upvalues
                                        .add(i)
                                        .write(capture_upvalue((*frame).slots.add(index), self));
                                }
                            } else {
                                unsafe {
                                    (*closure)
                                        .upvalues
                                        .add(i)
                                        .write(*(*(*frame).closure).upvalues.add(index));
                                }
                            }
                        }
                    }
                    OpCode::GetUpvalue => {
                        let slot = unsafe { *(*frame).ip } as usize;
                        (*frame).ip = unsafe { (*frame).ip.add(1) };

                        let upvalue =
                            unsafe { (*(*(*(*frame).closure).upvalues.add(slot))).location };
                        self.stack.push(unsafe { (*upvalue).clone() });
                    }
                    OpCode::SetUpvalue => {
                        let slot = unsafe { *(*frame).ip } as usize;
                        (*frame).ip = unsafe { (*frame).ip.add(1) };

                        let upvalue =
                            unsafe { (*(*(*(*frame).closure).upvalues.add(slot))).location };
                        unsafe { *upvalue = self.stack.peek(0).clone() };
                    }
                    OpCode::CloseUpvalue => {
                        let last = unsafe { self.stack.as_mut_ptr().add(self.stack.len() - 1) };
                        close_upvalues(&mut self.open_upvalues, last);
                        self.stack.pop();
                    }
                    OpCode::Class => {
                        let position = unsafe { *(*frame).ip } as usize;
                        (*frame).ip = unsafe { (*frame).ip.add(1) };

                        let name = unsafe { &(*(*(*frame).closure).function).chunk }.values
                            [position]
                            .as_string();
                        let class = ObjClass::new(&mut self.objects, name, self);
                        self.stack.push(Value::from(class));
                    }
                    OpCode::GetProperty => {
                        if !self.stack.peek(0).is_instance() {
                            self.runtime_error("Only instances have properties.");
                            return InterpretResult::RuntimeError;
                        }

                        let instance = self.stack.peek(0).as_instance();

                        let name_position = unsafe { *(*frame).ip } as usize;
                        (*frame).ip = unsafe { (*frame).ip.add(1) };

                        let name = unsafe { &(*(*(*frame).closure).function).chunk }.values
                            [name_position]
                            .as_string();

                        let value = unsafe { (*instance).fields.get(name) };
                        if let Some(v) = value {
                            self.stack.pop();
                            self.stack.push(unsafe { (*v).clone() });
                        } else if let Err(message) = bind_method(self, (*instance).class, name) {
                            self.runtime_error(&message);
                            return InterpretResult::RuntimeError;
                        }
                    }
                    OpCode::SetProperty => {
                        if !self.stack.peek(1).is_instance() {
                            self.runtime_error("Only instances have fields.");
                            return InterpretResult::RuntimeError;
                        }

                        let instance = self.stack.peek(1).as_instance();

                        let name_position = unsafe { *(*frame).ip } as usize;
                        (*frame).ip = unsafe { (*frame).ip.add(1) };

                        let name = unsafe { &(*(*(*frame).closure).function).chunk }.values
                            [name_position]
                            .as_string();

                        let value = self.stack.peek(0).clone();
                        unsafe { (*instance).fields.set(name, value.clone(), self) };
                        self.stack.pop();
                        self.stack.pop();
                        self.stack.push(value);
                    }
                    OpCode::Method => {
                        let name_position = unsafe { *(*frame).ip } as usize;
                        (*frame).ip = unsafe { (*frame).ip.add(1) };

                        let name = unsafe { &(*(*(*frame).closure).function).chunk }.values
                            [name_position]
                            .as_string();

                        let class = self.stack.peek(1).as_class();
                        let method = self.stack.peek(0).clone();
                        unsafe { (*class).methods.set(name, method, self) };
                        self.stack.pop();
                    }
                    OpCode::Invoke => {
                        let method_position = unsafe { *(*frame).ip } as usize;
                        (*frame).ip = unsafe { (*frame).ip.add(1) };
                        let method = unsafe { &(*(*(*frame).closure).function).chunk }.values
                            [method_position]
                            .as_string();

                        let arg_count = unsafe { *(*frame).ip } as usize;
                        (*frame).ip = unsafe { (*frame).ip.add(1) };

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
                                .add_all(&(*superclass.as_class()).methods, self);
                        };
                        self.stack.pop();
                    }
                    OpCode::GetSuper => {
                        let name_position = unsafe { *(*frame).ip } as usize;
                        (*frame).ip = unsafe { (*frame).ip.add(1) };

                        let name = unsafe { &(*(*(*frame).closure).function).chunk }.values
                            [name_position]
                            .as_string();

                        let superclass = self.stack.pop().as_class();

                        if bind_method(self, superclass, name).is_err() {
                            return InterpretResult::RuntimeError;
                        }
                    }
                    OpCode::SuperInvoke => {
                        let method_position = unsafe { *(*frame).ip } as usize;
                        (*frame).ip = unsafe { (*frame).ip.add(1) };
                        let method = unsafe { &(*(*(*frame).closure).function).chunk }.values
                            [method_position]
                            .as_string();

                        let arg_count = unsafe { *(*frame).ip } as usize;
                        (*frame).ip = unsafe { (*frame).ip.add(1) };

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
            let instance = Value::from(ObjInstance::new(&mut self.objects, class, self));
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
            unsafe { free_object(object, self) };
            object = next;
        }

        self.objects = std::ptr::null_mut();

        self.strings_free();
        self.globals_free();
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

    fn define_native(&mut self, name: &str, function: NativeFn) {
        let name = Value::from(ObjString::new(
            name,
            &mut self.objects,
            &mut self.strings,
            self,
        ));
        self.stack.push(name);

        let native = Value::from(ObjNative::new(function, &mut self.objects, self));
        self.stack.push(native);

        self.globals_set(self.stack.peek(1).as_string(), self.stack.peek(0).clone());

        self.stack.pop();
        self.stack.pop();
    }

    fn globals_set(&mut self, key: *mut ObjString, value: Value) -> bool {
        let mut globals = std::mem::replace(&mut self.globals, HashTable::new());
        let is_new = globals.set(key, value, self);
        self.globals = globals;
        is_new
    }

    fn strings_free(&mut self) {
        let mut strings = std::mem::replace(&mut self.strings, HashTable::new());
        strings.free(self);
        self.strings = strings;
    }

    fn globals_free(&mut self) {
        let mut globals = std::mem::replace(&mut self.globals, HashTable::new());
        globals.free(self);
        self.globals = globals;
    }

    fn gray_stack_push_no_gc(&mut self, object: *mut Obj) {
        let mut gray_stack = std::mem::take(&mut self.gray_stack);
        gray_stack.write_no_gc(object, self);
        self.gray_stack = gray_stack;
    }
}

fn bind_method(vm: &mut VM, class: *mut ObjClass, name: *mut ObjString) -> Result<(), String> {
    let method = unsafe { (*class).methods.get(name) };
    if let Some(m) = method {
        let receiver = vm.stack.peek(0).clone();
        let objects = std::ptr::addr_of_mut!(vm.objects);
        let bound_method = ObjBoundMethod::new(objects, receiver, unsafe { (*m).as_closure() }, vm);
        vm.stack.pop();
        vm.stack.push(Value::from(bound_method));
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
    Value::Number(elapsed)
}

fn capture_upvalue(local: *mut Value, vm: &mut VM) -> *mut ObjUpvalue {
    let upvalue = std::ptr::addr_of_mut!(vm.open_upvalues);
    let mut prev_upvalue = std::ptr::null_mut();
    let mut curr_upvalue = unsafe { *upvalue };
    while !curr_upvalue.is_null() && unsafe { (*curr_upvalue).location } > local {
        prev_upvalue = curr_upvalue;
        curr_upvalue = unsafe { (*curr_upvalue).next };
    }

    if !curr_upvalue.is_null() && unsafe { (*curr_upvalue).location } == local {
        return curr_upvalue;
    }

    let objects = std::ptr::addr_of_mut!(vm.objects);
    let created_upvalue = ObjUpvalue::new(objects, local, vm);
    unsafe { (*created_upvalue).next = curr_upvalue };

    if prev_upvalue.is_null() {
        unsafe { *upvalue = created_upvalue };
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

impl GcCollector for VM {
    fn gc_context(&mut self) -> GcContext<'_> {
        GcContext {
            bytes_allocated: &mut self.bytes_allocated,
            next_gc: &mut self.next_gc,
            stress_gc: crate::DEBUG_STRESS_GC,
        }
    }

    fn collect_garbage(&mut self) {
        self.garbage_collect();
    }
}

impl VM {
    pub fn garbage_collect(&mut self) {
        let before = self.bytes_allocated;
        if DEBUG_LOG_GC {
            println!("-- gc begin");
        }

        self.mark_roots();
        self.trace_references();
        self.table_remove_white();
        self.sweep();

        self.next_gc = self.bytes_allocated * 2;

        if DEBUG_LOG_GC {
            println!("-- gc end");
            println!(
                "   collected {} bytes (from {} to {}) next at {}",
                before - self.bytes_allocated,
                before,
                self.bytes_allocated,
                self.next_gc
            );
        }
    }

    fn mark_roots(&mut self) {
        unsafe {
            for i in 0..self.stack.len() {
                let value = &mut self.stack[i].clone();
                self.mark_value(value);
            }

            for i in 0..self.frame_count as usize {
                self.mark_object(self.frames[i].closure as *mut Obj);
            }

            let mut next_upvalue = self.open_upvalues;
            while !next_upvalue.is_null() {
                self.mark_object(next_upvalue as *mut Obj);
                next_upvalue = (*next_upvalue).next;
            }

            for i in 0..self.globals.capacity {
                let entry = self.globals.entries.add(i);
                self.mark_object((*entry).key as *mut Obj);
                self.mark_value(&mut (*entry).value);
            }
            self.mark_compiler_roots(self.compiler);
            self.mark_object(self.init_string as *mut Obj);
        }
    }

    fn mark_value(&mut self, value: &mut Value) {
        if value.is_obj() {
            self.mark_object(value.as_obj());
        }
    }

    fn mark_object(&mut self, object: *mut Obj) {
        if object.is_null() {
            return;
        }
        if unsafe { (*object).is_marked } {
            return;
        }

        if DEBUG_LOG_GC {
            println!("{:?} mark", object);
            println!("{:?}", Value::from(object));
        }

        unsafe {
            (*object).is_marked = true;
        }

        self.gray_stack_push_no_gc(object);
    }

    fn mark_table(&mut self, table: &mut HashTable) {
        for i in 0..table.capacity {
            let entry = unsafe { table.entries.add(i) };
            self.mark_object(unsafe { (*entry).key } as *mut Obj);
            self.mark_value(unsafe { &mut (*entry).value });
        }
    }

    fn mark_compiler_roots(&mut self, compiler: *mut Compiler) {
        let mut curr_compiler = compiler;
        while !curr_compiler.is_null() {
            self.mark_object(unsafe { (*curr_compiler).function } as *mut Obj);
            curr_compiler = unsafe { (*curr_compiler).enclosing };
        }
    }

    fn trace_references(&mut self) {
        while self.gray_stack.count > 0 {
            self.gray_stack.count -= 1;
            let object = self.gray_stack[self.gray_stack.count];
            self.blacken_object(object);
        }
    }

    fn blacken_object(&mut self, object: *mut Obj) {
        if DEBUG_LOG_GC {
            println!("{:?} blacken", object);
            println!("{:?}", Value::from(object));
        }

        match unsafe { &(*object).otype } {
            ObjType::String => {}
            ObjType::Native => {}
            ObjType::Upvalue => {
                let upvalue = object as *mut ObjUpvalue;
                self.mark_value(unsafe { &mut (*upvalue).closed });
            }
            ObjType::Function => {
                let function = object as *mut ObjFunction;
                self.mark_object(unsafe { (*function).name } as *mut Obj);
                for i in 0..unsafe { (*function).chunk.values.count } {
                    self.mark_value(unsafe { &mut (&mut (*function).chunk.values)[i] });
                }
            }
            ObjType::Closure => {
                let closure = object as *mut ObjClosure;
                self.mark_object(unsafe { (*closure).function } as *mut Obj);
                for i in 0..unsafe { (*closure).upvalue_count } {
                    self.mark_object(unsafe { (*closure).upvalues.add(i) } as *mut Obj);
                }
            }
            ObjType::Class => {
                let class = object as *mut ObjClass;
                self.mark_object(unsafe { (*class).name } as *mut Obj);
                self.mark_table(unsafe { &mut (*class).methods });
            }
            ObjType::Instance => {
                let instance = object as *mut ObjInstance;
                self.mark_object(unsafe { (*instance).class } as *mut Obj);
                self.mark_table(unsafe { &mut (*instance).fields });
            }
            ObjType::BoundMethod => {
                let bound_method = object as *mut ObjBoundMethod;
                self.mark_value(unsafe { &mut (*bound_method).receiver });
                self.mark_object(unsafe { (*bound_method).method } as *mut Obj);
            }
        }
    }

    fn table_remove_white(&mut self) {
        unsafe {
            let mut i = 0;
            while i < self.strings.capacity {
                let entry = self.strings.entries.add(i);
                let key = (*entry).key;
                let key_obj = key as *mut Obj;
                if !key.is_null() && !(*key_obj).is_marked {
                    self.strings.delete(key);
                }
                i += 1;
            }
        }
    }

    fn sweep(&mut self) {
        unsafe {
            let mut previous = std::ptr::null_mut();
            let mut object = self.objects;
            while !object.is_null() {
                if (*object).is_marked {
                    (*object).is_marked = false;
                    previous = object;
                    object = (*object).next.0;
                } else {
                    let unreached = object;
                    object = (*object).next.0;
                    if !previous.is_null() {
                        (*previous).next = object.into();
                    } else {
                        self.objects = object;
                    }
                    free_object(unreached, self);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpret_compile_error() {
        let mut vm = VM::new();
        let source = "print 1>;";
        let result = vm.interpret(source);
        assert_eq!(result, InterpretResult::CompileError);
    }

    #[test]
    fn test_interpret_runtime_error() {
        let mut vm = VM::new();
        let source = "print true + false;";
        let result = vm.interpret(source);
        assert_eq!(result, InterpretResult::RuntimeError);
    }

    macro_rules! test_interpret {
        ($($name:ident: $data:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (source, expected) = $data;

                    let mut vm = VM::new();
                    let source = format!("print {};", source);
                    let result = vm.interpret(&source);
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
        let source = "class Brioche {} print Brioche;";
        let result = vm.interpret(source);

        assert_eq!(result, InterpretResult::Ok);
        assert_eq!(vm.output(), &["Brioche".to_string()]);
    }
}
