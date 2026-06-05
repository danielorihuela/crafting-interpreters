use std::{mem::MaybeUninit, ops::Index};

const STACK_MAX: usize = 256;

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
        unsafe { &*self.data[self.top - 1 - index].as_ptr() }
    }
}

impl<T> Stack<T> {
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
}
