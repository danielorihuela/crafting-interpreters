use std::ops::Index;

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

    pub fn count(&self) -> usize {
        self.dyn_array.count
    }
}

impl Index<usize> for Values {
    type Output = Value;

    fn index(&self, index: usize) -> &Self::Output {
        &self.dyn_array[index]
    }
}
