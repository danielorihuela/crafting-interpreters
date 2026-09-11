use crate::{
    memory::array::grow_array,
    types::value::{
        ObjPtrTarget,
        function::ObjFunction,
        obj::{Obj, ObjType, allocate_object},
        upvalue::ObjUpvalue,
    },
};
use std::fmt::Display;

#[repr(C)]
pub struct ObjClosure {
    obj: Obj,
    pub function: *mut ObjFunction,
    pub upvalues: *mut *mut ObjUpvalue,
    pub upvalue_count: usize,
}

impl ObjPtrTarget for ObjClosure {}

impl Display for ObjClosure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.function.is_null() {
            write!(f, "<script>")
        } else {
            let function = unsafe { &*self.function };
            write!(f, "{}", function)
        }
    }
}

impl ObjClosure {
    pub fn new(objects: *mut *mut Obj, function: *mut ObjFunction) -> *mut ObjClosure {
        let count = unsafe { (*function).upvalue_count };

        let upvalues = std::ptr::null_mut::<*mut ObjUpvalue>();
        if count == 0 {
            return allocate_closure(objects, function, upvalues);
        }

        let upvalues = grow_array(upvalues, 0, count);

        for i in 0..count {
            unsafe {
                upvalues.add(i).write(std::ptr::null_mut());
            }
        }

        allocate_closure(objects, function, upvalues)
    }
}

fn allocate_closure(
    objects: *mut *mut Obj,
    function: *mut ObjFunction,
    upvalues: *mut *mut ObjUpvalue,
) -> *mut ObjClosure {
    let closure = allocate_object::<ObjClosure>(ObjType::Closure, objects);

    unsafe {
        (*closure).function = function;
        (*closure).upvalues = upvalues;
        (*closure).upvalue_count = (*function).upvalue_count;
    }

    closure
}
