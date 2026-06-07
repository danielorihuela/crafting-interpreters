use crate::{
    memory::{
        alloc::allocate,
        array::{free_array, grow_capacity},
    },
    types::{
        AsciiChar,
        value::{Value, string::ObjString},
    },
};

const MAX_LOAD: f32 = 0.75;

struct Entry {
    key: *mut ObjString,
    value: Value,
}

pub struct HashTable {
    count: usize,
    capacity: usize,
    entries: *mut Entry,
}

impl HashTable {
    pub fn new() -> Self {
        Self {
            count: 0,
            capacity: 0,
            entries: std::ptr::null_mut(),
        }
    }

    pub fn set(&mut self, key: *mut ObjString, value: Value) -> bool {
        if self.count + 1 > (self.capacity as f32 * MAX_LOAD) as usize {
            let new_capacity = grow_capacity(self.capacity);
            self.adjust_capacity(new_capacity);
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

    fn adjust_capacity(&mut self, capacity: usize) {
        let entries = allocate::<Entry>(capacity);
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

        free_array(self.entries, self.capacity, self.count);

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

    pub fn find_string(
        &self,
        chars: *const AsciiChar,
        length: usize,
        hash: u32,
    ) -> Option<*mut ObjString> {
        if self.count == 0 {
            return None;
        }

        let mut index = hash % self.capacity as u32;
        loop {
            let entry = unsafe { self.entries.add(index as usize) };
            if unsafe { (*entry).key.is_null() } {
                if unsafe { (*entry).value.is_nil() } {
                    return None;
                }
            } else if unsafe { (*(*entry).key).length } == length
                && unsafe { (*(*entry).key).hash } == hash
                && unsafe {
                    std::slice::from_raw_parts((*(*entry).key).chars, (*(*entry).key).length)
                        == std::slice::from_raw_parts(chars, length)
                }
            {
                return Some(unsafe { (*entry).key });
            }

            index = (index + 1) % self.capacity as u32;
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

    fn add_all(&mut self, from: &HashTable) {
        for i in 0..from.capacity {
            let entry = unsafe { from.entries.add(i) };
            if unsafe { (*entry).key.is_null() } {
                continue;
            }
            self.set(unsafe { (*entry).key }, unsafe {
                std::mem::take(&mut (*entry).value)
            });
        }
    }

    pub fn free(&mut self) {
        if self.entries.is_null() {
            return;
        }

        free_array(self.entries, self.capacity, self.count);
        *self = Self::new();
    }
}

fn find_entry(entries: *mut Entry, capacity: usize, key: *mut ObjString) -> *mut Entry {
    let mut index = unsafe { (*key).hash } as usize % capacity;
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

        index = (index + 1) % capacity;
    }
}
