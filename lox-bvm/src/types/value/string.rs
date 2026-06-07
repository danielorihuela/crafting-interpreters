use std::ptr::copy_nonoverlapping;

use crate::collections::hashtable::HashTable;
use crate::memory::alloc::allocate;
use crate::memory::array::free_array;
use crate::types::value::Value;
use crate::types::{
    AsciiChar,
    value::obj::{Obj, ObjType, allocate_object},
};

#[repr(C)]
pub struct ObjString {
    obj: Obj,
    pub length: usize,
    pub chars: *mut AsciiChar,
    pub hash: u32,
}

impl ObjString {
    pub fn new(
        chars: *const AsciiChar,
        length: usize,
        objects: *mut *mut Obj,
        strings: *mut HashTable,
    ) -> *mut ObjString {
        let hash = hash_string(chars, length);
        let interned = unsafe { (*strings).find_string(chars, length, hash) };
        if let Some(interned) = interned {
            return interned;
        }

        let heap_chars = allocate::<AsciiChar>(length + 1);
        unsafe {
            copy_nonoverlapping(chars, heap_chars, length);
            heap_chars.add(length).write(0)
        };

        allocate_string(heap_chars, length, hash, objects, strings)
    }

    pub fn add(
        &mut self,
        rhs: *mut ObjString,
        objects: *mut *mut Obj,
        strings: *mut HashTable,
    ) -> *mut ObjString {
        let length = self.length + unsafe { (*rhs).length };
        let chars = allocate::<AsciiChar>(length + 1);

        unsafe {
            copy_nonoverlapping(self.chars, chars, self.length);
            copy_nonoverlapping((*rhs).chars, chars.add(self.length), (*rhs).length);
            chars.add(length).write(0);
        }

        take_string(chars, length, objects, strings)
    }
}

fn hash_string(chars: *const AsciiChar, length: usize) -> u32 {
    let mut hash: u32 = 2166136261;
    for i in 0..length {
        hash ^= unsafe { *chars.add(i) } as u32;
        hash = hash.wrapping_mul(16777619);
    }

    hash
}

fn allocate_string(
    chars: *mut AsciiChar,
    length: usize,
    hash: u32,
    objects: *mut *mut Obj,
    strings: *mut HashTable,
) -> *mut ObjString {
    let obj_string = allocate_object::<ObjString>(ObjType::String, objects);
    unsafe {
        (*obj_string).length = length;
        (*obj_string).chars = chars;
        (*obj_string).hash = hash;
        (*strings).set(obj_string, Value::from(()));
    }

    obj_string
}

fn take_string(
    chars: *mut AsciiChar,
    length: usize,
    objects: *mut *mut Obj,
    strings: *mut HashTable,
) -> *mut ObjString {
    let hash = hash_string(chars, length);
    let interned = unsafe { (*strings).find_string(chars, length, hash) };
    if let Some(interned) = interned {
        free_array(chars, length + 1, length + 1);
        return interned;
    }

    allocate_string(chars, length, hash, objects, strings)
}
