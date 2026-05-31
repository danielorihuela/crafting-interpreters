use std::{
    fmt::{Debug, Display},
    ops::{Add, Div, Mul, Sub},
};

#[repr(u8)]
#[derive(PartialEq)]
enum ValueType {
    Bool,
    Number,
    Nil,
}

pub struct Value {
    vtype: ValueType,
    vunion: ValueUnion,
}

union ValueUnion {
    boolean: bool,
    number: f64,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.vtype {
            ValueType::Bool => write!(f, "{}", self.as_bool()),
            ValueType::Number => write!(f, "{}", self.as_number()),
            ValueType::Nil => write!(f, "nil"),
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

impl Add for Value {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        if self.is_number() && rhs.is_number() {
            Value::from(self.as_number() + rhs.as_number())
        } else {
            panic!("Operands must be numbers.");
        }
    }
}

impl Sub for Value {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.is_number() && rhs.is_number() {
            Value::from(self.as_number() - rhs.as_number())
        } else {
            panic!("Operands must be numbers.");
        }
    }
}

impl Mul for Value {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        if self.is_number() && rhs.is_number() {
            Value::from(self.as_number() * rhs.as_number())
        } else {
            panic!("Operands must be numbers.");
        }
    }
}

impl Div for Value {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        if self.is_number() && rhs.is_number() {
            Value::from(self.as_number() / rhs.as_number())
        } else {
            panic!("Operands must be numbers.");
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

    pub fn as_bool(&self) -> bool {
        unsafe { self.vunion.boolean }
    }

    pub fn as_number(&self) -> f64 {
        unsafe { self.vunion.number }
    }

    pub fn is_falsey(&self) -> bool {
        self.is_nil() || (self.is_bool() && !self.as_bool())
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
