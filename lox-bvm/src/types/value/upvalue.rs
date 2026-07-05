use crate::types::value::{
    Value,
    obj::{Obj, ObjType, allocate_object},
};

#[repr(C)]
pub struct ObjUpvalue {
    obj: Obj,
    pub location: *mut Value,
    pub next: *mut ObjUpvalue,
    pub closed: Value,
}

impl ObjUpvalue {
    pub fn new(objects: *mut *mut Obj, slot: *mut Value) -> *mut ObjUpvalue {
        allocate_upvalue(objects, slot)
    }
}

fn allocate_upvalue(objects: *mut *mut Obj, slot: *mut Value) -> *mut ObjUpvalue {
    let upvalue = allocate_object::<ObjUpvalue>(ObjType::Upvalue, objects);

    unsafe {
        (*upvalue).location = slot;
        (*upvalue).next = std::ptr::null_mut();
        (*upvalue).closed = Value::from(());
    }

    upvalue
}
