use crate::types::{
    chunk::Chunk,
    value::{
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
    }

    function
}
