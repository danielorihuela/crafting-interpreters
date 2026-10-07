use crate::types::value::class::{ObjBoundMethod, ObjClass, ObjInstance};
use crate::types::value::closure::ObjClosure;
use crate::types::value::function::ObjFunction;
use crate::types::value::native::ObjNative;
use crate::types::value::upvalue::ObjUpvalue;

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
