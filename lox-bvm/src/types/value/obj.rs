use std::fmt::Display;
use std::ops::{Deref, DerefMut};
use std::ptr::drop_in_place;

use crate::collections::hashtable::HashTable;
use crate::compiler::Compiler;
use crate::memory::alloc::reallocate;
use crate::memory::array::free_array;
use crate::types::value::class::{ObjBoundMethod, ObjClass, ObjInstance};
use crate::types::value::closure::ObjClosure;
use crate::types::value::function::ObjFunction;
use crate::types::value::native::ObjNative;
use crate::types::value::string::ObjString;
use crate::types::value::upvalue::ObjUpvalue;
use crate::types::value::{ObjPtrTarget, Value};
use crate::{DEBUG_LOG_GC, VM_INSTANCE};

const GC_HEAP_GROW_FACTOR: usize = 2;

#[derive(PartialEq, Clone, Debug)]
pub enum ObjType {
    String,
    Function,
    Closure,
    Native,
    Upvalue,
    Class,
    Instance,
    BoundMethod,
}

pub struct Obj {
    pub otype: ObjType,
    pub next: ObjPtr,
    pub is_marked: bool,
}

impl ObjPtrTarget for Obj {}

impl Display for ObjPtr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = match self.otype {
            ObjType::String => {
                let s = self.0 as *mut ObjString;
                unsafe { (*s).to_string() }
            }
            ObjType::Function => {
                let function = self.0 as *mut ObjFunction;
                unsafe { (*function).to_string() }
            }
            ObjType::Closure => {
                let closure = self.0 as *mut ObjClosure;
                unsafe { (*closure).to_string() }
            }
            ObjType::Native => {
                let native = self.0 as *mut ObjNative;
                unsafe { (*native).to_string() }
            }
            ObjType::Upvalue => {
                let upvalue = self.0 as *mut ObjUpvalue;
                unsafe { (*upvalue).to_string() }
            }
            ObjType::Class => {
                let class = self.0 as *mut ObjClass;
                unsafe { (*class).to_string() }
            }
            ObjType::Instance => {
                let instance = self.0 as *mut ObjInstance;
                unsafe { (*instance).to_string() }
            }
            ObjType::BoundMethod => {
                let bound_method = self.0 as *mut ObjBoundMethod;
                unsafe { (*bound_method).to_string() }
            }
        };
        write!(f, "{}", data)
    }
}

pub fn allocate_object<T>(obj_type: ObjType, objects: *mut *mut Obj) -> *mut T {
    let obj = reallocate(std::ptr::null_mut::<T>(), 0, 1);
    let mut obj = ObjPtr(obj as *mut Obj);

    unsafe {
        obj.otype = obj_type;
        obj.is_marked = false;
        obj.next = ObjPtr(*objects);
        *objects = obj.0;
    };

    if DEBUG_LOG_GC {
        println!(
            "{:?} allocate {:?} for {:?}",
            obj.0,
            std::mem::size_of::<T>(),
            obj.otype
        );
    }

    obj.0 as *mut T
}

pub unsafe fn free_object(object: impl Into<ObjPtr>) {
    let object_ptr = object.into();

    if DEBUG_LOG_GC {
        println!("{:?} free type {:?}", object_ptr.0, object_ptr.otype);
    }

    match &object_ptr.otype {
        ObjType::String => {
            let string = object_ptr.0 as *mut ObjString;
            unsafe {
                free_array((*string).chars, (*string).length + 1, (*string).length + 1);
            };
            reallocate(string, 1, 0);
        }
        ObjType::Function => {
            let function = object_ptr.0 as *mut ObjFunction;
            unsafe { drop_in_place(&mut (*function).chunk) };
            reallocate(function, 1, 0);
        }
        ObjType::Closure => {
            let closure = object_ptr.0 as *mut ObjClosure;
            unsafe {
                free_array(
                    (*closure).upvalues,
                    (*closure).upvalue_count,
                    (*closure).upvalue_count,
                );
            }
            reallocate(closure, 1, 0);
        }
        ObjType::Native => {
            let native = object_ptr.0 as *mut ObjNative;
            reallocate(native, 1, 0);
        }
        ObjType::Upvalue => {
            let upvalue = object_ptr.0 as *mut ObjUpvalue;
            reallocate(upvalue, 1, 0);
        }
        ObjType::Class => {
            let class = object_ptr.0 as *mut ObjClass;
            unsafe { (*class).methods.free() };
            reallocate(class, 1, 0);
        }
        ObjType::Instance => {
            let instance = object_ptr.0 as *mut ObjInstance;
            unsafe { (*instance).fields.free() };
            reallocate(instance, 1, 0);
        }
        ObjType::BoundMethod => {
            let bound_method = object_ptr.0 as *mut ObjBoundMethod;
            reallocate(bound_method, 1, 0);
        }
    }
}

pub fn garbage_collect() {
    unsafe {
        let vm = &mut *VM_INSTANCE;
        let before = vm.bytes_allocated;
        if DEBUG_LOG_GC {
            println!("-- gc begin");
        }

        mark_roots();
        trace_references();
        table_remove_white();
        sweep();

        vm.next_gc = vm.bytes_allocated * GC_HEAP_GROW_FACTOR;

        if DEBUG_LOG_GC {
            println!("-- gc end");
            println!(
                "   collected {} bytes (from {} to {}) next at {}",
                before - vm.bytes_allocated,
                before,
                vm.bytes_allocated,
                vm.next_gc
            );
        }
    }
}

fn mark_roots() {
    unsafe {
        let vm = &mut *VM_INSTANCE;
        for i in 0..vm.stack.len() {
            let value = &mut vm.stack[i];
            mark_value(value);
        }

        for i in 0..vm.frame_count as usize {
            mark_object(vm.frames[i].closure as *mut Obj);
        }

        let mut next_upvalue = vm.open_upvalues;
        while !next_upvalue.is_null() {
            mark_object(next_upvalue as *mut Obj);
            next_upvalue = (*next_upvalue).next;
        }

        mark_table(&mut vm.globals);
        mark_compiler_roots(vm.compiler);
        mark_object(vm.init_string as *mut Obj);
    }
}

fn mark_value(value: &mut Value) {
    if value.is_obj() {
        mark_object(value.as_obj());
    }
}

fn mark_object(object: *mut Obj) {
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

    unsafe {
        let gray_stack = &mut (*VM_INSTANCE).gray_stack;
        gray_stack.write_no_gc(object);
    }
}

fn mark_table(table: &mut HashTable) {
    for i in 0..table.capacity {
        let entry = unsafe { table.entries.add(i) };
        mark_object(unsafe { (*entry).key } as *mut Obj);
        mark_value(unsafe { &mut (*entry).value });
    }
}

fn mark_compiler_roots(compiler: *mut Compiler) {
    let mut curr_compiler = compiler;
    while !curr_compiler.is_null() {
        mark_object(unsafe { (*curr_compiler).function } as *mut Obj);
        curr_compiler = unsafe { (*curr_compiler).enclosing };
    }
}

fn trace_references() {
    unsafe {
        let vm = &mut *VM_INSTANCE;
        while vm.gray_stack.count > 0 {
            vm.gray_stack.count -= 1;
            let object = vm.gray_stack[vm.gray_stack.count];
            blacken_object(object);
        }
    }
}

fn blacken_object(object: *mut Obj) {
    if DEBUG_LOG_GC {
        println!("{:?} blacken", object);
        println!("{:?}", Value::from(object));
    }

    match unsafe { &(*object).otype } {
        ObjType::String => {}
        ObjType::Native => {}
        ObjType::Upvalue => {
            let upvalue = object as *mut ObjUpvalue;
            mark_value(unsafe { &mut (*upvalue).closed });
        }
        ObjType::Function => {
            let function = object as *mut ObjFunction;
            mark_object(unsafe { (*function).name } as *mut Obj);
            for i in 0..unsafe { (*function).chunk.values.count } {
                mark_value(unsafe { &mut (&mut (*function).chunk.values)[i] });
            }
        }
        ObjType::Closure => {
            let closure = object as *mut ObjClosure;
            mark_object(unsafe { (*closure).function } as *mut Obj);
            for i in 0..unsafe { (*closure).upvalue_count } {
                mark_object(unsafe { (*closure).upvalues.add(i) } as *mut Obj);
            }
        }
        ObjType::Class => {
            let class = object as *mut ObjClass;
            mark_object(unsafe { (*class).name } as *mut Obj);
            mark_table(unsafe { &mut (*class).methods });
        }
        ObjType::Instance => {
            let instance = object as *mut ObjInstance;
            mark_object(unsafe { (*instance).class } as *mut Obj);
            mark_table(unsafe { &mut (*instance).fields });
        }
        ObjType::BoundMethod => {
            let bound_method = object as *mut ObjBoundMethod;
            mark_value(unsafe { &mut (*bound_method).receiver });
            mark_object(unsafe { (*bound_method).method } as *mut Obj);
        }
    }
}

fn table_remove_white() {
    unsafe {
        let vm = &mut *VM_INSTANCE;
        let mut i = 0;
        while i < vm.strings.capacity {
            let entry = vm.strings.entries.add(i);
            let key = (*entry).key;
            let key_obj = key as *mut Obj;
            if !key.is_null() && !(*key_obj).is_marked {
                vm.strings.delete(key);
            }
            i += 1;
        }
    }
}

fn sweep() {
    unsafe {
        let vm = &mut *VM_INSTANCE;
        let mut previous = std::ptr::null_mut();
        let mut object = vm.objects;
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
                    vm.objects = object;
                }
                free_object(unreached);
            }
        }
    }
}

pub struct ObjPtr(pub *mut Obj);

impl Deref for ObjPtr {
    type Target = Obj;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl DerefMut for ObjPtr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}

impl From<*mut Obj> for ObjPtr {
    fn from(value: *mut Obj) -> Self {
        ObjPtr(value)
    }
}
