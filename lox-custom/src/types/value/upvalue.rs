use crate::{memory::heap::ObjId, types::value::Value};
use std::fmt::Display;

pub struct ObjUpvalue {
    pub location: isize,
    pub next: ObjId,
    pub closed: Value,
}

impl Display for ObjUpvalue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<upvalue>")
    }
}

impl ObjUpvalue {
    pub fn new(slot: isize) -> Self {
        Self {
            location: slot,
            next: ObjId::null(),
            closed: Value::Nil,
        }
    }
}
