use crate::types::value::class::{ObjBoundMethod, ObjClass, ObjInstance};
use crate::types::value::closure::ObjClosure;
use crate::types::value::function::ObjFunction;
use crate::types::value::native::ObjNative;
use crate::types::value::upvalue::ObjUpvalue;

use crate::memory::heap::ObjId;
use crate::vm::VM;

pub enum HeapObj {
    String(String),
    Function(ObjFunction),
    Closure(ObjClosure),
    Native(ObjNative),
    Upvalue(ObjUpvalue),
    Class(ObjClass),
    Instance(ObjInstance),
    BoundMethod(ObjBoundMethod),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Obj {
    String(ObjId),
    Function(ObjId),
    Closure(ObjId),
    Native(ObjId),
    Upvalue(ObjId),
    Class(ObjId),
    Instance(ObjId),
    BoundMethod(ObjId),
}

impl PartialOrd for Obj {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        panic!("PartialOrd is not implemented for Obj");
    }
}

impl Obj {
    pub fn to_string(&self, vm: &VM) -> String {
        let id = match self {
            Obj::String(id) => id,
            Obj::Function(id) => id,
            Obj::Closure(id) => id,
            Obj::Native(id) => id,
            Obj::Upvalue(id) => id,
            Obj::Class(id) => id,
            Obj::Instance(id) => id,
            Obj::BoundMethod(id) => id,
        };

        match &vm.heap[*id] {
            HeapObj::String(s) => s.clone(),
            HeapObj::Function(f) => f.to_string(vm),
            HeapObj::Closure(c) => c.to_string(vm),
            HeapObj::Native(n) => n.to_string(),
            HeapObj::Upvalue(u) => u.to_string(),
            HeapObj::Class(c) => c.to_string(vm),
            HeapObj::Instance(i) => i.to_string(vm),
            HeapObj::BoundMethod(b) => b.to_string(vm),
        }
    }
}
