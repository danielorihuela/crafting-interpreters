use crate::{memory::heap::ObjId, types::value::obj::HeapObj, vm::VM};

pub struct ObjClosure {
    pub function_id: ObjId,
    pub upvalues: Vec<ObjId>,
}

impl ObjClosure {
    pub fn new(function_id: ObjId, vm: &mut VM) -> ObjId {
        vm.bytes_allocated += std::mem::size_of::<HeapObj>();
        if vm.bytes_allocated > vm.next_gc {
            vm.garbage_collect();
        }

        let HeapObj::Function(function) = &vm.heap[function_id] else {
            panic!("Expected a function object");
        };
        let upvalues = vec![ObjId::null(); function.upvalue_count];
        vm.heap.allocate(HeapObj::Closure(ObjClosure {
            function_id,
            upvalues,
        }))
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
