use crate::types::value::class::{ObjBoundMethod, ObjClass, ObjInstance};
use crate::types::value::closure::ObjClosure;
use crate::types::value::function::ObjFunction;
use crate::types::value::native::ObjNative;
use crate::types::value::upvalue::ObjUpvalue;

use crate::heap::ObjId;
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

#[derive(Debug, Copy, Clone, PartialEq)]
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
        match self {
            Obj::String(id) => vm.heap.string(*id).clone(),
            Obj::Function(id) => vm.heap.function(*id).to_string(vm),
            Obj::Closure(id) => vm.heap.closure(*id).to_string(vm),
            Obj::Native(id) => vm.heap.native(*id).to_string(),
            Obj::Upvalue(id) => vm.heap.upvalue(*id).to_string(),
            Obj::Class(id) => vm.heap.class(*id).to_string(vm),
            Obj::Instance(id) => vm.heap.instance(*id).to_string(vm),
            Obj::BoundMethod(id) => vm.heap.bound_method(*id).to_string(vm),
        }
    }
}
