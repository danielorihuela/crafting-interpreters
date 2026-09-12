use crate::types::value::{
    ObjPtrTarget, Value,
    obj::{Obj, ObjType, allocate_object},
};
use std::fmt::Display;

pub type NativeFn = fn(usize, *mut Value) -> Value;

#[repr(C)]
pub struct ObjNative {
    obj: Obj,
    pub function: NativeFn,
}

impl ObjPtrTarget for ObjNative {}

impl Display for ObjNative {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn>")
    }
}

impl ObjNative {
    pub fn new(function: NativeFn, objects: *mut *mut Obj) -> *mut ObjNative {
        allocate_native(function, objects)
    }
}

fn allocate_native(function: NativeFn, objects: *mut *mut Obj) -> *mut ObjNative {
    let native = allocate_object::<ObjNative>(ObjType::Native, objects);

    unsafe {
        std::ptr::addr_of_mut!((*native).function).write(function);
    }

    native
}
