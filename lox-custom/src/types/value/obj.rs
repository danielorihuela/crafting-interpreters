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

impl From<String> for HeapObj {
    fn from(s: String) -> Self {
        HeapObj::String(s)
    }
}

impl From<ObjFunction> for HeapObj {
    fn from(f: ObjFunction) -> Self {
        HeapObj::Function(f)
    }
}

impl From<ObjClosure> for HeapObj {
    fn from(c: ObjClosure) -> Self {
        HeapObj::Closure(c)
    }
}

impl From<ObjNative> for HeapObj {
    fn from(n: ObjNative) -> Self {
        HeapObj::Native(n)
    }
}

impl From<ObjUpvalue> for HeapObj {
    fn from(u: ObjUpvalue) -> Self {
        HeapObj::Upvalue(u)
    }
}

impl From<ObjClass> for HeapObj {
    fn from(c: ObjClass) -> Self {
        HeapObj::Class(c)
    }
}

impl From<ObjInstance> for HeapObj {
    fn from(i: ObjInstance) -> Self {
        HeapObj::Instance(i)
    }
}

impl From<ObjBoundMethod> for HeapObj {
    fn from(b: ObjBoundMethod) -> Self {
        HeapObj::BoundMethod(b)
    }
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
