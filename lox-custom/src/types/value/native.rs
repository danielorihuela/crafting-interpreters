use crate::types::value::Value;
use std::fmt::Display;

pub type NativeFn = fn(usize, *mut Value) -> Value;

pub struct ObjNative {
    pub function: NativeFn,
}

impl Display for ObjNative {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn>")
    }
}

impl ObjNative {
    pub fn new(function: NativeFn) -> Self {
        Self { function }
    }
}
