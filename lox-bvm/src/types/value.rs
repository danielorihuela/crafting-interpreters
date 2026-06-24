pub mod function;
pub mod native;
pub mod obj;
pub mod string;

use std::{
    fmt::{Debug, Display},
    ops::{Add, Deref, Div, Mul, Sub},
};

use crate::types::{
    AsciiChar,
    value::{
        function::ObjFunction,
        native::ObjNative,
        obj::{Obj, ObjType},
        string::ObjString,
    },
};

#[repr(u8)]
#[derive(PartialEq)]
enum ValueType {
    Bool,
    Number,
    Nil,

    // Heap values
    Obj,
}

pub struct Value {
    vtype: ValueType,
    vunion: ValueUnion,
}

union ValueUnion {
    boolean: bool,
    number: f64,
    obj: *mut Obj,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.vtype {
            ValueType::Bool => write!(f, "{}", self.as_bool()),
            ValueType::Number => write!(f, "{}", self.as_number()),
            ValueType::Nil => write!(f, "nil"),
            ValueType::Obj => {
                let data = match self.obj_type() {
                    ObjType::String => {
                        let s = self.as_obj() as *mut ObjString;

                        let string = unsafe {
                            std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                                (*s).chars,
                                (*s).length,
                            ))
                        };

                        string.to_string()
                    }
                    ObjType::Function => {
                        let function = self.as_obj() as *mut ObjFunction;
                        if unsafe { (*function).name.is_null() } {
                            "<script>".to_string()
                        } else {
                            let value_name = Value::from(unsafe { (*function).name });
                            format!("<fn {}>", value_name)
                        }
                    }
                    ObjType::Native => "<native fn>".to_string(),
                };
                write!(f, "{}", data)
            }
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::from(())
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

impl From<*mut Obj> for Value {
    fn from(value: *mut Obj) -> Self {
        Self {
            vtype: ValueType::Obj,
            vunion: ValueUnion { obj: value },
        }
    }
}

impl From<*mut ObjString> for Value {
    fn from(value: *mut ObjString) -> Self {
        Self {
            vtype: ValueType::Obj,
            vunion: ValueUnion {
                obj: value as *mut Obj,
            },
        }
    }
}

impl From<*mut ObjFunction> for Value {
    fn from(value: *mut ObjFunction) -> Self {
        Self {
            vtype: ValueType::Obj,
            vunion: ValueUnion {
                obj: value as *mut Obj,
            },
        }
    }
}

impl From<*mut ObjNative> for Value {
    fn from(value: *mut ObjNative) -> Self {
        Self {
            vtype: ValueType::Obj,
            vunion: ValueUnion {
                obj: value as *mut Obj,
            },
        }
    }
}

#[derive(Debug)]
pub struct OperationError(pub String);

impl Deref for OperationError {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Add for Value {
    type Output = Result<Self, OperationError>;

    fn add(self, rhs: Self) -> Self::Output {
        // Strings are directly handled in the VM, because they require
        // access to the heap.
        if self.is_number() && rhs.is_number() {
            Ok(Value::from(self.as_number() + rhs.as_number()))
        } else {
            Err(OperationError(
                "Operands must be two numbers or two strings.".to_string(),
            ))
        }
    }
}

impl Sub for Value {
    type Output = Result<Self, OperationError>;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.is_number() && rhs.is_number() {
            Ok(Value::from(self.as_number() - rhs.as_number()))
        } else {
            Err(OperationError("Operands must be numbers.".to_string()))
        }
    }
}

impl Mul for Value {
    type Output = Result<Self, OperationError>;

    fn mul(self, rhs: Self) -> Self::Output {
        if self.is_number() && rhs.is_number() {
            Ok(Value::from(self.as_number() * rhs.as_number()))
        } else {
            Err(OperationError("Operands must be numbers.".to_string()))
        }
    }
}

impl Div for Value {
    type Output = Result<Self, OperationError>;

    fn div(self, rhs: Self) -> Self::Output {
        if self.is_number() && rhs.is_number() {
            Ok(Value::from(self.as_number() / rhs.as_number()))
        } else {
            Err(OperationError("Operands must be numbers.".to_string()))
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

impl Value {
    pub fn is_bool(&self) -> bool {
        matches!(self.vtype, ValueType::Bool)
    }

    pub fn is_number(&self) -> bool {
        matches!(self.vtype, ValueType::Number)
    }

    pub fn is_nil(&self) -> bool {
        matches!(self.vtype, ValueType::Nil)
    }

    pub fn is_falsey(&self) -> bool {
        self.is_nil() || (self.is_bool() && !self.as_bool())
    }

    pub fn is_obj(&self) -> bool {
        matches!(self.vtype, ValueType::Obj)
    }

    fn obj_type(&self) -> &ObjType {
        &unsafe { &*self.as_obj() }.otype
    }

    pub fn is_obj_type(&self, otype: &ObjType) -> bool {
        self.is_obj() && self.obj_type() == otype
    }

    pub fn is_string(&self) -> bool {
        self.is_obj() && self.obj_type() == &ObjType::String
    }

    pub fn is_function(&self) -> bool {
        self.is_obj() && self.obj_type() == &ObjType::Function
    }

    pub fn is_native(&self) -> bool {
        self.is_obj() && self.obj_type() == &ObjType::Native
    }

    pub fn as_bool(&self) -> bool {
        unsafe { self.vunion.boolean }
    }

    pub fn as_number(&self) -> f64 {
        unsafe { self.vunion.number }
    }

    pub fn as_obj(&self) -> *mut Obj {
        unsafe { self.vunion.obj }
    }

    pub fn as_string(&self) -> *mut ObjString {
        self.as_obj() as *mut ObjString
    }

    pub fn as_cstring(&self) -> *mut AsciiChar {
        unsafe { (*self.as_string()).chars }
    }

    pub fn as_function(&self) -> *mut ObjFunction {
        self.as_obj() as *mut ObjFunction
    }

    pub fn as_native(&self) -> *mut ObjNative {
        self.as_obj() as *mut ObjNative
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value() {
        let v = Value::from(true);
        assert_eq!(v.as_bool(), true);

        let v = Value::from(3.14);
        assert_eq!(v.as_number(), 3.14);
    }
}
