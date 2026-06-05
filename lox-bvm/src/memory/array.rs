use super::alloc::reallocate;

pub fn grow_capacity(capacity: usize) -> usize {
    if capacity == 0 { 8 } else { capacity * 2 }
}

pub fn grow_array<T>(ptr: *mut T, old_capacity: usize, new_capacity: usize) -> *mut T {
    reallocate(ptr, old_capacity, new_capacity)
}

pub fn free_array<T>(ptr: *mut T, capacity: usize, length: usize) {
    if ptr.is_null() {
        return;
    }

    unsafe { std::ptr::drop_in_place(std::ptr::slice_from_raw_parts_mut(ptr, length)) }
    let _ = reallocate(ptr, capacity, 0);
}
