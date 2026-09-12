use crate::{
    memory::{
        alloc::allocate,
        array::{free_array, grow_capacity},
        gc::GcCollector,
    },
    types::value::{Value, string::ObjString},
};

const MAX_LOAD: f32 = 0.75;

pub struct Entry {
    pub key: *mut ObjString,
    pub value: Value,
}

pub struct HashTable {
    count: usize,
    pub capacity: usize,
    pub entries: *mut Entry,
}

impl HashTable {
    pub fn new() -> Self {
        Self {
            count: 0,
            capacity: 0,
            entries: std::ptr::null_mut(),
        }
    }

    pub fn set(&mut self, key: *mut ObjString, value: Value, gc: &mut impl GcCollector) -> bool {
        if self.count + 1 > (self.capacity as f32 * MAX_LOAD) as usize {
            let new_capacity = grow_capacity(self.capacity);
            self.adjust_capacity(new_capacity, gc);
        }

        let entry = find_entry(self.entries, self.capacity, key);
        let is_new_key = unsafe { (*entry).key.is_null() };
        if is_new_key && unsafe { (*entry).value.is_nil() } {
            self.count += 1;
        }

        unsafe {
            (*entry).key = key;
            (*entry).value = value;
        }

        is_new_key
    }

    fn adjust_capacity(&mut self, capacity: usize, gc: &mut impl GcCollector) {
        let entries = allocate::<Entry>(capacity, gc);
        for i in 0..capacity {
            unsafe {
                entries.add(i).write(Entry {
                    key: std::ptr::null_mut(),
                    value: Value::from(()),
                })
            };
        }

        self.count = 0;
        for i in 0..self.capacity {
            let entry = unsafe { self.entries.add(i) };
            if unsafe { (*entry).key.is_null() } {
                continue;
            }

            let dest = find_entry(entries, capacity, unsafe { (*entry).key });
            unsafe {
                (*dest).key = (*entry).key;
                (*dest).value = std::mem::take(&mut (*entry).value);
            }
            self.count += 1;
        }

        free_array(self.entries, self.capacity, self.count, gc);

        self.entries = entries;
        self.capacity = capacity;
    }

    pub fn get(&self, key: *mut ObjString) -> Option<*const Value> {
        if self.count == 0 {
            return None;
        }

        let entry = find_entry(self.entries, self.capacity, key);
        if unsafe { (*entry).key.is_null() } {
            return None;
        }

        Some(unsafe { &(*entry).value })
    }

    pub fn find_string(&self, data: &str, hash: u32) -> Option<*mut ObjString> {
        if self.count == 0 {
            return None;
        }

        let mut index = hash & (self.capacity as u32 - 1);
        loop {
            let entry = unsafe { self.entries.add(index as usize) };
            if unsafe { (*entry).key.is_null() } {
                if unsafe { (*entry).value.is_nil() } {
                    return None;
                }
            } else if unsafe { (*(*entry).key).length } == data.len()
                && unsafe { (*(*entry).key).hash } == hash
                && unsafe {
                    std::slice::from_raw_parts((*(*entry).key).chars, (*(*entry).key).length)
                        == data.as_bytes()
                }
            {
                return Some(unsafe { (*entry).key });
            }

            index = (index + 1) & (self.capacity as u32 - 1);
        }
    }

    pub fn delete(&mut self, key: *mut ObjString) -> bool {
        if self.count == 0 {
            return false;
        }

        let entry = find_entry(self.entries, self.capacity, key);
        if unsafe { (*entry).key.is_null() } {
            return false;
        }

        unsafe {
            (*entry).key = std::ptr::null_mut();
            (*entry).value = Value::from(true);
        }

        true
    }

    pub fn add_all(&mut self, from: &HashTable, gc: &mut impl GcCollector) {
        for i in 0..from.capacity {
            let entry = unsafe { from.entries.add(i) };
            if unsafe { (*entry).key.is_null() } {
                continue;
            }
            self.set(unsafe { (*entry).key }, unsafe { (*entry).value.clone() }, gc);
        }
    }

    pub fn free(&mut self, gc: &mut impl GcCollector) {
        if self.entries.is_null() {
            return;
        }

        free_array(self.entries, self.capacity, self.count, gc);
        *self = Self::new();
    }
}

fn find_entry(entries: *mut Entry, capacity: usize, key: *mut ObjString) -> *mut Entry {
    let mut index = unsafe { (*key).hash } as usize & (capacity - 1);
    let mut tombstone = std::ptr::null_mut::<Entry>();

    loop {
        let entry = unsafe { entries.add(index) };
        if unsafe { (*entry).key.is_null() } {
            if unsafe { (*entry).value.is_nil() } {
                return if tombstone.is_null() {
                    entry
                } else {
                    tombstone
                };
            } else if tombstone.is_null() {
                tombstone = entry;
            }
        } else if unsafe { (*entry).key == key } {
            return entry;
        }

        index = (index + 1) & (capacity - 1);
    }
}
