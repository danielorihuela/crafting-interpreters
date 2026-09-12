use std::alloc::{Layout, alloc, dealloc, realloc};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{DEBUG_STRESS_GC, VM_INSTANCE, garbage_collect, vm_is_ready};

static BOOTSTRAP_BYTES_ALLOCATED: AtomicUsize = AtomicUsize::new(0);

fn adjust_bytes_allocated(old_size: usize, new_size: usize) {
    unsafe {
        if VM_INSTANCE.is_null() {
            if new_size >= old_size {
                BOOTSTRAP_BYTES_ALLOCATED.fetch_add(new_size - old_size, Ordering::Relaxed);
            } else {
                BOOTSTRAP_BYTES_ALLOCATED.fetch_sub(old_size - new_size, Ordering::Relaxed);
            }
            return;
        }

        if new_size >= old_size {
            (*VM_INSTANCE).bytes_allocated += new_size - old_size;
        } else {
            (*VM_INSTANCE).bytes_allocated -= old_size - new_size;
        }
    }
}

pub fn take_bootstrap_bytes_allocated() -> usize {
    BOOTSTRAP_BYTES_ALLOCATED.swap(0, Ordering::Relaxed)
}

pub fn allocate<T>(size: usize) -> *mut T {
    reallocate(std::ptr::null_mut::<T>(), 0, size)
}

pub fn reallocate<T>(ptr: *mut T, old_capacity: usize, new_capacity: usize) -> *mut T {
    reallocate_inner(ptr, old_capacity, new_capacity, true)
}

pub fn reallocate_no_gc<T>(ptr: *mut T, old_capacity: usize, new_capacity: usize) -> *mut T {
    reallocate_inner(ptr, old_capacity, new_capacity, false)
}

fn reallocate_inner<T>(
    ptr: *mut T,
    old_capacity: usize,
    new_capacity: usize,
    gc_enabled: bool,
) -> *mut T {
    let old_size = old_capacity.saturating_mul(std::mem::size_of::<T>());
    let new_size = new_capacity.saturating_mul(std::mem::size_of::<T>());

    adjust_bytes_allocated(old_size, new_size);

    if gc_enabled {
        if new_capacity > old_capacity && vm_is_ready() {
            if DEBUG_STRESS_GC {
                garbage_collect();
            }

            if unsafe { (*VM_INSTANCE).bytes_allocated > (*VM_INSTANCE).next_gc } {
                garbage_collect();
            }
        }
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
