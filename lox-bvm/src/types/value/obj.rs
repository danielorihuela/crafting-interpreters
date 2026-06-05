use std::ops::{Deref, DerefMut};

use crate::memory::alloc::reallocate;
use crate::memory::array::free_array;
use crate::types::value::string::ObjString;

#[derive(PartialEq)]
pub enum ObjType {
    String,
}

pub struct Obj {
    pub otype: ObjType,
    pub next: ObjPtr,
}

pub fn allocate_object<T>(obj_type: ObjType, objects: *mut *mut Obj) -> *mut T {
    let obj = reallocate(std::ptr::null_mut::<T>(), 0, size_of::<T>());
    let mut obj = ObjPtr(obj as *mut Obj);

    unsafe {
        obj.otype = obj_type;
        obj.next = ObjPtr(*objects);
        *objects = obj.0;
    };

    obj.0 as *mut T
}

pub unsafe fn free_object(object: impl Into<ObjPtr>) {
    let string = object.into().0 as *mut ObjString;
    unsafe {
        free_array((*string).chars, (*string).length + 1, (*string).length + 1);
    };
    reallocate(string, size_of::<ObjString>(), 0);
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
