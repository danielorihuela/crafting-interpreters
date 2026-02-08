use std::alloc::{Layout, dealloc, realloc};

pub struct DynArray<T> {
    pub data: *mut T,
    pub capacity: usize,
    pub count: usize,
}

impl<T> Default for DynArray<T> {
    fn default() -> Self {
        DynArray {
            data: std::ptr::null_mut(),
            capacity: 0,
            count: 0,
        }
    }
}

impl<T> DynArray<T> {
    pub fn write(&mut self, data: T) {
        if self.capacity == self.count {
            let old_capacity = self.capacity;
            self.capacity = grow_capacity(old_capacity);
            self.data = grow_array::<T>(self.data, old_capacity, self.capacity);
        }

        unsafe { *self.data.add(self.count) = data };
        self.count += 1;
    }

    pub fn free(&mut self) {
        free_array(self.data, self.capacity);
        *self = Self::default();
    }

    pub fn get_at(&self, pos: usize) -> &T {
        unsafe { &*self.data.add(pos) }
    }
}

fn grow_capacity(capacity: usize) -> usize {
    if capacity == 0 { 8 } else { capacity * 2 }
}

fn grow_array<T>(ptr: *mut T, old_capacity: usize, new_capacity: usize) -> *mut T {
    let type_size = size_of::<T>();
    reallocate(ptr, type_size * old_capacity, type_size * new_capacity)
}

fn free_array<T>(ptr: *mut T, capacity: usize) {
    let _ = reallocate(ptr, capacity, 0);
}

fn reallocate<T>(ptr: *mut T, _old_capacity: usize, new_capacity: usize) -> *mut T {
    let ptr = ptr as *mut u8;
    let layout = Layout::new::<T>();
    if new_capacity == 0 {
        unsafe {
            dealloc(ptr, layout);
        }
        return std::ptr::null_mut();
    }

    let ret = unsafe { realloc(ptr, layout, new_capacity) } as *mut T;
    if ret.is_null() {
        panic!("Memory reallocation failed");
    }

    ret
}
