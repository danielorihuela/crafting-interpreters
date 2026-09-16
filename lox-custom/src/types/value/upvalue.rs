use crate::{
    memory::heap::ObjId,
    types::value::{Value, obj::HeapObj},
    vm::VM,
};
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
    pub fn new(slot: isize, vm: &mut VM) -> ObjId {
        vm.bytes_allocated += std::mem::size_of::<HeapObj>();
        if vm.bytes_allocated > vm.next_gc {
            vm.garbage_collect();
        }

        vm.heap.allocate(HeapObj::Upvalue(Self {
            location: slot,
            next: ObjId::null(),
            closed: Value::Nil,
        }))
    }
}
