use std::{
    mem::MaybeUninit,
    ops::{Index, IndexMut},
};

use crate::vm::FRAMES_MAX;

const STACK_MAX: usize = FRAMES_MAX * u8::MAX as usize;

pub struct Stack<T> {
    data: [MaybeUninit<T>; STACK_MAX],
    top: usize,
}

impl<T> Default for Stack<T> {
    fn default() -> Self {
        Stack {
            data: [const { MaybeUninit::uninit() }; STACK_MAX],
            top: 0,
        }
    }
}

impl<T> Index<usize> for Stack<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &*self.data[index].as_ptr() }
    }
}

impl<T> IndexMut<usize> for Stack<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe { &mut *self.data[index].as_mut_ptr() }
    }
}

impl<T> Stack<T> {
    pub fn len(&self) -> usize {
        self.top
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.data.as_mut_ptr().cast::<T>()
    }

    pub fn truncate(&mut self, top: usize) {
        assert!(top <= self.top, "cannot grow stack with truncate");
        self.top = top;
    }

    pub unsafe fn truncate_to_ptr(&mut self, top: *mut T) {
        let base = self.as_mut_ptr();
        let new_top = unsafe { top.offset_from(base) } as usize;
        self.truncate(new_top);
    }

    pub fn peek(&self, distance: usize) -> &T {
        unsafe { &*self.data[self.top - 1 - distance].as_ptr() }
    }

    pub fn push(&mut self, value: T) {
        self.data[self.top] = MaybeUninit::new(value);
        self.top += 1;
    }

    pub fn pop(&mut self) -> T {
        self.top -= 1;
        unsafe { self.data[self.top].as_ptr().read() }
    }
}

#[cfg(debug_assertions)]
pub mod debug {
    use std::fmt::Display;

    use super::*;

    pub fn show_stack<T: Display>(stack: &Stack<T>) {
        print!("          ");
        for i in 0..stack.top {
            print!("[{}]", stack[i]);
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_push_pop() {
        let mut stack = Stack::<i32>::default();
        assert_eq!(stack.top, 0);

        stack.push(1);
        stack.push(2);
        assert_eq!(stack.top, 2);

        assert_eq!(stack.pop(), 2);
        assert_eq!(stack.pop(), 1);
        assert_eq!(stack.top, 0);
    }

    #[test]
    fn test_stack_truncate_to_ptr() {
        let mut stack = Stack::<i32>::default();
        stack.push(1);
        stack.push(2);
        stack.push(3);

        let new_top = unsafe { stack.as_mut_ptr().add(1) };
        unsafe { stack.truncate_to_ptr(new_top) };

        assert_eq!(stack.top, 1);
        assert_eq!(stack.peek(0), &1);
    }
}
