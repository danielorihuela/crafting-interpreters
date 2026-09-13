pub mod class;
pub mod closure;
pub mod function;
pub mod native;
pub mod obj;
pub mod string;
pub mod upvalue;

use std::{
    error::Error,
    fmt::{Debug, Display},
    ops::{Add, Div, Mul, Sub},
};

use crate::types::value::{
    class::{ObjBoundMethod, ObjClass, ObjInstance},
    closure::ObjClosure,
    function::ObjFunction,
    native::ObjNative,
    obj::{Obj, ObjPtr, ObjType},
    string::ObjString,
};

trait ObjPtrTarget {}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub enum Value {
    #[default]
    Nil,
    Bool(bool),
    Number(f64),

    // Heap values
    Obj(*mut Obj),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(b) => write!(f, "{b}"),
            Value::Number(n) => write!(f, "{n}"),
            Value::Nil => write!(f, "nil"),
            Value::Obj(o) => write!(f, "{}", ObjPtr::from(*o).to_string()),
        }
    }
}

impl<T> From<*mut T> for Value
where
    T: ObjPtrTarget,
{
    fn from(value: *mut T) -> Self {
        Self::Obj(value as *mut Obj)
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

    pub fn as_obj(&self) -> *mut Obj {
        if let Value::Obj(o) = self {
            *o
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
                    self.is_obj_type(&ObjType::$ty)
                }

                pub fn [<as_ $ty:snake>](&self) -> *mut [<Obj $ty>] {
                    self.as_obj() as *mut [<Obj $ty>]
                }
            )+
        }
    };
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        self.is_nil() || (self.is_bool() && !self.as_bool())
    }

    fn obj_type(&self) -> &ObjType {
        &unsafe { &*self.as_obj() }.otype
    }

    pub fn is_obj_type(&self, otype: &ObjType) -> bool {
        self.is_obj() && self.obj_type() == otype
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
