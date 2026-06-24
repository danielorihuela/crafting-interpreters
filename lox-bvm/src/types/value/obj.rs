use std::ops::{Deref, DerefMut};
use std::ptr::drop_in_place;

use crate::memory::alloc::reallocate;
use crate::memory::array::free_array;
use crate::types::value::function::ObjFunction;
use crate::types::value::native::ObjNative;
use crate::types::value::string::ObjString;

#[derive(PartialEq)]
pub enum ObjType {
    String,
    Function,
    Native,
}

pub struct Obj {
    pub otype: ObjType,
    pub next: ObjPtr,
}

pub fn allocate_object<T>(obj_type: ObjType, objects: *mut *mut Obj) -> *mut T {
    let obj = reallocate(std::ptr::null_mut::<T>(), 0, 1);
    let mut obj = ObjPtr(obj as *mut Obj);

    unsafe {
        obj.otype = obj_type;
        obj.next = ObjPtr(*objects);
        *objects = obj.0;
    };

    obj.0 as *mut T
}

pub unsafe fn free_object(object: impl Into<ObjPtr>) {
    let object_ptr = object.into();
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
        ObjType::Native => {
            let native = object_ptr.0 as *mut ObjNative;
            reallocate(native, 1, 0);
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
