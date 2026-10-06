use std::ops::{Index, IndexMut};

pub struct Arena<T> {
    slots: Vec<Option<Slot<T>>>,
    free_list: Vec<usize>,
}

struct Slot<T> {
    value: T,
    is_marked: bool,
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self {
            slots: Vec::default(),
            free_list: Vec::default(),
        }
    }
}

impl<T> Index<usize> for Arena<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.slots[index]
            .as_ref()
            .expect("attempted to access freed arena object")
            .value
    }
}

impl<T> IndexMut<usize> for Arena<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.slots[index]
            .as_mut()
            .expect("attempted to access freed arena object")
            .value
    }
}

impl<T> Arena<T> {
    pub fn allocate(&mut self, value: T) -> usize {
        let data = Slot {
            value,
            is_marked: false,
        };

        match self.free_list.pop() {
            Some(index) => {
                self.slots[index] = Some(data);
                index
            }
            None => {
                self.slots.push(Some(data));
                self.slots.len() - 1
            }
        }
    }

    pub fn deallocate(&mut self, index: usize) {
        if index < self.slots.len() && self.slots[index].is_some() {
            self.slots[index] = None;
            self.free_list.push(index);
        }
    }

    pub fn contains(&self, index: usize) -> bool {
        index < self.slots.len() && self.slots[index].is_some()
    }

    pub fn mark(&mut self, index: usize) -> bool {
        match self.slots.get_mut(index) {
            Some(Some(slot)) => {
                if slot.is_marked {
                    false
                } else {
                    slot.is_marked = true;
                    true
                }
            }
            _ => false,
        }
    }

    pub fn is_marked(&self, index: usize) -> bool {
        matches!(self.slots.get(index), Some(Some(slot)) if slot.is_marked)
    }

    pub fn clear_mark(&mut self, index: usize) {
        if let Some(Some(slot)) = self.slots.get_mut(index) {
            slot.is_marked = false;
        }
    }

    pub fn iter_indices(&self) -> Vec<usize> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| if slot.is_some() { Some(index) } else { None })
            .collect()
    }
}
