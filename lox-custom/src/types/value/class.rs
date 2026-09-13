use std::collections::HashMap;

use crate::{
    memory::heap::ObjId,
    types::value::{Value, obj::HeapObj},
    vm::VM,
};

pub struct ObjClass {
    pub name: ObjId,
    pub methods: HashMap<ObjId, Value>,
}

impl ObjClass {
    pub fn new(name: ObjId, vm: &mut VM) -> ObjId {
        vm.bytes_allocated += std::mem::size_of::<HeapObj>();
        if vm.bytes_allocated > vm.next_gc {
            vm.garbage_collect();
        }

        vm.heap.allocate(HeapObj::Class(Self {
            name,
            methods: HashMap::new(),
        }))
    }

    pub fn to_string(&self, vm: &VM) -> String {
        let HeapObj::String(name) = &vm.heap[self.name] else {
            panic!("Expected ObjString for class name");
        };

        name.clone()
    }
}

pub struct ObjInstance {
    pub class: ObjId,
    pub fields: HashMap<ObjId, Value>,
}

impl ObjInstance {
    pub fn new(class: ObjId, vm: &mut VM) -> ObjId {
        vm.bytes_allocated += std::mem::size_of::<HeapObj>();
        if vm.bytes_allocated > vm.next_gc {
            vm.garbage_collect();
        }

        vm.heap.allocate(HeapObj::Instance(Self {
            class,
            fields: HashMap::new(),
        }))
    }

    pub fn to_string(&self, vm: &VM) -> String {
        let HeapObj::Class(class) = &vm.heap[self.class] else {
            panic!("Expected ObjClass for instance's class");
        };

        format!("{} instance", class.to_string(vm))
    }
}

pub struct ObjBoundMethod {
    pub receiver: Value,
    pub method: ObjId,
}

impl ObjBoundMethod {
    pub fn new(receiver: Value, method: ObjId, vm: &mut VM) -> ObjId {
        vm.bytes_allocated += std::mem::size_of::<HeapObj>();
        if vm.bytes_allocated > vm.next_gc {
            vm.garbage_collect();
        }

        vm.heap
            .allocate(HeapObj::BoundMethod(Self { receiver, method }))
    }

    pub fn to_string(&self, vm: &VM) -> String {
        let HeapObj::Closure(closure) = &vm.heap[self.method] else {
            panic!("Expected closure for method");
        };

        closure.to_string(vm)
    }
}
