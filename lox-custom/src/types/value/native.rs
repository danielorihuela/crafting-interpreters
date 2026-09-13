use crate::{
    memory::heap::ObjId,
    types::value::{Value, obj::HeapObj},
    vm::VM,
};
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
    pub fn new(vm: &mut VM, function: NativeFn) -> ObjId {
        vm.bytes_allocated += std::mem::size_of::<HeapObj>();
        if vm.bytes_allocated > vm.next_gc {
            vm.garbage_collect();
        }

        vm.heap.allocate(HeapObj::Native(Self { function }))
    }
}
