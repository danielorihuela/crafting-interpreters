use std::{
    alloc::{Layout, alloc, dealloc, realloc},
    fmt::Debug,
    ops::Index,
};

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

        unsafe { self.data.add(self.count).write(data) };
        self.count += 1;
    }
}

impl<T> Drop for DynArray<T> {
    fn drop(&mut self) {
        if !self.data.is_null() {
            unsafe {
                std::ptr::drop_in_place(std::ptr::slice_from_raw_parts_mut(self.data, self.count));
            }
            free_array(self.data, self.capacity);
        }
    }
}

fn grow_capacity(capacity: usize) -> usize {
    if capacity == 0 { 8 } else { capacity * 2 }
}

fn grow_array<T>(ptr: *mut T, old_capacity: usize, new_capacity: usize) -> *mut T {
    reallocate(ptr, old_capacity, new_capacity)
}

fn free_array<T>(ptr: *mut T, capacity: usize) {
    let _ = reallocate(ptr, capacity, 0);
}

fn reallocate<T>(ptr: *mut T, old_capacity: usize, new_capacity: usize) -> *mut T {
    let new_layout = Layout::array::<T>(new_capacity).unwrap();
    if old_capacity == 0 {
        return unsafe { alloc(new_layout) } as *mut T;
    }

    let old_layout = Layout::array::<T>(old_capacity).unwrap();
    if new_capacity == 0 {
        unsafe {
            dealloc(ptr as *mut u8, old_layout);
        }
        return std::ptr::null_mut();
    };

    let ret = unsafe { realloc(ptr as *mut u8, old_layout, new_layout.size()) };
    if ret.is_null() {
        panic!("Memory reallocation failed");
    }

    ret as *mut T
}

impl<T> Index<usize> for DynArray<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &*self.data.add(index) }
    }
}

impl<T: Debug> std::fmt::Debug for DynArray<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let values = unsafe { std::slice::from_raw_parts(self.data, self.count) };
        write!(f, "{:?}", values)
    }
}

impl<T: PartialEq> Eq for DynArray<T> {}
impl<T: PartialEq> PartialEq for DynArray<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.count != other.count {
            return false;
        }
        let self_slice = unsafe { std::slice::from_raw_parts(self.data, self.count) };
        let other_slice = unsafe { std::slice::from_raw_parts(other.data, other.count) };
        self_slice == other_slice
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynarray_single_byte() {
        let mut array = DynArray::<u8>::default();
        assert_eq!(array.count, 0);
        assert_eq!(array.capacity, 0);
        assert_eq!(array.data, std::ptr::null_mut());

        for i in 0..8 {
            array.write(i);
            assert_eq!(array[i.into()], i);
            assert_eq!(array.count, (i + 1).into());
            assert_eq!(array.capacity, 8);
        }

        array.write(9);
        assert_eq!(array[8], 9);
        assert_eq!(array.count, 9);
        assert_eq!(array.capacity, 16);

        let start = array.data;
        let end = unsafe { start.add(array.count) };
        assert_eq!(unsafe { *end.sub(1) }, 9);

        let size = unsafe { end.offset_from(start) };
        assert_eq!(size, array.count as isize);
    }

    #[test]
    fn test_dynarray_two_bytes() {
        let mut array = DynArray::<u16>::default();
        for i in 0..8 {
            array.write(i);
            assert_eq!(array[i.into()], i);
            assert_eq!(array.count, (i + 1).into());
            assert_eq!(array.capacity, 8);
        }

        array.write(9);
        assert_eq!(array[8], 9);
        assert_eq!(array.count, 9);
        assert_eq!(array.capacity, 16);

        let start = array.data as *const u8;
        let end = unsafe { start.add(array.count * size_of::<u16>()) };
        assert_eq!(unsafe { *(end.sub(2) as *const u16) }, 9);

        let size = unsafe { end.offset_from(start) };
        assert_eq!(size, (array.count * size_of::<u16>()) as isize);
    }

    #[test]
    fn test_dynarray_eq() {
        let mut array1 = DynArray::default();
        let mut array2 = DynArray::default();

        for i in 0..8 {
            array1.write(i);
            array2.write(i);
        }

        assert_eq!(array1, array2);

        array1.write(8);
        assert_ne!(array1, array2);
    }

    // Checks memory is not leaked for types with Drop (like String) using Miri
    #[test]
    fn test_dynarray_string() {
        let mut array = DynArray::<String>::default();
        array.write(String::from("hello"));
    }
}
