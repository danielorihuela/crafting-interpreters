pub mod class;
pub mod closure;
pub mod function;
pub mod native;
pub mod obj;
pub mod upvalue;

use std::{
    error::Error,
    ops::{Add, Div, Mul, Sub},
};

use crate::{
    heap::{ObjId, ObjKind},
    vm::VM,
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),

    // Heap
    Obj(ObjId),
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a.partial_cmp(b),
            (Value::Bool(a), Value::Bool(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

impl Value {
    pub fn to_string(&self, vm: &VM) -> String {
        match self {
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::Nil => "nil".to_string(),
            Value::Obj(id) => match id.kind() {
                ObjKind::String => vm.heap.string(*id).clone(),
                ObjKind::Function => vm.heap.function(*id).to_string(vm),
                ObjKind::Closure => vm.heap.closure(*id).to_string(vm),
                ObjKind::Native => vm.heap.native(*id).to_string(),
                ObjKind::Upvalue => vm.heap.upvalue(*id).to_string(),
                ObjKind::Class => vm.heap.class(*id).to_string(vm),
                ObjKind::Instance => vm.heap.instance(*id).to_string(vm),
                ObjKind::BoundMethod => vm.heap.bound_method(*id).to_string(vm),
            },
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

    pub fn as_obj(self) -> ObjId {
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
                    matches!(self, Value::Obj(id) if id.kind() == ObjKind::$ty)
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

    #[test]
    fn object_values_store_objid_directly() {
        let id = ObjId::null();
        let value = Value::Obj(id);

        assert!(value.is_obj());
        assert_eq!(value.as_obj(), id);
    }
}
