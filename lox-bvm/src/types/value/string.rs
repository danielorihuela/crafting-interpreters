use std::ptr::copy_nonoverlapping;

use crate::memory::alloc::allocate;
use crate::types::{
    AsciiChar,
    value::obj::{Obj, ObjType, allocate_object},
};

#[repr(C)]
pub struct ObjString {
    obj: Obj,
    pub length: usize,
    pub chars: *mut AsciiChar,
}

impl ObjString {
    pub fn new(chars: *const AsciiChar, length: usize, objects: *mut *mut Obj) -> *mut ObjString {
        let heap_chars = allocate::<AsciiChar>(length + 1);
        unsafe {
            copy_nonoverlapping(chars, heap_chars, length);
            heap_chars.add(length).write(0)
        };

        allocate_string(heap_chars, length, objects)
    }

    pub fn add(&mut self, rhs: *mut ObjString, objects: *mut *mut Obj) -> *mut ObjString {
        let length = self.length + unsafe { (*rhs).length };
        let chars = allocate::<AsciiChar>(length + 1);

        unsafe {
            copy_nonoverlapping(self.chars, chars, self.length);
            copy_nonoverlapping((*rhs).chars, chars.add(self.length), (*rhs).length);
            chars.add(length).write(0);
        }

        allocate_string(chars, length, objects)
    }
}

fn allocate_string(chars: *mut AsciiChar, length: usize, objects: *mut *mut Obj) -> *mut ObjString {
    let obj_string = allocate_object::<ObjString>(ObjType::String, objects);
    unsafe {
        (*obj_string).length = length;
        (*obj_string).chars = chars;
    }

    obj_string
}
