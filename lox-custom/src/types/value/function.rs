use std::fmt::Display;

use crate::types::{
    chunk::Chunk,
    value::{
        ObjPtrTarget,
        obj::{Obj, ObjType, allocate_object},
        string::ObjString,
    },
};

#[repr(C)]
pub struct ObjFunction {
    obj: Obj,
    pub arity: usize,
    pub chunk: Chunk,
    pub name: *mut ObjString,
    pub upvalue_count: usize,
}

impl ObjPtrTarget for ObjFunction {}

impl Display for ObjFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.name.is_null() {
            write!(f, "<script>")
        } else {
            let name = unsafe { &*self.name };
            write!(f, "<fn {}>", name)
        }
    }
}

impl ObjFunction {
    pub fn new(objects: *mut *mut Obj) -> *mut ObjFunction {
        allocate_function(objects)
    }
}

fn allocate_function(objects: *mut *mut Obj) -> *mut ObjFunction {
    let function = allocate_object::<ObjFunction>(ObjType::Function, objects);

    unsafe {
        std::ptr::addr_of_mut!((*function).arity).write(0);
        std::ptr::addr_of_mut!((*function).chunk).write(Chunk::default());
        std::ptr::addr_of_mut!((*function).name).write(std::ptr::null_mut());
        std::ptr::addr_of_mut!((*function).upvalue_count).write(0);
    }

    function
}
