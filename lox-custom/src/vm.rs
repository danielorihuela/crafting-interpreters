use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    DEBUG_LOG_GC,
    compiler::{Compiler, FunctionType, Parser},
    memory::heap::{Heap, ObjId},
    scanner::Scanner,
    types::{
        opcode::OpCode,
        value::{
            Value,
            class::{ObjBoundMethod, ObjClass, ObjInstance},
            closure::ObjClosure,
            native::{NativeFn, ObjNative},
            obj::{HeapObj, Obj},
            upvalue::ObjUpvalue,
        },
    },
};

pub const FRAMES_MAX: usize = 64;

pub type StringsTable = HashMap<String, ObjId>;
pub type GlobalsTable = HashMap<ObjId, Value>;

pub struct VM {
    pub heap: Heap,
    pub stack: Vec<Value>,
    pub strings: StringsTable,

    pub frames: [CallFrame; FRAMES_MAX],
    pub frame_count: u8,

    pub open_upvalues: ObjId,
    pub compiler: Option<Rc<RefCell<Compiler>>>,
    pub globals: GlobalsTable,
    gray_stack: Vec<ObjId>,

    pub bytes_allocated: usize,
    pub next_gc: usize,

    pub init_string: ObjId,

    #[cfg(test)]
    output: Vec<String>,
}

impl VM {
    pub fn new() -> Self {
        let call_frame = CallFrame {
            closure: ObjId::null(),
            ip: 0,
            slots: 0,
        };

        let mut vm = VM {
            heap: Heap::new(),
            stack: Vec::new(),
            strings: StringsTable::new(),
            frames: [(); FRAMES_MAX].map(|_| call_frame.clone()),
            frame_count: 0,
            open_upvalues: ObjId::null(),
            compiler: None,
            globals: GlobalsTable::new(),
            gray_stack: Vec::new(),
            bytes_allocated: 0,
            next_gc: 1024 * 1024,
            init_string: ObjId::null(),
            #[cfg(test)]
            output: Vec::new(),
        };

        vm.init_string = vm.allocate_string("init");
        vm.define_native("clock", clock_native);

        vm
    }

    pub fn interpret(&mut self, source: &str) -> InterpretResult {
        let scanner = &mut Scanner::new(source);
        let compiler = Compiler::new(FunctionType::Script, None, "", self);
        self.compiler = Some(Rc::new(RefCell::new(compiler)));

        let parser = &mut Parser::new(scanner, self.compiler.clone(), self);
        let function_id = parser.compile();

        if function_id.is_null() {
            return InterpretResult::CompileError;
        }

        self.stack.push(Value::Obj(Obj::Function(function_id)));
        let HeapObj::Function(function) = &self.heap[function_id] else {
            panic!("Expected a function object");
        };
        let closure_id = self.allocate(ObjClosure::new(function_id, function.upvalue_count));
        self.stack.pop().unwrap();
        self.stack.push(Value::Obj(Obj::Closure(closure_id)));

        self.call(closure_id, 0);
        self.run()
    }

    fn run(&mut self) -> InterpretResult {
        #[cfg(debug_assertions)]
        {
            println!("\n=== Running bytecode ===");
        }

        loop {
            let frame_index = self.current_frame_index();

            #[cfg(debug_assertions)]
            {
                print!("          ");
                for i in 0..self.stack.len() {
                    print!("[{}]", self.stack[i].to_string(self));
                }
                println!();

                let frame = &self.frames[frame_index];
                let HeapObj::Closure(closure) = &self.heap[frame.closure] else {
                    panic!("Expected closure");
                };
                let HeapObj::Function(function) = &self.heap[closure.function_id] else {
                    panic!("Expected function");
                };

                use crate::types::chunk::debug::disassemble_instruction;
                let _ = disassemble_instruction(&function.chunk, frame.ip, self);
            }

            let instruction = OpCode::from(self.read_byte_from_frame(frame_index));
            match instruction {
                OpCode::Constant => {
                    let position = self.read_byte_from_frame(frame_index) as usize;

                    let function_id = self.frame_function_id(frame_index);
                    let value = {
                        let HeapObj::Function(function) = &self.heap[function_id] else {
                            panic!("Expected function object");
                        };
                        function.chunk.values[position].clone()
                    };
                    self.stack.push(value);
                }
                OpCode::Add => {
                    let b = self.stack[self.stack.len() - 1].clone();
                    let a = self.stack[self.stack.len() - 2].clone();
                    let result = if a.is_string() && b.is_string() {
                        let a_id = a.as_string();
                        let a_text = {
                            let HeapObj::String(a_str) = &self.heap[a_id] else {
                                panic!("Expected string object");
                            };
                            a_str.clone()
                        };
                        let b_id = b.as_string();
                        let b_text = {
                            let HeapObj::String(b_str) = &self.heap[b_id] else {
                                panic!("Expected string object");
                            };
                            b_str.clone()
                        };
                        Ok(Value::Obj(Obj::String(
                            self.allocate_string(&format!("{a_text}{b_text}")),
                        )))
                    } else {
                        a + b
                    };
                    self.stack.pop().unwrap();
                    self.stack.pop().unwrap();
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
                        let b = self.stack.pop().unwrap();
                        let a = self.stack.pop().unwrap();
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
                    if !self.stack[self.stack.len() - 1].is_number() {
                        self.runtime_error("Operand must be a number.");
                        return InterpretResult::RuntimeError;
                    }
                    let value = self.stack.pop().unwrap();
                    self.stack.push(Value::Number(-value.as_number()));
                }
                OpCode::Not => {
                    let value = self.stack.pop().unwrap();
                    self.stack.push(Value::Bool(value.is_falsey()));
                }
                OpCode::Equal => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(Value::Bool(a == b));
                }
                OpCode::False => self.stack.push(Value::Bool(false)),
                OpCode::True => self.stack.push(Value::Bool(true)),
                OpCode::Nil => self.stack.push(Value::Nil),
                OpCode::Return => {
                    let result = self.stack.pop().unwrap();
                    let slots = self.frames[frame_index].slots;
                    close_upvalues(self, slots);
                    self.frame_count -= 1;
                    if self.frame_count == 0 {
                        self.stack.pop().unwrap();
                        return InterpretResult::Ok;
                    }

                    self.stack.truncate(slots);
                    self.stack.push(result);
                }
                OpCode::Print => {
                    let value = self.stack.pop().unwrap();
                    let text = value.to_string(self);
                    #[cfg(test)]
                    self.output.push(text.clone());
                    println!("{}", text);
                }
                OpCode::Pop => {
                    self.stack.pop().unwrap();
                }
                OpCode::DefineGlobal => {
                    let position = self.read_byte_from_frame(frame_index) as usize;

                    let name = self.read_constant_string_id(frame_index, position);
                    self.globals
                        .insert(name, self.stack[self.stack.len() - 1].clone());
                    self.stack.pop().unwrap();
                }
                OpCode::GetGlobal => {
                    let position = self.read_byte_from_frame(frame_index) as usize;

                    let name = self.read_constant_string_id(frame_index, position);
                    match self.globals.get(&name) {
                        Some(value) => self.stack.push(value.clone()),
                        None => {
                            self.runtime_error(&format!(
                                "Undefined variable '{}'.",
                                self.string_text(name)
                            ));
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::SetGlobal => {
                    let position = self.read_byte_from_frame(frame_index) as usize;

                    let name = self.read_constant_string_id(frame_index, position);
                    if let std::collections::hash_map::Entry::Occupied(mut entry) =
                        self.globals.entry(name)
                    {
                        entry.insert(self.stack[self.stack.len() - 1].clone());
                    } else {
                        self.runtime_error(&format!(
                            "Undefined variable '{}'.",
                            self.string_text(name)
                        ));
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::GetLocal => {
                    let slot = self.read_byte_from_frame(frame_index) as usize;
                    let slots = self.frames[frame_index].slots;
                    self.stack.push(self.stack[slots + slot].clone());
                }
                OpCode::SetLocal => {
                    let slot = self.read_byte_from_frame(frame_index) as usize;
                    let slots = self.frames[frame_index].slots;
                    self.stack[slots + slot] = self.stack[self.stack.len() - 1].clone();
                }
                OpCode::JumpIfFalse => {
                    let offset_0 = self.read_byte_from_frame(frame_index) as usize;
                    let offset_1 = self.read_byte_from_frame(frame_index) as usize;
                    let offset = (offset_0 << 8) | offset_1;

                    if self.stack[self.stack.len() - 1].is_falsey() {
                        self.advance_frame_ip(frame_index, offset);
                    }
                }
                OpCode::Jump => {
                    let offset_0 = self.read_byte_from_frame(frame_index) as usize;
                    let offset_1 = self.read_byte_from_frame(frame_index) as usize;
                    self.advance_frame_ip(frame_index, (offset_0 << 8) | offset_1);
                }
                OpCode::Loop => {
                    let offset_0 = self.read_byte_from_frame(frame_index) as usize;
                    let offset_1 = self.read_byte_from_frame(frame_index) as usize;
                    self.rewind_frame_ip(frame_index, (offset_0 << 8) | offset_1);
                }
                OpCode::Call => {
                    let arg_count = self.read_byte_from_frame(frame_index) as usize;

                    let callee = self.stack[self.stack.len() - 1 - arg_count].clone();
                    if !self.call_value(callee, arg_count) {
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::Closure => {
                    let position = self.read_byte_from_frame(frame_index) as usize;

                    let function_id = self.read_constant_function_id(frame_index, position);
                    let HeapObj::Function(function) = &self.heap[function_id] else {
                        panic!("Expected a function object");
                    };
                    let closure_id =
                        self.allocate(ObjClosure::new(function_id, function.upvalue_count));
                    self.stack.push(Value::Obj(Obj::Closure(closure_id)));

                    let upvalue_count = {
                        let HeapObj::Function(function) = &self.heap[function_id] else {
                            panic!("Expected string object");
                        };
                        function.upvalue_count
                    };

                    for i in 0..upvalue_count {
                        let is_local = self.read_byte_from_frame(frame_index) != 0;

                        let index = self.read_byte_from_frame(frame_index) as usize;

                        let upvalue_id = if is_local {
                            let slots = self.frames[frame_index].slots;
                            capture_upvalue(slots + index, self)
                        } else {
                            let parent_closure = self.frames[frame_index].closure;
                            let HeapObj::Closure(parent) = &self.heap[parent_closure] else {
                                panic!("Expected closure object");
                            };
                            parent.upvalues[index]
                        };

                        let HeapObj::Closure(closure) = &mut self.heap[closure_id] else {
                            panic!("Expected closure object");
                        };
                        closure.upvalues[i] = upvalue_id;
                    }
                }
                OpCode::GetUpvalue => {
                    let slot = self.read_byte_from_frame(frame_index) as usize;

                    let closure_id = self.frames[frame_index].closure;
                    let upvalue_id = {
                        let HeapObj::Closure(closure) = &self.heap[closure_id] else {
                            panic!("Expected closure object");
                        };
                        closure.upvalues[slot]
                    };

                    let value = {
                        let HeapObj::Upvalue(upvalue) = &self.heap[upvalue_id] else {
                            panic!("Expected upvalue object");
                        };
                        if upvalue.location != -1 {
                            self.stack[upvalue.location as usize].clone()
                        } else {
                            upvalue.closed.clone()
                        }
                    };
                    self.stack.push(value);
                }
                OpCode::SetUpvalue => {
                    let slot = self.read_byte_from_frame(frame_index) as usize;

                    let closure_id = self.frames[frame_index].closure;
                    let upvalue_id = {
                        let HeapObj::Closure(closure) = &self.heap[closure_id] else {
                            panic!("Expected closure object");
                        };
                        closure.upvalues[slot]
                    };

                    let HeapObj::Upvalue(upvalue) = &mut self.heap[upvalue_id] else {
                        panic!("Expected upvalue object");
                    };
                    if upvalue.location != -1 {
                        self.stack[upvalue.location as usize] =
                            self.stack[self.stack.len() - 1].clone();
                    } else {
                        upvalue.closed = self.stack[self.stack.len() - 1].clone();
                    }
                }
                OpCode::CloseUpvalue => {
                    let last = self.stack.len() - 1;
                    close_upvalues(self, last);
                    self.stack.pop().unwrap();
                }
                OpCode::Class => {
                    let position = self.read_byte_from_frame(frame_index) as usize;

                    let name = self.read_constant_string_id(frame_index, position);
                    let class_id = self.allocate(ObjClass::new(name));
                    self.stack.push(Value::Obj(Obj::Class(class_id)));
                }
                OpCode::GetProperty => {
                    if !self.stack[self.stack.len() - 1].is_instance() {
                        self.runtime_error("Only instances have properties.");
                        return InterpretResult::RuntimeError;
                    }

                    let instance_id = self.stack[self.stack.len() - 1].as_instance();
                    let name_position = self.read_byte_from_frame(frame_index) as usize;

                    let name = self.read_constant_string_id(frame_index, name_position);

                    let field_value = {
                        let HeapObj::Instance(instance) = &self.heap[instance_id] else {
                            panic!("Expected instance object");
                        };
                        instance.fields.get(&name).cloned()
                    };

                    if let Some(v) = field_value {
                        self.stack.pop().unwrap();
                        self.stack.push(v);
                    } else {
                        let class_id = {
                            let HeapObj::Instance(instance) = &self.heap[instance_id] else {
                                panic!("Expected instance object");
                            };
                            instance.class
                        };
                        if let Err(message) = bind_method(self, class_id, name) {
                            self.runtime_error(&message);
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::SetProperty => {
                    if !self.stack[self.stack.len() - 2].is_instance() {
                        self.runtime_error("Only instances have fields.");
                        return InterpretResult::RuntimeError;
                    }

                    let instance_id = self.stack[self.stack.len() - 2].as_instance();
                    let name_position = self.read_byte_from_frame(frame_index) as usize;

                    let name = self.read_constant_string_id(frame_index, name_position);
                    let value = self.stack[self.stack.len() - 1].clone();

                    let HeapObj::Instance(instance) = &mut self.heap[instance_id] else {
                        panic!("Expected instance object");
                    };
                    instance.fields.insert(name, value.clone());

                    self.stack.pop().unwrap();
                    self.stack.pop().unwrap();
                    self.stack.push(value);
                }
                OpCode::Method => {
                    let name_position = self.read_byte_from_frame(frame_index) as usize;

                    let name = self.read_constant_string_id(frame_index, name_position);
                    let class_id = self.stack[self.stack.len() - 2].as_class();
                    let method = self.stack[self.stack.len() - 1].clone();

                    let HeapObj::Class(class) = &mut self.heap[class_id] else {
                        panic!("Expected class object");
                    };
                    class.methods.insert(name, method);
                    self.stack.pop().unwrap();
                }
                OpCode::Invoke => {
                    let method_position = self.read_byte_from_frame(frame_index) as usize;
                    let method = self.read_constant_string_id(frame_index, method_position);

                    let arg_count = self.read_byte_from_frame(frame_index) as usize;

                    if !self.invoke(method, arg_count) {
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::Inherit => {
                    let superclass = self.stack[self.stack.len() - 2].clone();
                    if !superclass.is_class() {
                        self.runtime_error("Superclass must be a class.");
                        return InterpretResult::RuntimeError;
                    }

                    let superclass_id = superclass.as_class();
                    let subclass_id = self.stack[self.stack.len() - 1].as_class();

                    let inherited = {
                        let HeapObj::Class(superclass) = &self.heap[superclass_id] else {
                            panic!("Expected class object");
                        };
                        superclass.methods.clone()
                    };

                    let HeapObj::Class(subclass) = &mut self.heap[subclass_id] else {
                        panic!("Expected class object");
                    };
                    subclass.methods.extend(inherited);
                    self.stack.pop().unwrap();
                }
                OpCode::GetSuper => {
                    let name_position = self.read_byte_from_frame(frame_index) as usize;

                    let name = self.read_constant_string_id(frame_index, name_position);
                    let superclass = self.stack.pop().unwrap().as_class();

                    if bind_method(self, superclass, name).is_err() {
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::SuperInvoke => {
                    let method_position = self.read_byte_from_frame(frame_index) as usize;
                    let method = self.read_constant_string_id(frame_index, method_position);

                    let arg_count = self.read_byte_from_frame(frame_index) as usize;

                    let superclass = self.stack.pop().unwrap().as_class();
                    if !self.invoke_from_class(superclass, method, arg_count) {
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::Unknown => panic!("Invalid opcode"),
            }
        }
    }

    fn current_frame_index(&self) -> usize {
        self.frame_count as usize - 1
    }

    fn read_byte_from_frame(&mut self, frame_index: usize) -> u8 {
        let ip = self.frames[frame_index].ip;
        let closure_id = self.frames[frame_index].closure;
        let HeapObj::Closure(closure) = &self.heap[closure_id] else {
            panic!("Expected closure object");
        };
        let HeapObj::Function(function) = &self.heap[closure.function_id] else {
            panic!("Expected function object");
        };
        let byte = function.chunk.code[ip];
        self.frames[frame_index].ip = ip + 1;

        byte
    }

    fn advance_frame_ip(&mut self, frame_index: usize, offset: usize) {
        let ip = self.frames[frame_index].ip;
        self.frames[frame_index].ip = ip + offset;
    }

    fn rewind_frame_ip(&mut self, frame_index: usize, offset: usize) {
        let ip = self.frames[frame_index].ip;
        self.frames[frame_index].ip = ip - offset;
    }

    fn frame_function_id(&self, frame_index: usize) -> ObjId {
        let closure_id = self.frames[frame_index].closure;
        let HeapObj::Closure(closure) = &self.heap[closure_id] else {
            panic!("Expected closure object");
        };
        closure.function_id
    }

    fn read_constant_string_id(&self, frame_index: usize, position: usize) -> ObjId {
        let function_id = self.frame_function_id(frame_index);
        let value = {
            let HeapObj::Function(function) = &self.heap[function_id] else {
                panic!("Expected function object");
            };
            function.chunk.values[position].clone()
        };
        value.as_string()
    }

    fn read_constant_function_id(&self, frame_index: usize, position: usize) -> ObjId {
        let function_id = self.frame_function_id(frame_index);
        let value = {
            let HeapObj::Function(function) = &self.heap[function_id] else {
                panic!("Expected function object");
            };
            function.chunk.values[position].clone()
        };
        value.as_function()
    }

    fn string_text(&self, id: ObjId) -> String {
        let HeapObj::String(s) = &self.heap[id] else {
            panic!("Expected string object");
        };
        s.clone()
    }

    fn invoke(&mut self, name: ObjId, arg_count: usize) -> bool {
        let receiver = self.stack[self.stack.len() - 1 - arg_count].clone();
        if !receiver.is_instance() {
            self.runtime_error("Only instances have methods.");
            return false;
        }

        let instance_id = receiver.as_instance();
        let field = {
            let HeapObj::Instance(instance) = &self.heap[instance_id] else {
                panic!("Expected instance object");
            };
            instance.fields.get(&name).cloned()
        };

        if let Some(value) = field {
            let stack_len = self.stack.len();
            self.stack[stack_len - arg_count - 1] = value.clone();
            return self.call_value(value, arg_count);
        }

        let class_id = {
            let HeapObj::Instance(instance) = &self.heap[instance_id] else {
                panic!("Expected instance object");
            };
            instance.class
        };

        self.invoke_from_class(class_id, name, arg_count)
    }

    fn invoke_from_class(&mut self, class_id: ObjId, name: ObjId, arg_count: usize) -> bool {
        let method = {
            let HeapObj::Class(class) = &self.heap[class_id] else {
                panic!("Expected class object");
            };
            class.methods.get(&name).cloned()
        };

        if let Some(value) = method {
            return self.call(value.as_closure(), arg_count);
        }

        self.runtime_error(&format!("Undefined property '{}'.", self.string_text(name)));
        false
    }

    fn call_value(&mut self, callee: Value, arg_count: usize) -> bool {
        if callee.is_function() {
            unreachable!("Functions are always wrapped in closures")
        } else if callee.is_class() {
            let class_id = callee.as_class();
            let instance_id = self.allocate(ObjInstance::new(class_id));
            let instance_value = Value::Obj(Obj::Instance(instance_id));
            let stack_len = self.stack.len();
            self.stack[stack_len - arg_count - 1] = instance_value;

            let initializer = {
                let HeapObj::Class(class) = &self.heap[class_id] else {
                    panic!("Expected class object");
                };
                class.methods.get(&self.init_string).cloned()
            };

            if let Some(initializer) = initializer {
                return self.call(initializer.as_closure(), arg_count);
            }
            if arg_count != 0 {
                self.runtime_error(&format!("Expected 0 arguments but got {}.", arg_count));
                return false;
            }
            true
        } else if callee.is_native() {
            let native_id = callee.as_native();
            let function = {
                let HeapObj::Native(native) = &self.heap[native_id] else {
                    panic!("Expected native object");
                };
                native.function
            };
            let stack_base = self.stack.len() - arg_count;
            let args = unsafe { self.stack.as_mut_ptr().add(stack_base) };
            let result = function(arg_count, args);
            self.stack.truncate(stack_base - 1);
            self.stack.push(result);
            true
        } else if callee.is_closure() {
            self.call(callee.as_closure(), arg_count)
        } else if callee.is_bound_method() {
            let bm_id = callee.as_bound_method();
            let (receiver, method) = {
                let HeapObj::BoundMethod(bound_method) = &self.heap[bm_id] else {
                    panic!("Expected bound method object");
                };
                (bound_method.receiver.clone(), bound_method.method)
            };
            let stack_len = self.stack.len();
            self.stack[stack_len - arg_count - 1] = receiver;
            self.call(method, arg_count)
        } else {
            self.runtime_error("Can only call functions and classes.");
            false
        }
    }

    fn call(&mut self, closure_id: ObjId, arg_count: usize) -> bool {
        let function_id = {
            let HeapObj::Closure(closure) = &self.heap[closure_id] else {
                panic!("Expected closure object");
            };
            closure.function_id
        };

        let arity = {
            let HeapObj::Function(function) = &self.heap[function_id] else {
                panic!("Expected function object");
            };
            function.arity
        };

        if arg_count != arity {
            self.runtime_error(&format!(
                "Expected {} arguments but got {}.",
                arity, arg_count
            ));
            return false;
        }

        if self.frame_count as usize == FRAMES_MAX {
            self.runtime_error("Stack overflow.");
            return false;
        }

        let frame = &mut self.frames[self.frame_count as usize];
        self.frame_count += 1;
        frame.closure = closure_id;
        frame.ip = 0;
        let stack_base = self.stack.len() - arg_count - 1;
        frame.slots = stack_base;

        true
    }

    #[cfg(test)]
    fn output(&self) -> &[String] {
        &self.output
    }

    pub fn free(&mut self) {
        self.reset_stack();
        self.globals.clear();
        self.strings.clear();
        self.heap.clear();
        self.compiler = None;
        self.init_string = ObjId::null();
    }

    fn runtime_error(&mut self, message: &str) {
        eprintln!("{}", message);

        for i in (0..self.frame_count).rev() {
            let frame = &self.frames[i as usize];
            let function_id = {
                let HeapObj::Closure(closure) = &self.heap[frame.closure] else {
                    break;
                };
                closure.function_id
            };
            let HeapObj::Function(function) = &self.heap[function_id] else {
                break;
            };

            let instruction = frame.ip - 1;
            let line = function.chunk.lines[instruction];
            let where_ = if function.name.is_null() {
                "script".to_string()
            } else {
                self.string_text(function.name)
            };
            eprintln!("[line {}] in {}", line, where_);
        }

        self.reset_stack();
    }

    fn reset_stack(&mut self) {
        self.stack = Vec::new();
        self.frame_count = 0;
        self.open_upvalues = ObjId::null();
        self.gray_stack.clear();
    }

    fn define_native(&mut self, name: &str, function: NativeFn) {
        let name_id = self.allocate_string(name);
        let native_id = self.allocate(ObjNative::new(function));
        // let native_id = ObjNative::new(self, function);
        self.globals
            .insert(name_id, Value::Obj(Obj::Native(native_id)));
    }
}

fn bind_method(vm: &mut VM, class_id: ObjId, name: ObjId) -> Result<(), String> {
    let method = {
        let HeapObj::Class(class) = &vm.heap[class_id] else {
            panic!("Expected class object");
        };
        class.methods.get(&name).cloned()
    };

    if let Some(m) = method {
        let receiver = vm.stack[vm.stack.len() - 1].clone();
        let bound_method = vm.allocate(ObjBoundMethod::new(receiver, m.as_closure()));
        vm.stack.pop();
        vm.stack.push(Value::Obj(Obj::BoundMethod(bound_method)));
        Ok(())
    } else {
        Err(format!("Undefined property '{}'.", vm.string_text(name)))
    }
}

fn capture_upvalue(local: usize, vm: &mut VM) -> ObjId {
    let mut prev = ObjId::null();
    let mut curr = vm.open_upvalues;

    while !curr.is_null() {
        let curr_location = {
            let HeapObj::Upvalue(upvalue) = &vm.heap[curr] else {
                panic!("Expected upvalue object");
            };
            upvalue.location
        };

        if curr_location <= local as isize {
            break;
        }

        prev = curr;
        let next = {
            let HeapObj::Upvalue(upvalue) = &vm.heap[curr] else {
                panic!("Expected upvalue object");
            };
            upvalue.next
        };
        curr = next;
    }

    if !curr.is_null() {
        let existing_location = {
            let HeapObj::Upvalue(upvalue) = &vm.heap[curr] else {
                panic!("Expected upvalue object");
            };
            upvalue.location
        };

        if existing_location == local as isize {
            return curr;
        }
    }

    let created = vm.allocate(ObjUpvalue::new(local as isize));
    {
        let HeapObj::Upvalue(upvalue) = &mut vm.heap[created] else {
            panic!("Expected upvalue object");
        };
        upvalue.next = curr;
    }

    if prev.is_null() {
        vm.open_upvalues = created;
    } else {
        let HeapObj::Upvalue(prev_upvalue) = &mut vm.heap[prev] else {
            panic!("Expected upvalue object");
        };
        prev_upvalue.next = created;
    }

    created
}

fn close_upvalues(vm: &mut VM, last: usize) {
    while !vm.open_upvalues.is_null() {
        let curr = vm.open_upvalues;
        let location = {
            let HeapObj::Upvalue(upvalue) = &vm.heap[curr] else {
                panic!("Expected upvalue object");
            };
            upvalue.location
        };

        if location < last as isize {
            break;
        }

        {
            let HeapObj::Upvalue(upvalue) = &mut vm.heap[curr] else {
                panic!("Expected upvalue object");
            };
            upvalue.closed = vm.stack[location as usize].clone();
            upvalue.location = -1;
            vm.open_upvalues = upvalue.next;
        }
    }
}

fn clock_native(_: usize, _: *mut Value) -> Value {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
    Value::Number(elapsed)
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
    pub closure: ObjId,
    pub ip: usize,
    pub slots: usize,
}

impl Drop for VM {
    fn drop(&mut self) {
        self.free();
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

        self.next_gc = self.bytes_allocated.saturating_mul(2).max(1024 * 1024);

        if DEBUG_LOG_GC {
            println!("-- gc end");
            println!(
                "   collected {} bytes (from {} to {}) next at {}",
                before.saturating_sub(self.bytes_allocated),
                before,
                self.bytes_allocated,
                self.next_gc
            );
        }
    }

    fn mark_roots(&mut self) {
        for i in 0..self.stack.len() {
            let value = self.stack[i].clone();
            self.mark_value(&value);
        }

        for i in 0..self.frame_count as usize {
            self.mark_object(self.frames[i].closure);
        }

        let mut next_upvalue = self.open_upvalues;
        while !next_upvalue.is_null() {
            self.mark_object(next_upvalue);

            let HeapObj::Upvalue(upvalue) = &self.heap[next_upvalue] else {
                break;
            };
            next_upvalue = upvalue.next;
        }

        let global_roots: Vec<(ObjId, Value)> = self
            .globals
            .iter()
            .map(|(key, value)| (*key, value.clone()))
            .collect();
        for (name, value) in global_roots {
            self.mark_object(name);
            self.mark_value(&value);
        }

        self.mark_compiler_roots(self.compiler.clone());
        self.mark_object(self.init_string);
    }

    fn mark_value(&mut self, value: &Value) {
        if let Value::Obj(obj) = value {
            match obj {
                Obj::String(id)
                | Obj::Function(id)
                | Obj::Closure(id)
                | Obj::Native(id)
                | Obj::Upvalue(id)
                | Obj::Class(id)
                | Obj::Instance(id)
                | Obj::BoundMethod(id) => self.mark_object(*id),
            }
        }
    }

    fn mark_object(&mut self, id: ObjId) {
        if id.is_null() || !self.heap.contains(id) {
            return;
        }

        if self.heap.mark(id) {
            self.gray_stack.push(id);
        }
    }

    fn mark_compiler_roots(&mut self, compiler: Option<Rc<RefCell<Compiler>>>) {
        let mut curr_compiler = compiler;
        while let Some(compiler_rc) = curr_compiler {
            let function = compiler_rc.borrow().function;
            self.mark_object(function);
            curr_compiler = compiler_rc.borrow().enclosing.clone();
        }
    }

    fn trace_references(&mut self) {
        while let Some(object_id) = self.gray_stack.pop() {
            self.blacken_object(object_id);
        }
    }

    fn blacken_object(&mut self, object_id: ObjId) {
        let mut children = Vec::new();
        let mut value_children = Vec::new();

        match &self.heap[object_id] {
            HeapObj::String(_) => {}
            HeapObj::Native(_) => {}
            HeapObj::Upvalue(upvalue) => {
                value_children.push(upvalue.closed.clone());
            }
            HeapObj::Function(function) => {
                children.push(function.name);
                for i in 0..function.chunk.values.len() {
                    value_children.push(function.chunk.values[i].clone());
                }
            }
            HeapObj::Closure(closure) => {
                children.push(closure.function_id);
                for upvalue in &closure.upvalues {
                    children.push(*upvalue);
                }
            }
            HeapObj::Class(class) => {
                children.push(class.name);
                for (name, method) in &class.methods {
                    children.push(*name);
                    value_children.push(method.clone());
                }
            }
            HeapObj::Instance(instance) => {
                children.push(instance.class);
                for (name, value) in &instance.fields {
                    children.push(*name);
                    value_children.push(value.clone());
                }
            }
            HeapObj::BoundMethod(bound_method) => {
                value_children.push(bound_method.receiver.clone());
                children.push(bound_method.method);
            }
        }

        for child in children {
            self.mark_object(child);
        }
        for value in value_children {
            self.mark_value(&value);
        }
    }

    fn table_remove_white(&mut self) {
        self.strings.retain(|_, id| self.heap.is_marked(*id));
    }

    fn sweep(&mut self) {
        let ids = self.heap.iter_ids();
        for id in ids {
            if self.heap.is_marked(id) {
                self.heap.clear_mark(id);
            } else {
                self.heap.deallocate(id);
                self.bytes_allocated -= std::mem::size_of::<HeapObj>();
            }
        }
    }
}

impl VM {
    pub fn allocate(&mut self, obj: impl Into<HeapObj>) -> ObjId {
        self.bytes_allocated += std::mem::size_of::<HeapObj>();
        if self.bytes_allocated > self.next_gc {
            self.garbage_collect();
        }

        self.heap.allocate(obj.into())
    }

    pub fn allocate_string(&mut self, data: &str) -> ObjId {
        let interned = self.strings.get(data);
        if let Some(interned) = interned {
            return *interned;
        }

        let id = self.allocate(data.to_string());

        self.strings.insert(data.to_string(), id);

        self.stack.push(Value::Obj(Obj::String(id)));
        self.strings.insert(data.to_string(), id);
        self.stack.pop();

        id
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
