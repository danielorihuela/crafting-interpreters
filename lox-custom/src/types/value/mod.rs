pub mod class;
pub mod closure;
pub mod function;
pub mod native;
pub mod obj;
pub mod string;
pub mod upvalue;

use std::{
    error::Error,
    ops::{Add, Div, Mul, Sub},
};

use crate::{memory::heap::ObjId, types::value::obj::Obj, vm::VM};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),

    // Heap
    Obj(Obj),
}

impl Value {
    pub fn to_string(&self, vm: &VM) -> String {
        match self {
            Value::Bool(b) => format!("{}", b),
            Value::Number(n) => format!("{}", n),
            Value::Nil => format!("{}", "nil"),
            Value::Obj(o) => format!("{}", o.to_string(vm)),
        }
    }
}

impl Value {
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    pub fn as_number(&self) -> f64 {
        if let Value::Number(n) = self {
            *n
        } else {
            panic!("Called as_number on a non-number value");
        }
    }

    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }

    pub fn as_bool(&self) -> bool {
        if let Value::Bool(b) = self {
            *b
        } else {
            panic!("Called as_bool on a non-bool value");
        }
    }

    pub fn is_obj(&self) -> bool {
        matches!(self, Value::Obj(_))
    }

    pub fn as_obj(self) -> Obj {
        if let Value::Obj(o) = self {
            o
        } else {
            panic!("Called as_obj on a non-obj value");
        }
    }
}

impl Add for Value {
    type Output = Result<Self, Box<dyn Error>>;

    fn add(self, rhs: Self) -> Self::Output {
        // Strings are directly handled in the VM, because they require
        // access to the heap.
        if self.is_number() && rhs.is_number() {
            Ok(Value::Number(self.as_number() + rhs.as_number()))
        } else {
            Err("Operands must be two numbers or two strings.".into())
        }
    }
}

impl Sub for Value {
    type Output = Result<Self, Box<dyn Error>>;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.is_number() && rhs.is_number() {
            Ok(Value::Number(self.as_number() - rhs.as_number()))
        } else {
            Err("Operands must be numbers.".into())
        }
    }
}

impl Mul for Value {
    type Output = Result<Self, Box<dyn Error>>;

    fn mul(self, rhs: Self) -> Self::Output {
        if self.is_number() && rhs.is_number() {
            Ok(Value::Number(self.as_number() * rhs.as_number()))
        } else {
            Err("Operands must be numbers.".into())
        }
    }
}

impl Div for Value {
    type Output = Result<Self, Box<dyn Error>>;

    fn div(self, rhs: Self) -> Self::Output {
        if self.is_number() && rhs.is_number() {
            Ok(Value::Number(self.as_number() / rhs.as_number()))
        } else {
            Err("Operands must be numbers.".into())
        }
    }
}

macro_rules! value_obj_accessors {
    ($($ty:ident),+ $(,)?) => {
        paste::paste! {
            $(
                pub fn [<is_ $ty:snake>](&self) -> bool {
                    matches!(self, Value::Obj(Obj::$ty(_)))
                }

                pub fn [<as_ $ty:snake>](&self) -> ObjId {
                    if let Value::Obj(Obj::$ty(s)) = self {
                        *s
                    } else {
                        panic!("Value is not a {}", stringify!($ty));
                    }
                }
            )+
        }
    };
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        self.is_nil() || (self.is_bool() && !self.as_bool())
    }

    value_obj_accessors!(
        String,
        Function,
        Native,
        Closure,
        Class,
        Instance,
        BoundMethod
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value() {
        let v = Value::Bool(true);
        assert_eq!(v.is_bool(), true);

        let v = Value::Number(3.14);
        assert_eq!(v.is_number(), true);
    }
}
