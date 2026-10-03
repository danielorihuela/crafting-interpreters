use std::str::FromStr;

use strum::EnumString;

#[derive(Clone, Default)]
pub struct Token<'src> {
    pub ttype: Type,
    pub lexeme: &'src str,
    pub line: isize,
}

#[derive(Debug, Default, PartialEq, Clone, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Type {
    // Single-character tokens
    #[strum(serialize = "(")]
    LeftParen,
    #[strum(serialize = ")")]
    RightParen,
    #[strum(serialize = "{")]
    LeftBrace,
    #[strum(serialize = "}")]
    RightBrace,
    #[strum(serialize = ",")]
    Comma,
    #[strum(serialize = ".")]
    Dot,
    #[strum(serialize = "-")]
    Minus,
    #[strum(serialize = "+")]
    Plus,
    #[strum(serialize = ";")]
    Semicolon,
    #[strum(serialize = "?")]
    QuestionMark,
    #[strum(serialize = ":")]
    Colon,
    #[strum(serialize = "/")]
    Slash,
    #[strum(serialize = "*")]
    Star,

    // Comparison operators
    #[strum(serialize = "!")]
    Bang,
    #[strum(serialize = "!=")]
    BangEqual,
    #[strum(serialize = "=")]
    Equal,
    #[strum(serialize = "==")]
    EqualEqual,
    #[strum(serialize = ">")]
    Greater,
    #[strum(serialize = ">=")]
    GreaterEqual,
    #[strum(serialize = "<")]
    Less,
    #[strum(serialize = "<=")]
    LessEqual,

    // Literals
    #[strum(disabled)]
    Identifier,
    #[strum(disabled)]
    String,
    #[strum(disabled)]
    Number,

    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    #[strum(disabled)]
    #[default]
    Error,
    #[strum(disabled)]
    Eof,
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Type {
    pub fn from_byte(c: u8) -> Option<Self> {
        Type::from_str(std::str::from_utf8(&[c]).unwrap()).ok()
    }

    pub fn from_byte_array(bytes: &[u8]) -> Option<Self> {
        Type::from_str(std::str::from_utf8(bytes).unwrap()).ok()
    }
}
