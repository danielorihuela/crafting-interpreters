use crate::{
    types::value::{
        ObjPtrTarget, Value,
        obj::{Obj, ObjType, allocate_object},
    },
    vm::VM,
};
use std::fmt::Display;

#[repr(C)]
pub struct ObjUpvalue {
    obj: Obj,
    pub location: *mut Value,
    pub next: *mut ObjUpvalue,
    pub closed: Value,
}

impl ObjPtrTarget for ObjUpvalue {}

impl Display for ObjUpvalue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<upvalue>")
    }
}

impl ObjUpvalue {
    pub fn new(objects: *mut *mut Obj, slot: *mut Value, vm: &mut VM) -> *mut ObjUpvalue {
        allocate_upvalue(objects, slot, vm)
    }
}

fn allocate_upvalue(objects: *mut *mut Obj, slot: *mut Value, vm: &mut VM) -> *mut ObjUpvalue {
    let upvalue = allocate_object::<ObjUpvalue>(ObjType::Upvalue, objects, vm);

    unsafe {
        (*upvalue).location = slot;
        (*upvalue).next = std::ptr::null_mut();
        (*upvalue).closed = Value::Nil;
    }

    upvalue
}
