use std::fmt::Display;

#[repr(u8)]
pub enum OpCode {
    OpConstant,
    OpReturn,
}

impl TryFrom<u8> for OpCode {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == OpCode::OpConstant as u8 => Ok(OpCode::OpConstant),
            x if x == OpCode::OpReturn as u8 => Ok(OpCode::OpReturn),
            _ => Err(()),
        }
    }
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            OpCode::OpConstant => "OP_CONSTANT",
            OpCode::OpReturn => "OP_RETURN",
        };
        write!(f, "{}", name)
    }
}
