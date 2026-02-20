use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
};

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
    Unknown,
}

impl OpCode {
    pub fn maybe_binary_op<
        T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T>,
    >(
        &self,
    ) -> Option<fn(T, T) -> T> {
        match self {
            OpCode::Add => Some(Add::add),
            OpCode::Subtract => Some(Sub::sub),
            OpCode::Multiply => Some(Mul::mul),
            OpCode::Divide => Some(Div::div),
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
        match value {
            0 => OpCode::Constant,
            1 => OpCode::Add,
            2 => OpCode::Subtract,
            3 => OpCode::Multiply,
            4 => OpCode::Divide,
            5 => OpCode::Negate,
            6 => OpCode::Return,
            _ => OpCode::Unknown,
        }
    }
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = format!("{:?}", self).to_ascii_uppercase();
        write!(f, "OP_{}", name)
    }
}
