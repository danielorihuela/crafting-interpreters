use std::fmt::Display;
use std::ptr::copy_nonoverlapping;

use crate::VM_INSTANCE;
use crate::collections::hashtable::HashTable;
use crate::memory::alloc::allocate;
use crate::types::value::{ObjPtrTarget, Value};
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

impl ObjPtrTarget for ObjString {}

impl Display for ObjString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let slice = unsafe { std::slice::from_raw_parts(self.chars, self.length) };
        let s = std::str::from_utf8(slice).map_err(|_| std::fmt::Error)?;
        write!(f, "{}", s)
    }
}

impl ObjString {
    pub fn new(data: &str, objects: *mut *mut Obj, strings: *mut HashTable) -> *mut ObjString {
        copy_string(data, objects, strings)
    }

    pub fn add(
        &mut self,
        rhs: *mut ObjString,
        objects: *mut *mut Obj,
        strings: *mut HashTable,
    ) -> *mut ObjString {
        let lhs = unsafe { std::slice::from_raw_parts(self.chars, self.length) };
        let rhs_len = unsafe { (*rhs).length };
        let rhs_bytes = unsafe { std::slice::from_raw_parts((*rhs).chars, rhs_len) };

        let mut merged = String::with_capacity(self.length + rhs_len);
        merged.push_str(unsafe { std::str::from_utf8_unchecked(lhs) });
        merged.push_str(unsafe { std::str::from_utf8_unchecked(rhs_bytes) });

        take_string(merged, objects, strings)
    }
}

fn hash_string(data: &str) -> u32 {
    let mut hash: u32 = 2166136261;
    for i in 0..data.len() {
        hash ^= data.as_bytes()[i] as u32;
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

        if !VM_INSTANCE.is_null() {
            (*VM_INSTANCE).stack.push(Value::from(obj_string));
            (*strings).set(obj_string, Value::from(()));
            (*VM_INSTANCE).stack.pop();
        }

        if (*strings).get(obj_string).is_none() {
            (*strings).set(obj_string, Value::from(()));
        }
    }

    obj_string
}

pub fn copy_string(data: &str, objects: *mut *mut Obj, strings: *mut HashTable) -> *mut ObjString {
    let hash = hash_string(data);
    let interned = unsafe { (*strings).find_string(data, hash) };
    if let Some(interned) = interned {
        return interned;
    }

    let bytes = data.as_bytes();
    let chars = allocate::<AsciiChar>(bytes.len() + 1);
    unsafe {
        copy_nonoverlapping(bytes.as_ptr() as *const AsciiChar, chars, bytes.len());
        chars.add(bytes.len()).write(0);
    }

    allocate_string(chars, bytes.len(), hash, objects, strings)
}

fn take_string(data: String, objects: *mut *mut Obj, strings: *mut HashTable) -> *mut ObjString {
    let bytes = data.into_bytes();
    let hash = hash_string(std::str::from_utf8(&bytes).expect("string literals must be utf-8"));
    let interned = unsafe {
        (*strings).find_string(
            std::str::from_utf8(&bytes).expect("string literals must be utf-8"),
            hash,
        )
    };
    if let Some(interned) = interned {
        return interned;
    }

    let chars = allocate::<AsciiChar>(bytes.len() + 1);
    unsafe {
        copy_nonoverlapping(bytes.as_ptr() as *const AsciiChar, chars, bytes.len());
        chars.add(bytes.len()).write(0);
    }

    allocate_string(chars, bytes.len(), hash, objects, strings)
}
