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

use crate::types::value::{
    class::{ObjBoundMethod, ObjClass, ObjInstance},
    closure::ObjClosure,
    function::ObjFunction,
    native::ObjNative,
    obj::ObjType,
    string::ObjString,
};

trait ObjPtrTarget {}

#[cfg(value_repr = "union")]
mod value_union {

    use std::fmt::{Debug, Display};

    use super::ObjPtrTarget;

    use crate::types::value::obj::{Obj, ObjPtr};

    #[repr(u8)]
    #[derive(PartialEq, Default)]
    enum ValueType {
        #[default]
        Nil,
        Bool,
        Number,

        // Heap values
        Obj,
    }

    #[derive(Default)]
    pub struct Value {
        vtype: ValueType,
        vunion: ValueUnion,
    }

    union ValueUnion {
        boolean: bool,
        number: f64,
        obj: *mut Obj,
    }

    impl Default for ValueUnion {
        fn default() -> Self {
            ValueUnion { number: 0.0 }
        }
    }

    impl Display for Value {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self.vtype {
                ValueType::Bool => write!(f, "{}", self.as_bool()),
                ValueType::Number => write!(f, "{}", self.as_number()),
                ValueType::Nil => write!(f, "nil"),
                ValueType::Obj => write!(f, "{}", ObjPtr::from(self.as_obj()).to_string()),
            }
        }
    }

    impl Debug for Value {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self)
        }
    }

    impl Clone for Value {
        fn clone(&self) -> Self {
            match self.vtype {
                ValueType::Bool => Value::from(self.as_bool()),
                ValueType::Number => Value::from(self.as_number()),
                ValueType::Nil => Value::from(()),
                ValueType::Obj => Value::from(self.as_obj()),
            }
        }
    }

    impl PartialEq for Value {
        fn eq(&self, other: &Self) -> bool {
            if self.vtype != other.vtype {
                return false;
            }

            match self.vtype {
                ValueType::Bool => self.as_bool() == other.as_bool(),
                ValueType::Number => self.as_number() == other.as_number(),
                ValueType::Nil => true,
                ValueType::Obj => {
                    let a = self.as_obj();
                    let b = other.as_obj();

                    a == b
                }
            }
        }
    }

    impl From<bool> for Value {
        fn from(value: bool) -> Self {
            Self {
                vtype: ValueType::Bool,
                vunion: ValueUnion { boolean: value },
            }
        }
    }

    impl From<f64> for Value {
        fn from(value: f64) -> Self {
            Self {
                vtype: ValueType::Number,
                vunion: ValueUnion { number: value },
            }
        }
    }

    impl From<()> for Value {
        fn from(_: ()) -> Self {
            Self {
                vtype: ValueType::Nil,
                vunion: ValueUnion { number: 0.0 },
            }
        }
    }

    impl<T> From<*mut T> for Value
    where
        T: ObjPtrTarget,
    {
        fn from(value: *mut T) -> Self {
            Self {
                vtype: ValueType::Obj,
                vunion: ValueUnion {
                    obj: value as *mut Obj,
                },
            }
        }
    }

    impl Value {
        pub fn is_number(&self) -> bool {
            matches!(self.vtype, ValueType::Number)
        }

        pub fn as_number(&self) -> f64 {
            unsafe { self.vunion.number }
        }

        pub fn is_nil(&self) -> bool {
            matches!(self.vtype, ValueType::Nil)
        }

        pub fn is_bool(&self) -> bool {
            matches!(self.vtype, ValueType::Bool)
        }

        pub fn as_bool(&self) -> bool {
            unsafe { self.vunion.boolean }
        }

        pub fn is_obj(&self) -> bool {
            matches!(self.vtype, ValueType::Obj)
        }

        pub fn as_obj(&self) -> *mut Obj {
            unsafe { self.vunion.obj }
        }
    }
}

#[cfg(value_repr = "nan")]
mod value_nan {
    use super::ObjPtrTarget;

    use std::fmt::{Debug, Display};

    use crate::types::value::obj::{Obj, ObjPtr};

    const SIGN_BIT: u64 = 0x8000000000000000;

    const QNAN: u64 = 0x7ffc000000000000;

    const NIL_MASK: u64 = QNAN | 1;
    const FALSE_MASK: u64 = QNAN | 2;
    const TRUE_MASK: u64 = QNAN | 3;

    #[derive(Debug, Clone)]
    pub struct Value(u64);

    impl Default for Value {
        fn default() -> Self {
            Self(NIL_MASK)
        }
    }

    impl Value {
        pub fn is_number(&self) -> bool {
            (self.0 & QNAN) != QNAN
        }

        pub fn as_number(&self) -> f64 {
            f64::from_bits(self.0)
        }

        pub fn is_nil(&self) -> bool {
            self.0 == NIL_MASK
        }

        pub fn is_bool(&self) -> bool {
            self.0 == TRUE_MASK || self.0 == FALSE_MASK
        }

        pub fn as_bool(&self) -> bool {
            self.0 == TRUE_MASK
        }

        pub fn is_obj(&self) -> bool {
            (self.0 & (QNAN | SIGN_BIT)) == (QNAN | SIGN_BIT)
        }

        pub fn as_obj(&self) -> *mut Obj {
            (self.0 & !(SIGN_BIT | QNAN)) as *mut Obj
        }
    }

    impl Display for Value {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if self.is_bool() {
                return write!(f, "{}", self.as_bool());
            } else if self.is_nil() {
                return write!(f, "nil");
            } else if self.is_number() {
                return write!(f, "{}", self.as_number());
            } else if self.is_obj() {
                return write!(f, "{}", ObjPtr::from(self.as_obj()));
            }

            write!(f, "<unknown>")
        }
    }

    impl PartialEq for Value {
        fn eq(&self, other: &Self) -> bool {
            if self.is_number() && other.is_number() {
                return self.as_number() == other.as_number();
            }
            self.0 == other.0
        }
    }

    impl From<bool> for Value {
        fn from(value: bool) -> Self {
            Value(if value { TRUE_MASK } else { FALSE_MASK })
        }
    }

    impl From<f64> for Value {
        fn from(value: f64) -> Self {
            Value(f64::to_bits(value))
        }
    }

    impl From<()> for Value {
        fn from(_: ()) -> Self {
            Value(NIL_MASK)
        }
    }

    impl<T> From<*mut T> for Value
    where
        T: ObjPtrTarget,
    {
        fn from(value: *mut T) -> Self {
            Value(SIGN_BIT | QNAN | (value as *mut Obj as u64))
        }
    }
}

#[cfg(value_repr = "union")]
pub use crate::types::value::value_union::Value;

#[cfg(value_repr = "nan")]
pub use crate::types::value::value_nan::Value;

impl Add for Value {
    type Output = Result<Self, Box<dyn Error>>;

    fn add(self, rhs: Self) -> Self::Output {
        // Strings are directly handled in the VM, because they require
        // access to the heap.
        if self.is_number() && rhs.is_number() {
            Ok(Value::from(self.as_number() + rhs.as_number()))
        } else {
            Err("Operands must be two numbers or two strings.".into())
        }
    }
}

impl Sub for Value {
    type Output = Result<Self, Box<dyn Error>>;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.is_number() && rhs.is_number() {
            Ok(Value::from(self.as_number() - rhs.as_number()))
        } else {
            Err("Operands must be numbers.".into())
        }
    }
}

impl Mul for Value {
    type Output = Result<Self, Box<dyn Error>>;

    fn mul(self, rhs: Self) -> Self::Output {
        if self.is_number() && rhs.is_number() {
            Ok(Value::from(self.as_number() * rhs.as_number()))
        } else {
            Err("Operands must be numbers.".into())
        }
    }
}

impl Div for Value {
    type Output = Result<Self, Box<dyn Error>>;

    fn div(self, rhs: Self) -> Self::Output {
        if self.is_number() && rhs.is_number() {
            Ok(Value::from(self.as_number() / rhs.as_number()))
        } else {
            Err("Operands must be numbers.".into())
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.is_number() && other.is_number() {
            self.as_number().partial_cmp(&other.as_number())
        } else {
            None
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
        let v = Value::from(true);
        assert_eq!(v.is_bool(), true);

        let v = Value::from(3.14);
        assert_eq!(v.is_number(), true);
    }
}
