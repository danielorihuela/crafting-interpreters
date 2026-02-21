const STACK_MAX: usize = 256;

pub struct Stack<T> {
    data: [T; STACK_MAX],
    top: usize,
}

impl<T: Default> Default for Stack<T> {
    fn default() -> Self {
        Stack {
            data: std::array::from_fn(|_| T::default()),
            top: 0,
        }
    }
}

impl<T> Stack<T> {
    pub fn push(&mut self, value: T) {
        unsafe {
            *self.data.as_mut_ptr().add(self.top) = value;
            self.top += 1;
        }
    }

    pub fn pop(&mut self) -> T {
        unsafe {
            self.top -= 1;
            self.data.as_ptr().add(self.top).read()
        }
    }
}

#[cfg(debug_assertions)]
pub mod debug {
    use std::fmt::Display;

    use super::*;

    pub fn show_stack<T: Display>(stack: &Stack<T>) {
        print!("          ");
        for i in 0..stack.top {
            print!("[{}]", &stack.data[i]);
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
        stack.push(3);

        assert_eq!(stack.pop(), 3);
        assert_eq!(stack.pop(), 2);
        assert_eq!(stack.pop(), 1);
        assert_eq!(stack.top, 0);
    }
}
