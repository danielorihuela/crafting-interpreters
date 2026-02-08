const STACK_MAX: usize = 256;

pub struct Stack<T> {
    data: Box<[T; STACK_MAX]>,
    top: *mut T,
}

impl<T: Default> Default for Stack<T> {
    fn default() -> Self {
        let mut data = Box::new(std::array::from_fn(|_| T::default()));
        let top = data.as_mut_ptr();
        Stack { data, top }
    }
}

impl<T> Stack<T> {
    pub fn push(&mut self, value: T) {
        unsafe {
            *self.top = value;
            self.top = self.top.add(1);
        }
    }

    pub fn pop(&mut self) -> T {
        unsafe {
            self.top = self.top.sub(1);
            self.top.read()
        }
    }
}

#[cfg(debug_assertions)]
pub mod debug {
    use std::fmt::Display;

    use super::*;

    pub fn show_stack<T: Display>(stack: &Stack<T>) {
        print!("          ");
        let mut current = stack.data.as_ptr();
        while current != stack.top {
            print!("[{}]", unsafe { &*current });
            current = unsafe { current.add(1) };
        }
        println!();
    }
}
