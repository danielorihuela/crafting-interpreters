use crate::collections::dynarray::DynArray;

pub type Value = f64;

#[derive(Default)]
pub struct Values {
    dyn_array: DynArray<Value>,
}

impl Values {
    pub fn write(&mut self, value: Value) {
        self.dyn_array.write(value);
    }

    pub fn free(&mut self) {
        self.dyn_array.free();
    }

    pub fn count(&self) -> usize {
        self.dyn_array.count
    }

    pub fn get_at(&self, pos: usize) -> Value {
        *self.dyn_array.get_at(pos)
    }
}
