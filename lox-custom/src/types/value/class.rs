use crate::{HashMap, heap::ObjId, types::value::Value, vm::VM};

pub struct ObjClass {
    pub name: ObjId,
    pub methods: HashMap<ObjId, Value>,
}

impl ObjClass {
    pub fn new(name: ObjId) -> Self {
        Self {
            name,
            methods: HashMap::default(),
        }
    }

    pub fn to_string(&self, vm: &VM) -> String {
        vm.heap.string(self.name).clone()
    }
}

pub struct ObjInstance {
    pub class: ObjId,
    pub fields: HashMap<ObjId, Value>,
}

impl ObjInstance {
    pub fn new(class: ObjId) -> Self {
        Self {
            class,
            fields: HashMap::default(),
        }
    }

    pub fn to_string(&self, vm: &VM) -> String {
        let class = vm.heap.class(self.class);
        format!("{} instance", class.to_string(vm))
    }
}

pub struct ObjBoundMethod {
    pub receiver: Value,
    pub method: ObjId,
}

impl ObjBoundMethod {
    pub fn new(receiver: Value, method: ObjId) -> Self {
        Self { receiver, method }
    }

    pub fn to_string(&self, vm: &VM) -> String {
        let closure = vm.heap.closure(self.method);
        closure.to_string(vm)
    }
}
