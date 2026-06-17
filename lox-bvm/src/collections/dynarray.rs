use std::{
    fmt::Debug,
    ops::{Index, IndexMut},
};

use crate::memory::array::{free_array, grow_array, grow_capacity};

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
        free_array(self.data, self.capacity, self.count);
    }
}

impl<T> Index<usize> for DynArray<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &*self.data.add(index) }
    }
}

impl<T> IndexMut<usize> for DynArray<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe { &mut *self.data.add(index) }
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

    fn assert_dynarray_works_with_type<T>()
    where
        T: Default + Copy + Debug + PartialEq + From<u8>,
    {
        let mut array = DynArray::<T>::default();
        for i in 0..8u8 {
            array.write(T::from(i));
            assert_eq!(array[i as usize], T::from(i));
            assert_eq!(array.count, (i + 1) as usize);
            assert_eq!(array.capacity, 8);
        }

        array.write(T::from(9));
        assert_eq!(array[8], T::from(9));
        assert_eq!(array.count, 9);
        assert_eq!(array.capacity, 16);

        let start = array.data as *const u8;
        let end = unsafe { start.add(array.count * std::mem::size_of::<T>()) };
        assert_eq!(
            unsafe { *(end.sub(std::mem::size_of::<T>()) as *const T) },
            T::from(9)
        );

        let size = unsafe { end.offset_from(start) };
        assert_eq!(size, (array.count * std::mem::size_of::<T>()) as isize);
    }

    #[test]
    fn test_dynarray_for_type() {
        assert_dynarray_works_with_type::<u8>();
        assert_dynarray_works_with_type::<u16>();
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
