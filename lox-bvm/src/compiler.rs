use std::{ffi::CString, mem::transmute, slice::from_raw_parts, str::from_utf8_unchecked};

use crate::{
    AsciiChar,
    collections::hashtable::HashTable,
    scanner::Scanner,
    types::{
        TokenType,
        chunk::Chunk,
        opcode::OpCode,
        token::Token,
        value::{Value, obj::Obj, string::ObjString},
    },
};

pub struct Parser<'a> {
    scanner: &'a mut Scanner,

    current: Token,
    previous: Token,

    chunk: *mut Chunk,
    objects: *mut *mut Obj,
    strings: *mut HashTable,

    had_error: bool,
    panic_mode: bool,
}

impl<'a> Parser<'a> {
    pub fn new(scanner: &'a mut Scanner, objects: *mut *mut Obj, strings: *mut HashTable) -> Self {
        Self {
            scanner,
            current: Token::default(),
            previous: Token::default(),
            chunk: std::ptr::null_mut(),
            objects,
            strings,
            had_error: false,
            panic_mode: false,
        }
    }

    pub fn compile(&mut self, chunk: &mut Chunk) -> bool {
        self.had_error = false;
        self.panic_mode = false;

        self.chunk = chunk;
        self.advance();

        while !self.match_type(TokenType::Eof) {
            self.declaration();
        }

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

    fn declaration(&mut self) {
        if self.match_type(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.panic_mode {
            self.synchronize();
        }
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.");

        if self.match_type(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_byte(OpCode::Nil);
        }
        self.consume(
            TokenType::Semicolon,
            CString::new("Expect ';' after variable declaration.")
                .unwrap()
                .as_bytes_with_nul()
                .as_ptr(),
        );

        self.define_variable(global);
    }

    fn parse_variable(&mut self, error_message: &str) -> u8 {
        self.consume(
            TokenType::Identifier,
            CString::new(error_message)
                .unwrap()
                .as_bytes_with_nul()
                .as_ptr(),
        );

        self.identifier_constant()
    }

    fn identifier_constant(&mut self) -> u8 {
        let obj_string = ObjString::new(
            self.previous.start,
            self.previous.length,
            self.objects,
            self.strings,
        );
        let value = Value::from(obj_string);
        self.make_constant(value)
    }

    fn define_variable(&mut self, global: u8) {
        self.emit_bytes(OpCode::DefineGlobal, global);
    }

    fn statement(&mut self) {
        if self.match_type(TokenType::Print) {
            self.print_statement();
        } else {
            self.expression_statement();
        }
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(
            TokenType::Semicolon,
            CString::new("Expect ';' after value.")
                .unwrap()
                .as_bytes_with_nul()
                .as_ptr(),
        );
        self.emit_byte(OpCode::Print);
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(
            TokenType::Semicolon,
            CString::new("Expect ';' after expression.")
                .unwrap()
                .as_bytes_with_nul()
                .as_ptr(),
        );
        self.emit_byte(OpCode::Pop);
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;

        while self.current.ttype != TokenType::Eof {
            if self.previous.ttype == TokenType::Semicolon {
                return;
            }

            match self.current.ttype {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => (),
            }

            self.advance();
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let can_assign = precedence.clone() as u8 <= Precedence::Assignment as u8;
        let prefix_rule = self
            .get_rule(self.previous.ttype.clone(), can_assign)
            .prefix;
        if let Some(prefix) = prefix_rule {
            prefix(self, can_assign);
        } else {
            self.error(
                CString::new("Expect expression.")
                    .unwrap()
                    .as_bytes_with_nul()
                    .as_ptr(),
            );
            return;
        }

        while precedence.clone() as u8
            <= self
                .get_rule(self.current.ttype.clone(), can_assign)
                .precedence as u8
        {
            self.advance();
            let infix_rule = self.get_rule(self.previous.ttype.clone(), can_assign).infix;
            if let Some(infix) = infix_rule {
                infix(self, can_assign);
            }
        }

        if can_assign && self.match_type(TokenType::Equal) {
            self.error(
                CString::new("Invalid assignment target.")
                    .unwrap()
                    .as_bytes_with_nul()
                    .as_ptr(),
            );
        }
    }

    fn consume(&mut self, ttype: TokenType, message: *const AsciiChar) {
        if self.current.ttype == ttype {
            self.advance();
            return;
        }

        self.error_at_current(message);
    }

    fn match_type(&mut self, ttype: TokenType) -> bool {
        if self.current.ttype != ttype {
            return false;
        }

        self.advance();
        true
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

    fn number(&mut self, can_assign: bool) {
        let value = unsafe { from_raw_parts(self.previous.start, self.previous.length) };
        let value = unsafe { from_utf8_unchecked(value) };
        let value: f64 = value.parse().unwrap();
        self.emit_constant(Value::from(value));
    }

    fn grouping(&mut self, can_assign: bool) {
        self.expression();
        self.consume(
            TokenType::RightParen,
            CString::new("Expect ')' after expression.")
                .unwrap()
                .as_bytes_with_nul()
                .as_ptr(),
        );
    }

    fn unary(&mut self, can_assign: bool) {
        let operator_type = self.previous.ttype.clone();

        self.parse_precedence(Precedence::Unary);

        match operator_type {
            TokenType::Bang => self.emit_byte(OpCode::Not),
            TokenType::Minus => self.emit_byte(OpCode::Negate),
            _ => (),
        }
    }

    fn binary(&mut self, can_assign: bool) {
        let operator_type = self.previous.ttype.clone();

        let rule = self.get_rule(operator_type.clone(), can_assign);
        self.parse_precedence(Precedence::from(rule.precedence as u8 + 1));

        match operator_type {
            TokenType::Plus => self.emit_byte(OpCode::Add),
            TokenType::Minus => self.emit_byte(OpCode::Subtract),
            TokenType::Star => self.emit_byte(OpCode::Multiply),
            TokenType::Slash => self.emit_byte(OpCode::Divide),
            TokenType::BangEqual => self.emit_bytes(OpCode::Equal, OpCode::Not),
            TokenType::EqualEqual => self.emit_byte(OpCode::Equal),
            TokenType::Greater => self.emit_byte(OpCode::Greater),
            TokenType::GreaterEqual => self.emit_bytes(OpCode::Less, OpCode::Not),
            TokenType::Less => self.emit_byte(OpCode::Less),
            TokenType::LessEqual => self.emit_bytes(OpCode::Greater, OpCode::Not),
            _ => (),
        }
    }

    fn literal(&mut self, can_assign: bool) {
        match self.previous.ttype.clone() {
            TokenType::False => self.emit_byte(OpCode::False),
            TokenType::True => self.emit_byte(OpCode::True),
            TokenType::Nil => self.emit_byte(OpCode::Nil),
            _ => unreachable!(),
        }
    }

    fn string(&mut self, can_assign: bool) {
        let start = unsafe { self.previous.start.add(1) };
        let end = self.previous.length - 2;
        let string = ObjString::new(start, end, self.objects, self.strings);
        let obj = Value::from(string);
        self.emit_constant(obj);
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(can_assign);
    }

    fn named_variable(&mut self, can_assign: bool) {
        let arg = self.identifier_constant();

        if can_assign && self.match_type(TokenType::Equal) {
            self.expression();
            self.emit_bytes(OpCode::SetGlobal, arg);
        } else {
            self.emit_bytes(OpCode::GetGlobal, arg);
        }
    }

    fn get_rule(&self, ttype: TokenType, can_assign: bool) -> ParseRule {
        match ttype {
            TokenType::LeftParen => ParseRule {
                prefix: Some(|parser, can_assign| parser.grouping(can_assign)),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Minus => ParseRule {
                prefix: Some(|parser, can_assign| parser.unary(can_assign)),
                infix: Some(|parser, can_assign| parser.binary(can_assign)),
                precedence: Precedence::Term,
            },
            TokenType::Plus => ParseRule {
                prefix: None,
                infix: Some(|parser, can_assign| parser.binary(can_assign)),
                precedence: Precedence::Term,
            },
            TokenType::Star => ParseRule {
                prefix: None,
                infix: Some(|parser, can_assign| parser.binary(can_assign)),
                precedence: Precedence::Factor,
            },
            TokenType::Slash => ParseRule {
                prefix: None,
                infix: Some(|parser, can_assign| parser.binary(can_assign)),
                precedence: Precedence::Factor,
            },
            TokenType::Number => ParseRule {
                prefix: Some(|parser, can_assign| parser.number(can_assign)),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::True | TokenType::False | TokenType::Nil => ParseRule {
                prefix: Some(|parser, can_assign| parser.literal(can_assign)),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Bang => ParseRule {
                prefix: Some(|parser, can_assign| parser.unary(can_assign)),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::BangEqual | TokenType::EqualEqual => ParseRule {
                prefix: None,
                infix: Some(|parser, can_assign| parser.binary(can_assign)),
                precedence: Precedence::Equality,
            },
            TokenType::Greater
            | TokenType::GreaterEqual
            | TokenType::Less
            | TokenType::LessEqual => ParseRule {
                prefix: None,
                infix: Some(|parser, can_assign| parser.binary(can_assign)),
                precedence: Precedence::Comparison,
            },
            TokenType::String => ParseRule {
                prefix: Some(|parser, can_assign| parser.string(can_assign)),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Identifier => ParseRule {
                prefix: Some(|parser, can_assign| parser.variable(can_assign)),
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

    fn emit_byte(&mut self, b: impl Into<u8>) {
        unsafe { (*self.chunk).write(b.into(), self.previous.line as usize) }
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
        eprint!("[line {}] Error", token.line);

        if token.ttype == TokenType::Eof {
            eprint!(" at end");
        } else if token.ttype == TokenType::Error {
        } else {
            let s = unsafe { from_raw_parts(token.start, token.length) };
            let s = String::from_utf8_lossy(s);
            eprint!(" at '{}'", s);
        }

        let s = unsafe { std::ffi::CStr::from_ptr(message as *const i8) };
        eprintln!(": {}", s.to_string_lossy());
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
        if value > Precedence::Primary as u8 {
            Precedence::None
        } else {
            unsafe { transmute::<u8, Precedence>(value) }
        }
    }
}

struct ParseRule {
    prefix: Option<fn(&mut Parser, bool)>,
    infix: Option<fn(&mut Parser, bool)>,
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
                    let parser = &mut Parser::new(scanner, std::ptr::null_mut(), std::ptr::null_mut());

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
            "-3;",
            vec![
                OpCode::Constant.into(), 0,
                OpCode::Negate.into(),
                OpCode::Pop.into(),
                OpCode::Return.into(),
            ]
        ),
        addition: (
            "2 + 3;",
            vec![
                OpCode::Constant.into(), 0,
                OpCode::Constant.into(), 1,
                OpCode::Add.into(),
                OpCode::Pop.into(),
                OpCode::Return.into(),
            ]
        ),
        subtraction: (
            "5 - 2;",
            vec![
                OpCode::Constant.into(), 0,
                OpCode::Constant.into(), 1,
                OpCode::Subtract.into(),
                OpCode::Pop.into(),
                OpCode::Return.into(),
            ]
        ),
        multiplication: (
            "2 * 3;",
            vec![
                OpCode::Constant.into(), 0,
                OpCode::Constant.into(), 1,
                OpCode::Multiply.into(),
                OpCode::Pop.into(),
                OpCode::Return.into(),
            ]
        ),
        division: (
            "6 / 2;",
            vec![
                OpCode::Constant.into(), 0,
                OpCode::Constant.into(), 1,
                OpCode::Divide.into(),
                OpCode::Pop.into(),
                OpCode::Return.into(),
            ]
        ),
        grouping: (
            "(2 + 3) * 4;",
            vec![
                OpCode::Constant.into(), 0,
                OpCode::Constant.into(), 1,
                OpCode::Add.into(),
                OpCode::Constant.into(), 2,
                OpCode::Multiply.into(),
                OpCode::Pop.into(),
                OpCode::Return.into(),
            ]
        ),
        literal: (
            "true;",
            vec![
                OpCode::True,
                OpCode::Pop.into(),
                OpCode::Return.into(),
            ]
        ),
        not: (
            "!true;",
            vec![
                OpCode::True,
                OpCode::Not,
                OpCode::Pop.into(),
                OpCode::Return.into(),
            ]
        ),
        not_equal: (
            "2 != 3",
            vec![
                OpCode::Constant.into(), 0,
                OpCode::Constant.into(), 1,
                OpCode::Equal.into(),
                OpCode::Not.into(),
                OpCode::Pop.into(),
                OpCode::Return.into(),
            ]
        ),
        equal: (
            "2 == 3",
            vec![
                OpCode::Constant.into(), 0,
                OpCode::Constant.into(), 1,
                OpCode::Equal.into(),
                OpCode::Pop.into(),
                OpCode::Return.into(),
            ]
        ),
        greater: (
            "2 > 3",
            vec![
                OpCode::Constant.into(), 0,
                OpCode::Constant.into(), 1,
                OpCode::Greater.into(),
                OpCode::Pop.into(),
                OpCode::Return.into(),
            ]
        ),
        greater_equal: (
            "2 >= 3",
            vec![
                OpCode::Constant.into(), 0,
                OpCode::Constant.into(), 1,
                OpCode::Less.into(),
                OpCode::Not.into(),
                OpCode::Pop.into(),
                OpCode::Return.into(),
            ]
        ),
        less: (
            "2 < 3",
            vec![
                OpCode::Constant.into(), 0,
                OpCode::Constant.into(), 1,
                OpCode::Less.into(),
                OpCode::Pop.into(),
                OpCode::Return.into(),
            ]
        ),
        less_equal: (
            "2 <= 3",
            vec![
                OpCode::Constant.into(), 0,
                OpCode::Constant.into(), 1,
                OpCode::Greater.into(),
                OpCode::Not.into(),
                OpCode::Pop.into(),
                OpCode::Return.into(),
            ]
        ),
    }
}
