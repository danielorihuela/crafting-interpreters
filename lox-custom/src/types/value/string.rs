use crate::memory::heap::ObjId;
use crate::types::value::Value;
use crate::types::value::obj::{HeapObj, Obj};
use crate::vm::VM;

pub struct ObjString(pub String);

impl ObjString {
    pub fn new(vm: &mut VM, data: &str) -> ObjId {
        allocate_string(vm, data)
    }

    pub fn add(&self, vm: &mut VM, rhs: &str) -> ObjId {
        allocate_string(vm, &format!("{}{}", self.0, rhs))
    }
}

pub fn allocate_string(vm: &mut VM, data: &str) -> ObjId {
    let interned = vm.strings.get(data);
    if let Some(interned) = interned {
        return *interned;
    }

    vm.bytes_allocated += std::mem::size_of::<HeapObj>();
    if vm.bytes_allocated > vm.next_gc {
        vm.garbage_collect();
    }

    let id = vm.heap.allocate(HeapObj::String(data.to_string()));
    vm.strings.insert(data.to_string(), id);

    vm.stack.push(Value::Obj(Obj::String(id)));
    vm.strings.insert(data.to_string(), id);
    vm.stack.pop();

    id
}
