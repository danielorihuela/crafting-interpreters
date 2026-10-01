use crate::{heap::ObjId, types::value::obj::HeapObj, vm::VM};

pub struct ObjClosure {
    pub function_id: ObjId,
    pub upvalues: Vec<ObjId>,
}

impl ObjClosure {
    pub fn new(function_id: ObjId, upvalue_count: usize) -> Self {
        Self {
            function_id,
            upvalues: vec![ObjId::null(); upvalue_count],
        }
    }

    pub fn to_string(&self, vm: &VM) -> String {
        if self.function_id.is_null() {
            "<script>".to_string()
        } else {
            let HeapObj::Function(function) = &vm.heap[self.function_id] else {
                panic!("Expected a function object");
            };
            function.to_string(vm)
        }
    }
}
