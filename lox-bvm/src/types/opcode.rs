use std::{
    fmt::Display,
    mem::transmute,
    ops::{Add, Div, Mul, Sub},
};

use crate::types::value::Value;

#[repr(u8)]
#[derive(Debug)]
pub enum OpCode {
    Constant,
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    Return,

    True,
    False,
    Nil,

    Not,
    Equal,
    Greater,
    Less,

    Unknown,
}

impl OpCode {
    pub fn maybe_binary_op(&self) -> Option<fn(Value, Value) -> Value> {
        match self {
            OpCode::Add => Some(Add::add),
            OpCode::Subtract => Some(Sub::sub),
            OpCode::Multiply => Some(Mul::mul),
            OpCode::Divide => Some(Div::div),
            OpCode::Greater => Some(|a, b| Value::from(a.as_number() > b.as_number())),
            OpCode::Less => Some(|a, b| Value::from(a.as_number() < b.as_number())),
            _ => None,
        }
    }
}

impl From<OpCode> for u8 {
    fn from(op: OpCode) -> Self {
        op as u8
    }
}

impl From<u8> for OpCode {
    fn from(value: u8) -> Self {
        if value > OpCode::Unknown as u8 {
            OpCode::Unknown
        } else {
            unsafe { transmute::<u8, OpCode>(value) }
        }
    }
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = format!("{:?}", self).to_ascii_uppercase();
        write!(f, "OP_{}", name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_conversion() {
        let op = OpCode::from(0);
        assert_eq!(op as u8, OpCode::Constant as u8);

        let op = OpCode::from(OpCode::Unknown as u8 + 1);
        assert_eq!(op as u8, OpCode::Unknown as u8);
    }
}
