use crate::{heap::ObjId, types::chunk::Chunk, vm::VM};

pub struct ObjFunction {
    pub arity: usize,
    pub chunk: Chunk,
    pub name: ObjId,
    pub upvalue_count: usize,
}

impl ObjFunction {
    pub fn new() -> Self {
        Self {
            arity: 0,
            chunk: Chunk::default(),
            name: ObjId::null(),
            upvalue_count: 0,
        }
    }

    pub fn to_string(&self, vm: &VM) -> String {
        if self.name.is_null() {
            "<script>".to_string()
        } else {
            let name = vm.heap.string(self.name);
            format!("<fn {}>", name)
        }
    }
}
