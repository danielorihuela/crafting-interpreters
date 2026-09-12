use std::fmt::Display;
use std::ops::{Deref, DerefMut};
use std::ptr::drop_in_place;

use crate::DEBUG_LOG_GC;
use crate::memory::alloc::reallocate;
use crate::memory::array::free_array;
use crate::types::value::ObjPtrTarget;
use crate::types::value::class::{ObjBoundMethod, ObjClass, ObjInstance};
use crate::types::value::closure::ObjClosure;
use crate::types::value::function::ObjFunction;
use crate::types::value::native::ObjNative;
use crate::types::value::string::ObjString;
use crate::types::value::upvalue::ObjUpvalue;
use crate::vm::VM;

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

pub fn allocate_object<T>(obj_type: ObjType, objects: *mut *mut Obj, vm: &mut VM) -> *mut T {
    let obj = reallocate(std::ptr::null_mut::<T>(), 0, 1, vm);
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

pub unsafe fn free_object(object: impl Into<ObjPtr>, vm: &mut VM) {
    let object_ptr = object.into();

    if DEBUG_LOG_GC {
        println!("{:?} free type {:?}", object_ptr.0, object_ptr.otype);
    }

    match &object_ptr.otype {
        ObjType::String => {
            let string = object_ptr.0 as *mut ObjString;
            unsafe {
                free_array(
                    (*string).chars,
                    (*string).length + 1,
                    (*string).length + 1,
                    vm,
                );
            };
            reallocate(string, 1, 0, vm);
        }
        ObjType::Function => {
            let function = object_ptr.0 as *mut ObjFunction;
            unsafe { drop_in_place(&mut (*function).chunk) };
            reallocate(function, 1, 0, vm);
        }
        ObjType::Closure => {
            let closure = object_ptr.0 as *mut ObjClosure;
            unsafe {
                free_array(
                    (*closure).upvalues,
                    (*closure).upvalue_count,
                    (*closure).upvalue_count,
                    vm,
                );
            }
            reallocate(closure, 1, 0, vm);
        }
        ObjType::Native => {
            let native = object_ptr.0 as *mut ObjNative;
            reallocate(native, 1, 0, vm);
        }
        ObjType::Upvalue => {
            let upvalue = object_ptr.0 as *mut ObjUpvalue;
            reallocate(upvalue, 1, 0, vm);
        }
        ObjType::Class => {
            let class = object_ptr.0 as *mut ObjClass;
            unsafe { (*class).methods.free(vm) };
            reallocate(class, 1, 0, vm);
        }
        ObjType::Instance => {
            let instance = object_ptr.0 as *mut ObjInstance;
            unsafe { (*instance).fields.free(vm) };
            reallocate(instance, 1, 0, vm);
        }
        ObjType::BoundMethod => {
            let bound_method = object_ptr.0 as *mut ObjBoundMethod;
            reallocate(bound_method, 1, 0, vm);
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
