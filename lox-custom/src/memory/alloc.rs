use std::alloc::{Layout, alloc, dealloc, realloc};

use crate::memory::gc::GcCollector;

pub fn allocate<T>(size: usize, gc: &mut impl GcCollector) -> *mut T {
    reallocate(std::ptr::null_mut::<T>(), 0, size, gc)
}

pub fn reallocate<T>(
    ptr: *mut T,
    old_capacity: usize,
    new_capacity: usize,
    gc: &mut impl GcCollector,
) -> *mut T {
    reallocate_inner(ptr, old_capacity, new_capacity, true, gc)
}

pub fn reallocate_no_gc<T>(
    ptr: *mut T,
    old_capacity: usize,
    new_capacity: usize,
    gc: &mut impl GcCollector,
) -> *mut T {
    reallocate_inner(ptr, old_capacity, new_capacity, false, gc)
}

fn reallocate_inner<T>(
    ptr: *mut T,
    old_capacity: usize,
    new_capacity: usize,
    gc_enabled: bool,
    gc: &mut impl GcCollector,
) -> *mut T {
    let old_size = old_capacity.saturating_mul(std::mem::size_of::<T>());
    let new_size = new_capacity.saturating_mul(std::mem::size_of::<T>());

    let should_collect_now = {
        let mut context = gc.gc_context();
        context.charge_allocation(old_size, new_size);
        gc_enabled && new_capacity > old_capacity && context.should_collect_now()
    };

    if should_collect_now {
        gc.collect_garbage();
    }

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
