use std::ops::{Index, IndexMut};

use crate::{DEBUG_LOG_GC, types::value::obj::HeapObj};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ObjId(usize);

impl ObjId {
    pub fn null() -> Self {
        ObjId(usize::MAX)
    }

    pub fn is_null(&self) -> bool {
        self.0 == usize::MAX
    }
}

impl std::fmt::Display for ObjId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ObjId({})", self.0)
    }
}

pub struct Heap {
    objects: Vec<Option<Slot>>,
    free_list: Vec<usize>,
}

pub struct Slot {
    object: HeapObj,
    is_marked: bool,
}

impl Heap {
    pub fn new() -> Self {
        Heap {
            objects: Vec::new(),
            free_list: Vec::new(),
        }
    }

    pub fn allocate(&mut self, object: HeapObj) -> ObjId {
        let slot = Slot {
            object,
            is_marked: false,
        };

        if let Some(index) = self.free_list.pop() {
            self.objects[index] = Some(slot);
            return ObjId(index);
        }

        self.objects.push(Some(slot));

        let id = self.objects.len() - 1;

        if DEBUG_LOG_GC {
            println!(
                "{id} allocate {:?} for HeapObj",
                std::mem::size_of::<HeapObj>(),
            );
        }

        ObjId(id)
    }

    pub fn deallocate(&mut self, id: ObjId) {
        if id.0 < self.objects.len() && self.objects[id.0].is_some() {
            self.objects[id.0] = None;
            self.free_list.push(id.0);
        }

        if DEBUG_LOG_GC {
            println!(
                "{id} deallocate {:?} for HeapObj",
                std::mem::size_of::<HeapObj>(),
            );
        }
    }

    pub fn clear(&mut self) {
        self.objects.clear();
        self.free_list.clear();
    }

    pub fn contains(&self, id: ObjId) -> bool {
        id.0 < self.objects.len() && self.objects[id.0].is_some()
    }

    pub fn mark(&mut self, id: ObjId) -> bool {
        if !self.contains(id) {
            return false;
        }

        let slot = self.objects[id.0].as_mut().expect("slot should exist");
        if slot.is_marked {
            false
        } else {
            slot.is_marked = true;
            true
        }
    }

    pub fn is_marked(&self, id: ObjId) -> bool {
        if !self.contains(id) {
            return false;
        }

        self.objects[id.0]
            .as_ref()
            .expect("slot should exist")
            .is_marked
    }

    pub fn clear_mark(&mut self, id: ObjId) {
        if !self.contains(id) {
            return;
        }
        self.objects[id.0]
            .as_mut()
            .expect("slot should exist")
            .is_marked = false;
    }

    pub fn iter_ids(&self) -> Vec<ObjId> {
        let mut ids = Vec::new();
        for (index, slot) in self.objects.iter().enumerate() {
            if slot.is_some() {
                ids.push(ObjId(index));
            }
        }
        ids
    }
}

impl Index<ObjId> for Heap {
    type Output = HeapObj;

    fn index(&self, index: ObjId) -> &Self::Output {
        &self.objects[index.0]
            .as_ref()
            .expect("attempted to access freed heap object")
            .object
    }
}

impl IndexMut<ObjId> for Heap {
    fn index_mut(&mut self, index: ObjId) -> &mut Self::Output {
        &mut self.objects[index.0]
            .as_mut()
            .expect("attempted to access freed heap object")
            .object
    }
}
