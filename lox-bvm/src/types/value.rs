pub mod class;
pub mod closure;
pub mod function;
pub mod native;
pub mod obj;
pub mod string;
pub mod upvalue;

use std::{
    fmt::{Debug, Display},
    ops::{Add, Deref, Div, Mul, Sub},
};

use crate::types::{
    AsciiChar,
    value::{
        class::{ObjBoundMethod, ObjClass, ObjInstance},
        closure::ObjClosure,
        function::ObjFunction,
        native::ObjNative,
        obj::{Obj, ObjType},
        string::ObjString,
    },
};

// #[repr(u8)]
// #[derive(PartialEq)]
// enum ValueType {
//     Bool,
//     Number,
//     Nil,

//     // Heap values
//     Obj,
// }

// pub struct Value {
//     vtype: ValueType,
//     vunion: ValueUnion,
// }

// union ValueUnion {
//     boolean: bool,
//     number: f64,
//     obj: *mut Obj,
// }

// impl Display for Value {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self.vtype {
//             ValueType::Bool => write!(f, "{}", self.as_bool()),
//             ValueType::Number => write!(f, "{}", self.as_number()),
//             ValueType::Nil => write!(f, "nil"),
//             ValueType::Obj => {
//                 let data = match self.obj_type() {
//                     ObjType::String => {
//                         let s = self.as_obj() as *mut ObjString;

//                         let string = unsafe {
//                             std::str::from_utf8_unchecked(std::slice::from_raw_parts(
//                                 (*s).chars,
//                                 (*s).length,
//                             ))
//                         };

//                         string.to_string()
//                     }
//                     ObjType::Function => {
//                         let function = self.as_obj() as *mut ObjFunction;
//                         if unsafe { (*function).name.is_null() } {
//                             "<script>".to_string()
//                         } else {
//                             let value_name = Value::from(unsafe { (*function).name });
//                             format!("<fn {}>", value_name)
//                         }
//                     }
//                     ObjType::Closure => {
//                         let closure = self.as_obj() as *mut ObjClosure;
//                         let function = unsafe { (*closure).function };
//                         if unsafe { (*function).name.is_null() } {
//                             "<script>".to_string()
//                         } else {
//                             let value_name = Value::from(unsafe { (*function).name });
//                             format!("<fn {}>", value_name)
//                         }
//                     }
//                     ObjType::Native => "<native fn>".to_string(),
//                     ObjType::Upvalue => "upvalue".to_string(),
//                     ObjType::Class => {
//                         let class = self.as_obj() as *mut ObjClass;
//                         let value_name = Value::from(unsafe { (*class).name });
//                         format!("{}", value_name)
//                     }
//                     ObjType::Instance => {
//                         let instance = self.as_obj() as *mut ObjInstance;
//                         let class = unsafe { (*instance).class };
//                         let value_name = Value::from(unsafe { (*class).name });
//                         format!("{} instance", value_name)
//                     }
//                     ObjType::BoundMethod => {
//                         let bound_method = self.as_obj() as *mut ObjBoundMethod;
//                         let method = unsafe { (*(*bound_method).method).function };
//                         if unsafe { (*method).name.is_null() } {
//                             "<script>".to_string()
//                         } else {
//                             let value_name = Value::from(unsafe { (*method).name });
//                             format!("<fn {}>", value_name)
//                         }
//                     }
//                 };
//                 write!(f, "{}", data)
//             }
//         }
//     }
// }

// impl Debug for Value {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self)
//     }
// }

// impl Default for Value {
//     fn default() -> Self {
//         Value::from(())
//     }
// }

// impl Clone for Value {
//     fn clone(&self) -> Self {
//         match self.vtype {
//             ValueType::Bool => Value::from(self.as_bool()),
//             ValueType::Number => Value::from(self.as_number()),
//             ValueType::Nil => Value::from(()),
//             ValueType::Obj => Value::from(self.as_obj()),
//         }
//     }
// }

// impl PartialEq for Value {
//     fn eq(&self, other: &Self) -> bool {
//         if self.vtype != other.vtype {
//             return false;
//         }

//         match self.vtype {
//             ValueType::Bool => self.as_bool() == other.as_bool(),
//             ValueType::Number => self.as_number() == other.as_number(),
//             ValueType::Nil => true,
//             ValueType::Obj => {
//                 let a = self.as_obj();
//                 let b = other.as_obj();

//                 a == b
//             }
//         }
//     }
// }

// impl From<bool> for Value {
//     fn from(value: bool) -> Self {
//         Self {
//             vtype: ValueType::Bool,
//             vunion: ValueUnion { boolean: value },
//         }
//     }
// }

// impl From<f64> for Value {
//     fn from(value: f64) -> Self {
//         Self {
//             vtype: ValueType::Number,
//             vunion: ValueUnion { number: value },
//         }
//     }
// }

// impl From<()> for Value {
//     fn from(_: ()) -> Self {
//         Self {
//             vtype: ValueType::Nil,
//             vunion: ValueUnion { number: 0.0 },
//         }
//     }
// }

// impl From<*mut Obj> for Value {
//     fn from(value: *mut Obj) -> Self {
//         Self {
//             vtype: ValueType::Obj,
//             vunion: ValueUnion { obj: value },
//         }
//     }
// }

// impl From<*mut ObjString> for Value {
//     fn from(value: *mut ObjString) -> Self {
//         Self {
//             vtype: ValueType::Obj,
//             vunion: ValueUnion {
//                 obj: value as *mut Obj,
//             },
//         }
//     }
// }

// impl From<*mut ObjFunction> for Value {
//     fn from(value: *mut ObjFunction) -> Self {
//         Self {
//             vtype: ValueType::Obj,
//             vunion: ValueUnion {
//                 obj: value as *mut Obj,
//             },
//         }
//     }
// }

// impl From<*mut ObjNative> for Value {
//     fn from(value: *mut ObjNative) -> Self {
//         Self {
//             vtype: ValueType::Obj,
//             vunion: ValueUnion {
//                 obj: value as *mut Obj,
//             },
//         }
//     }
// }

// impl From<*mut ObjClosure> for Value {
//     fn from(value: *mut ObjClosure) -> Self {
//         Self {
//             vtype: ValueType::Obj,
//             vunion: ValueUnion {
//                 obj: value as *mut Obj,
//             },
//         }
//     }
// }

// impl From<*mut ObjClass> for Value {
//     fn from(value: *mut ObjClass) -> Self {
//         Self {
//             vtype: ValueType::Obj,
//             vunion: ValueUnion {
//                 obj: value as *mut Obj,
//             },
//         }
//     }
// }

// impl From<*mut ObjInstance> for Value {
//     fn from(value: *mut ObjInstance) -> Self {
//         Self {
//             vtype: ValueType::Obj,
//             vunion: ValueUnion {
//                 obj: value as *mut Obj,
//             },
//         }
//     }
// }

// impl From<*mut ObjBoundMethod> for Value {
//     fn from(value: *mut ObjBoundMethod) -> Self {
//         Self {
//             vtype: ValueType::Obj,
//             vunion: ValueUnion {
//                 obj: value as *mut Obj,
//             },
//         }
//     }
// }

#[derive(Debug)]
pub struct OperationError(pub String);

impl Deref for OperationError {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// impl Add for Value {
//     type Output = Result<Self, OperationError>;

//     fn add(self, rhs: Self) -> Self::Output {
//         // Strings are directly handled in the VM, because they require
//         // access to the heap.
//         if self.is_number() && rhs.is_number() {
//             Ok(Value::from(self.as_number() + rhs.as_number()))
//         } else {
//             Err(OperationError(
//                 "Operands must be two numbers or two strings.".to_string(),
//             ))
//         }
//     }
// }

// impl Sub for Value {
//     type Output = Result<Self, OperationError>;

//     fn sub(self, rhs: Self) -> Self::Output {
//         if self.is_number() && rhs.is_number() {
//             Ok(Value::from(self.as_number() - rhs.as_number()))
//         } else {
//             Err(OperationError("Operands must be numbers.".to_string()))
//         }
//     }
// }

// impl Mul for Value {
//     type Output = Result<Self, OperationError>;

//     fn mul(self, rhs: Self) -> Self::Output {
//         if self.is_number() && rhs.is_number() {
//             Ok(Value::from(self.as_number() * rhs.as_number()))
//         } else {
//             Err(OperationError("Operands must be numbers.".to_string()))
//         }
//     }
// }

// impl Div for Value {
//     type Output = Result<Self, OperationError>;

//     fn div(self, rhs: Self) -> Self::Output {
//         if self.is_number() && rhs.is_number() {
//             Ok(Value::from(self.as_number() / rhs.as_number()))
//         } else {
//             Err(OperationError("Operands must be numbers.".to_string()))
//         }
//     }
// }

// impl PartialOrd for Value {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         if self.is_number() && other.is_number() {
//             self.as_number().partial_cmp(&other.as_number())
//         } else {
//             None
//         }
//     }
// }

// impl Value {
//     pub fn is_bool(&self) -> bool {
//         matches!(self.vtype, ValueType::Bool)
//     }

//     pub fn is_number(&self) -> bool {
//         matches!(self.vtype, ValueType::Number)
//     }

//     pub fn is_nil(&self) -> bool {
//         matches!(self.vtype, ValueType::Nil)
//     }

//     pub fn is_falsey(&self) -> bool {
//         self.is_nil() || (self.is_bool() && !self.as_bool())
//     }

//     pub fn is_obj(&self) -> bool {
//         matches!(self.vtype, ValueType::Obj)
//     }

//     fn obj_type(&self) -> &ObjType {
//         &unsafe { &*self.as_obj() }.otype
//     }

//     pub fn is_obj_type(&self, otype: &ObjType) -> bool {
//         self.is_obj() && self.obj_type() == otype
//     }

//     pub fn is_string(&self) -> bool {
//         self.is_obj() && self.obj_type() == &ObjType::String
//     }

//     pub fn is_function(&self) -> bool {
//         self.is_obj() && self.obj_type() == &ObjType::Function
//     }

//     pub fn is_native(&self) -> bool {
//         self.is_obj() && self.obj_type() == &ObjType::Native
//     }

//     pub fn is_closure(&self) -> bool {
//         self.is_obj() && self.obj_type() == &ObjType::Closure
//     }

//     pub fn is_class(&self) -> bool {
//         self.is_obj() && self.obj_type() == &ObjType::Class
//     }

//     pub fn is_instance(&self) -> bool {
//         self.is_obj() && self.obj_type() == &ObjType::Instance
//     }

//     pub fn is_bound_method(&self) -> bool {
//         self.is_obj() && self.obj_type() == &ObjType::BoundMethod
//     }

//     pub fn as_bool(&self) -> bool {
//         unsafe { self.vunion.boolean }
//     }

//     pub fn as_number(&self) -> f64 {
//         unsafe { self.vunion.number }
//     }

//     pub fn as_obj(&self) -> *mut Obj {
//         unsafe { self.vunion.obj }
//     }

//     pub fn as_string(&self) -> *mut ObjString {
//         self.as_obj() as *mut ObjString
//     }

//     pub fn as_cstring(&self) -> *mut AsciiChar {
//         unsafe { (*self.as_string()).chars }
//     }

//     pub fn as_function(&self) -> *mut ObjFunction {
//         self.as_obj() as *mut ObjFunction
//     }

//     pub fn as_native(&self) -> *mut ObjNative {
//         self.as_obj() as *mut ObjNative
//     }

//     pub fn as_closure(&self) -> *mut ObjClosure {
//         self.as_obj() as *mut ObjClosure
//     }

//     pub fn as_class(&self) -> *mut ObjClass {
//         self.as_obj() as *mut ObjClass
//     }

//     pub fn as_instance(&self) -> *mut ObjInstance {
//         self.as_obj() as *mut ObjInstance
//     }

//     pub fn as_bound_method(&self) -> *mut ObjBoundMethod {
//         self.as_obj() as *mut ObjBoundMethod
//     }
// }

pub struct Value(u64);

impl Value {
    pub fn num_to_value(num: f64) -> Value {
        Value(f64::to_bits(num))
    }

    pub fn value_to_num(&self) -> f64 {
        f64::from_bits(self.0)
    }

    pub fn is_number(&self) -> bool {
        (self.0 & QNAN.0) != QNAN.0
    }

    pub fn as_number(&self) -> f64 {
        self.value_to_num()
    }

    pub fn nil_val() -> Value {
        Value(QNAN.0 | TAG_NIL.0)
    }

    pub fn is_nil(&self) -> bool {
        self.0 == Self::nil_val().0
    }

    pub fn false_val() -> Value {
        Value(QNAN.0 | TAG_FALSE.0)
    }

    pub fn true_val() -> Value {
        Value(QNAN.0 | TAG_TRUE.0)
    }

    pub fn bool_val(b: bool) -> Value {
        if b {
            Self::true_val()
        } else {
            Self::false_val()
        }
    }

    pub fn is_bool(&self) -> bool {
        self.0 == Self::true_val().0 || self.0 == Self::false_val().0
    }

    pub fn as_bool(&self) -> bool {
        self.0 == Self::true_val().0
    }

    pub fn obj_val(obj: *mut Obj) -> Value {
        Value(SIGN_BIT.0 | QNAN.0 | (obj as u64))
    }

    pub fn as_obj(&self) -> *mut Obj {
        (self.0 & !(SIGN_BIT.0 | QNAN.0)) as *mut Obj
    }

    pub fn obj_type(&self) -> &ObjType {
        &unsafe { &*self.as_obj() }.otype
    }

    pub fn is_obj(&self) -> bool {
        (self.0 & (QNAN.0 | SIGN_BIT.0)) == (QNAN.0 | SIGN_BIT.0)
    }

    pub fn as_function(&self) -> *mut ObjFunction {
        self.as_obj() as *mut ObjFunction
    }

    pub fn as_native(&self) -> *mut ObjNative {
        self.as_obj() as *mut ObjNative
    }

    pub fn as_closure(&self) -> *mut ObjClosure {
        self.as_obj() as *mut ObjClosure
    }

    pub fn as_class(&self) -> *mut ObjClass {
        self.as_obj() as *mut ObjClass
    }

    pub fn as_instance(&self) -> *mut ObjInstance {
        self.as_obj() as *mut ObjInstance
    }

    pub fn as_bound_method(&self) -> *mut ObjBoundMethod {
        self.as_obj() as *mut ObjBoundMethod
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

    pub fn is_closure(&self) -> bool {
        self.is_obj() && self.obj_type() == &ObjType::Closure
    }

    pub fn is_class(&self) -> bool {
        self.is_obj() && self.obj_type() == &ObjType::Class
    }

    pub fn is_instance(&self) -> bool {
        self.is_obj() && self.obj_type() == &ObjType::Instance
    }

    pub fn is_bound_method(&self) -> bool {
        self.is_obj() && self.obj_type() == &ObjType::BoundMethod
    }

    pub fn as_string(&self) -> *mut ObjString {
        self.as_obj() as *mut ObjString
    }

    pub fn is_falsey(&self) -> bool {
        self.is_nil() || (self.is_bool() && !self.as_bool())
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_bool() {
            return write!(f, "{}", self.as_bool());
        } else if self.is_nil() {
            return write!(f, "nil");
        } else if self.is_number() {
            return write!(f, "{}", self.value_to_num());
        } else if self.is_obj() {
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
                ObjType::Closure => {
                    let closure = self.as_obj() as *mut ObjClosure;
                    let function = unsafe { (*closure).function };
                    if unsafe { (*function).name.is_null() } {
                        "<script>".to_string()
                    } else {
                        let value_name = Value::from(unsafe { (*function).name });
                        format!("<fn {}>", value_name)
                    }
                }
                ObjType::Native => "<native fn>".to_string(),
                ObjType::Upvalue => "upvalue".to_string(),
                ObjType::Class => {
                    let class = self.as_obj() as *mut ObjClass;
                    let value_name = Value::from(unsafe { (*class).name });
                    format!("{}", value_name)
                }
                ObjType::Instance => {
                    let instance = self.as_obj() as *mut ObjInstance;
                    let class = unsafe { (*instance).class };
                    let value_name = Value::from(unsafe { (*class).name });
                    format!("{} instance", value_name)
                }
                ObjType::BoundMethod => {
                    let bound_method = self.as_obj() as *mut ObjBoundMethod;
                    let method = unsafe { (*(*bound_method).method).function };
                    if unsafe { (*method).name.is_null() } {
                        "<script>".to_string()
                    } else {
                        let value_name = Value::from(unsafe { (*method).name });
                        format!("<fn {}>", value_name)
                    }
                }
            };
            return write!(f, "{}", data);
        }

        return write!(f, "<unknown>");
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
        Value(self.0.clone())
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
        Self::bool_val(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::num_to_value(value)
    }
}

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Self::nil_val()
    }
}

impl From<*mut Obj> for Value {
    fn from(value: *mut Obj) -> Self {
        Self::obj_val(value)
    }
}

impl From<*mut ObjString> for Value {
    fn from(value: *mut ObjString) -> Self {
        Self::obj_val(value as *mut Obj)
    }
}

impl From<*mut ObjFunction> for Value {
    fn from(value: *mut ObjFunction) -> Self {
        Self::obj_val(value as *mut Obj)
    }
}

impl From<*mut ObjNative> for Value {
    fn from(value: *mut ObjNative) -> Self {
        Self::obj_val(value as *mut Obj)
    }
}

impl From<*mut ObjClosure> for Value {
    fn from(value: *mut ObjClosure) -> Self {
        Self::obj_val(value as *mut Obj)
    }
}

impl From<*mut ObjClass> for Value {
    fn from(value: *mut ObjClass) -> Self {
        Self::obj_val(value as *mut Obj)
    }
}

impl From<*mut ObjInstance> for Value {
    fn from(value: *mut ObjInstance) -> Self {
        Self::obj_val(value as *mut Obj)
    }
}

impl From<*mut ObjBoundMethod> for Value {
    fn from(value: *mut ObjBoundMethod) -> Self {
        Self::obj_val(value as *mut Obj)
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

pub const SIGN_BIT: Value = Value(0x8000000000000000);
pub const QNAN: Value = Value(0x7ffc000000000000);
pub const TAG_NIL: Value = Value(1);
pub const TAG_FALSE: Value = Value(2);
pub const TAG_TRUE: Value = Value(3);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value() {
        let v = Value::bool_val(true);
        assert_eq!(v.is_bool(), true);

        let v = Value::num_to_value(3.14);
        assert_eq!(v.is_number(), true);
    }
}
