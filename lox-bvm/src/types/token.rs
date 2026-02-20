use crate::AsciiChar;
#[derive(Clone)]
pub struct Token {
    pub ttype: Type,
    pub start: *const AsciiChar,
    pub length: usize,
    pub line: isize,
}

impl Default for Token {
    fn default() -> Self {
        Self {
            ttype: Type::Error,
            start: std::ptr::null(),
            length: 0,
            line: 0,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    // Single-character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    QuestionMark,
    Colon,
    Slash,
    Star,

    // Comparison operators
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier,
    String,
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

    Error,
    Eof,
}

impl Type {
    pub fn is_single_byte(c: AsciiChar) -> bool {
        Self::single_byte(c).is_some()
    }

    pub fn single_byte(c: AsciiChar) -> Option<Self> {
        match c {
            b'(' => Some(Type::LeftParen),
            b')' => Some(Type::RightParen),
            b'{' => Some(Type::LeftBrace),
            b'}' => Some(Type::RightBrace),
            b',' => Some(Type::Comma),
            b'.' => Some(Type::Dot),
            b'-' => Some(Type::Minus),
            b'+' => Some(Type::Plus),
            b';' => Some(Type::Semicolon),
            b'?' => Some(Type::QuestionMark),
            b':' => Some(Type::Colon),
            b'/' => Some(Type::Slash),
            b'*' => Some(Type::Star),
            _ => None,
        }
    }

    pub fn is_comparison(c: AsciiChar) -> bool {
        matches!(c, b'!' | b'=' | b'<' | b'>')
    }

    pub fn comparison(c1: AsciiChar, c2: Option<AsciiChar>) -> Option<Self> {
        match (c1, c2) {
            (b'!', None) => Some(Type::Bang),
            (b'!', Some(b'=')) => Some(Type::BangEqual),
            (b'=', None) => Some(Type::Equal),
            (b'=', Some(b'=')) => Some(Type::EqualEqual),
            (b'>', None) => Some(Type::Greater),
            (b'>', Some(b'=')) => Some(Type::GreaterEqual),
            (b'<', None) => Some(Type::Less),
            (b'<', Some(b'=')) => Some(Type::LessEqual),
            _ => None,
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
