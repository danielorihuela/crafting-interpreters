use std::{ffi::CString, slice::from_raw_parts, str::from_utf8_unchecked};

use crate::{
    AsciiChar,
    collections::value::Value,
    scanner::Scanner,
    types::{TokenType, chunk::Chunk, opcode::OpCode, token::Token},
};

pub struct Parser<'a> {
    scanner: &'a mut Scanner,

    current: Token,
    previous: Token,

    chunk: *mut Chunk,

    had_error: bool,
    panic_mode: bool,
}

impl<'a> Parser<'a> {
    pub fn new(scanner: &'a mut Scanner) -> Self {
        Self {
            scanner,
            current: Token::default(),
            previous: Token::default(),
            chunk: std::ptr::null_mut(),
            had_error: false,
            panic_mode: false,
        }
    }

    pub fn compile(&mut self, chunk: &mut Chunk) -> bool {
        self.had_error = false;
        self.panic_mode = false;

        self.chunk = chunk;
        self.advance();
        self.expression();
        let error_message = CString::new("Expect end of expression.").unwrap();
        self.consume(TokenType::Eof, error_message.as_bytes_with_nul().as_ptr());

        self.end_compiler();

        !self.had_error
    }

    fn advance(&mut self) {
        self.previous = self.current.clone();

        loop {
            self.current = self.scanner.get_token();
            if self.current.ttype != TokenType::Error {
                break;
            }

            self.error_at_current(self.current.start);
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let prefix_rule = self.get_rule(self.previous.ttype.clone()).prefix;
        if let Some(prefix) = prefix_rule {
            prefix(self);
        } else {
            self.error(
                CString::new("Expect expression.")
                    .unwrap()
                    .as_bytes_with_nul()
                    .as_ptr(),
            );
            return;
        }

        while precedence.clone() as u8 <= self.get_rule(self.current.ttype.clone()).precedence as u8
        {
            self.advance();
            let infix_rule = self.get_rule(self.previous.ttype.clone()).infix;
            if let Some(infix) = infix_rule {
                infix(self);
            }
        }
    }

    fn consume(&mut self, ttype: TokenType, message: *const AsciiChar) {
        if self.current.ttype == ttype {
            self.advance();
            return;
        }

        self.error_at_current(message);
    }

    fn end_compiler(&mut self) {
        self.emit_return();

        #[cfg(debug_assertions)]
        {
            if !self.had_error {
                unsafe { (*self.chunk).disassemble("code") };
            }
        }
    }

    fn number(&mut self) {
        let value = unsafe { from_raw_parts(self.previous.start, self.previous.length) };
        let value = unsafe { from_utf8_unchecked(value) };
        let value: f64 = value.parse().unwrap();
        self.emit_constant(value);
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(
            TokenType::RightParen,
            CString::new("Expect ')' after expression.")
                .unwrap()
                .as_bytes_with_nul()
                .as_ptr(),
        );
    }

    fn unary(&mut self) {
        let operator_type = self.previous.ttype.clone();

        self.parse_precedence(Precedence::Unary);

        match operator_type {
            TokenType::Minus => self.emit_byte(OpCode::Negate),
            _ => (),
        }
    }

    fn binary(&mut self) {
        let operator_type = self.previous.ttype.clone();

        let rule = self.get_rule(operator_type.clone());
        self.parse_precedence(Precedence::from(rule.precedence as u8 + 1));

        match operator_type {
            TokenType::Plus => self.emit_byte(OpCode::Add),
            TokenType::Minus => self.emit_byte(OpCode::Subtract),
            TokenType::Star => self.emit_byte(OpCode::Multiply),
            TokenType::Slash => self.emit_byte(OpCode::Divide),
            _ => (),
        }
    }

    fn get_rule(&self, ttype: TokenType) -> ParseRule {
        match ttype {
            TokenType::LeftParen => ParseRule {
                prefix: Some(|parser| parser.grouping()),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Minus => ParseRule {
                prefix: Some(|parser| parser.unary()),
                infix: Some(|parser| parser.binary()),
                precedence: Precedence::Term,
            },
            TokenType::Plus => ParseRule {
                prefix: None,
                infix: Some(|parser| parser.binary()),
                precedence: Precedence::Term,
            },
            TokenType::Star => ParseRule {
                prefix: None,
                infix: Some(|parser| parser.binary()),
                precedence: Precedence::Factor,
            },
            TokenType::Slash => ParseRule {
                prefix: None,
                infix: Some(|parser| parser.binary()),
                precedence: Precedence::Factor,
            },
            TokenType::Number => ParseRule {
                prefix: Some(|parser| parser.number()),
                infix: None,
                precedence: Precedence::None,
            },
            _ => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        }
    }
}

impl<'a> Parser<'a> {
    fn emit_byte(&mut self, b: impl Into<u8>) {
        unsafe {
            (*self.chunk).write(b.into(), self.previous.line as usize);
        }
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::Return);
    }

    fn emit_bytes(&mut self, b1: impl Into<u8>, b2: impl Into<u8>) {
        self.emit_byte(b1);
        self.emit_byte(b2);
    }

    fn emit_constant(&mut self, value: Value) {
        let c = self.make_constant(value);
        self.emit_bytes(OpCode::Constant, c);
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let constant = unsafe { (*self.chunk).add_constant(value) };
        if constant > u8::MAX as usize {
            self.error(
                CString::new("Too many constants in one chunk.")
                    .unwrap()
                    .as_bytes_with_nul()
                    .as_ptr(),
            );
            return 0;
        }

        constant as u8
    }
}

impl<'a> Parser<'a> {
    fn error_at_current(&mut self, message: *const AsciiChar) {
        let token = &self.current.clone();
        self.error_at(token, message);
    }

    fn error(&mut self, message: *const AsciiChar) {
        let token = &self.previous.clone();
        self.error_at(token, message);
    }

    fn error_at(&mut self, token: &Token, message: *const AsciiChar) {
        if self.panic_mode {
            return;
        }

        self.panic_mode = true;
        eprintln!("[line {}] Error", token.line);

        if token.ttype == TokenType::Eof {
            eprintln!(" at end");
        } else if token.ttype == TokenType::Error {
        } else {
            let s = unsafe { from_raw_parts(token.start, token.length) };
            let s = unsafe { from_utf8_unchecked(s) };
            eprint!(" at '{}'", s);
        }

        let s = unsafe { std::ffi::CStr::from_ptr(message as *const i8) };
        eprintln!(": {}", s.to_str().unwrap());
        self.had_error = true;
    }
}

#[repr(u8)]
#[derive(Clone)]
enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

impl From<u8> for Precedence {
    fn from(value: u8) -> Self {
        match value {
            0 => Precedence::None,
            1 => Precedence::Assignment,
            2 => Precedence::Or,
            3 => Precedence::And,
            4 => Precedence::Equality,
            5 => Precedence::Comparison,
            6 => Precedence::Term,
            7 => Precedence::Factor,
            8 => Precedence::Unary,
            9 => Precedence::Call,
            10 => Precedence::Primary,
            _ => Precedence::None,
        }
    }
}

struct ParseRule {
    prefix: Option<fn(&mut Parser)>,
    infix: Option<fn(&mut Parser)>,
    precedence: Precedence,
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! parse_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (input, bytes) = $value;

                    let source = CString::new(input).unwrap();
                    let scanner = &mut Scanner::new(source.as_bytes_with_nul().as_ptr());
                    let parser = &mut Parser::new(scanner);

                    let chunk = &mut Chunk::default();
                    parser.compile(chunk);

                    let mut expected = Chunk::default();
                    for byte in bytes {
                        expected.write(byte.into(), 1);
                    }
                    assert_eq!(chunk.code, expected.code);
                }
            )*
        }
    }

    parse_tests! {
        unary: (
            "-3",
            vec![
                OpCode::Constant.into(), 0,
                OpCode::Negate.into(),
                OpCode::Return.into(),
            ]
        ),
        addition: (
            "2 + 3",
            vec![
                OpCode::Constant.into(), 0,
                OpCode::Constant.into(), 1,
                OpCode::Add.into(),
                OpCode::Return.into(),
            ]
        ),
        subtraction: (
            "5 - 2",
            vec![
                OpCode::Constant.into(), 0,
                OpCode::Constant.into(), 1,
                OpCode::Subtract.into(),
                OpCode::Return.into(),
            ]
        ),
        multiplication: (
            "2 * 3",
            vec![
                OpCode::Constant.into(), 0,
                OpCode::Constant.into(), 1,
                OpCode::Multiply.into(),
                OpCode::Return.into(),
            ]
        ),
        division: (
            "6 / 2",
            vec![
                OpCode::Constant.into(), 0,
                OpCode::Constant.into(), 1,
                OpCode::Divide.into(),
                OpCode::Return.into(),
            ]
        ),
        grouping: (
            "(2 + 3) * 4",
            vec![
                OpCode::Constant.into(), 0,
                OpCode::Constant.into(), 1,
                OpCode::Add.into(),
                OpCode::Constant.into(), 2,
                OpCode::Multiply.into(),
                OpCode::Return.into(),
            ]
        ),
    }
}
