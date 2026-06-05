use std::alloc::{Layout, alloc, dealloc, realloc};

pub fn allocate<T>(size: usize) -> *mut T {
    reallocate(std::ptr::null_mut::<T>(), 0, size)
}

pub fn reallocate<T>(ptr: *mut T, old_capacity: usize, new_capacity: usize) -> *mut T {
    let new_layout = Layout::array::<T>(new_capacity).unwrap();
    if old_capacity == 0 {
        return unsafe { alloc(new_layout) } as *mut T;
    }

    let old_layout = Layout::array::<T>(old_capacity).unwrap();
    if new_capacity == 0 {
        unsafe { dealloc(ptr as *mut u8, old_layout) }
        return std::ptr::null_mut();
    };

    let ret = unsafe { realloc(ptr as *mut u8, old_layout, new_layout.size()) };
    if ret.is_null() {
        panic!("Memory reallocation failed");
    }

    ret as *mut T
}
