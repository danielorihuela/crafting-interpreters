use crate::{
    AsciiChar,
    token::{Token, TokenType},
};

pub struct Scanner {
    start: *const AsciiChar,
    current: *const AsciiChar,
    line: isize,
}

impl Scanner {
    pub fn new(source: *const AsciiChar) -> Self {
        Self {
            start: source,
            current: source,
            line: 1,
        }
    }

    fn peek(&self) -> AsciiChar {
        unsafe { *self.current }
    }

    fn peek_at(&self, offset: usize) -> AsciiChar {
        unsafe { *self.current.add(offset) }
    }

    fn advance(&mut self) -> AsciiChar {
        let c = self.peek();
        self.current = unsafe { self.current.add(1) };
        c
    }

    fn is_end(&self) -> bool {
        unsafe { *self.current == b'\0' }
    }

    pub fn get_token(&mut self) -> Token {
        self.skip_noise();
        self.start = self.current;

        if self.is_end() {
            return self.make_token(TokenType::Eof);
        }

        let c = self.advance();
        match c {
            b'(' => self.make_token(TokenType::LeftParen),
            b')' => self.make_token(TokenType::RightParen),
            b'{' => self.make_token(TokenType::LeftBrace),
            b'}' => self.make_token(TokenType::RightBrace),
            b',' => self.make_token(TokenType::Comma),
            b'.' => self.make_token(TokenType::Dot),
            b'-' => self.make_token(TokenType::Minus),
            b'+' => self.make_token(TokenType::Plus),
            b';' => self.make_token(TokenType::Semicolon),
            b'?' => self.make_token(TokenType::QuestionMark),
            b'/' => self.make_token(TokenType::Slash),
            b'*' => self.make_token(TokenType::Star),
            b'!' => {
                if self.peek() == b'=' {
                    self.advance();
                    self.make_token(TokenType::BangEqual)
                } else {
                    self.make_token(TokenType::Bang)
                }
            }
            b'=' => {
                if self.peek() == b'=' {
                    self.advance();
                    self.make_token(TokenType::EqualEqual)
                } else {
                    self.make_token(TokenType::Equal)
                }
            }
            b'>' => {
                if self.peek() == b'=' {
                    self.advance();
                    self.make_token(TokenType::GreaterEqual)
                } else {
                    self.make_token(TokenType::Greater)
                }
            }
            b'<' => {
                if self.peek() == b'=' {
                    self.advance();
                    self.make_token(TokenType::LessEqual)
                } else {
                    self.make_token(TokenType::Less)
                }
            }
            b'"' => self.string(),
            x if x.is_ascii_digit() => self.number(),
            x if x.is_ascii_alphabetic() || x == b'_' => self.identifier(),
            _ => self.error_token("Unexpected character."),
        }
    }

    fn skip_noise(&mut self) {
        loop {
            let c = self.peek();
            match c {
                b' ' | b'\r' | b'\t' => {
                    self.advance();
                }
                b'\n' => {
                    self.line += 1;
                    self.advance();
                }
                b'/' => {
                    if self.peek_at(1) != b'/' {
                        break;
                    }

                    // Skip comment
                    while self.peek() != b'\n' && !self.is_end() {
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }

    fn string(&mut self) -> Token {
        while self.peek() != b'"' && !self.is_end() {
            if self.advance() == b'\n' {
                self.line += 1;
            }
        }

        if self.peek() == b'"' {
            self.advance();
            self.make_token(TokenType::String)
        } else {
            self.error_token("Unterminated string.")
        }
    }

    fn number(&mut self) -> Token {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == b'.' && self.peek_at(1).is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        self.make_token(TokenType::Number)
    }

    fn identifier(&mut self) -> Token {
        while self.peek().is_ascii_alphanumeric() || self.peek() == b'_' {
            self.advance();
        }

        let identifier_type = match unsafe { *self.start } {
            b'a' => self.check_keyword(1, 2, "nd".as_ptr() as *const AsciiChar, TokenType::And),
            b'c' => self.check_keyword(1, 4, "lass".as_ptr() as *const AsciiChar, TokenType::Class),
            b'e' => self.check_keyword(1, 3, "lse".as_ptr() as *const AsciiChar, TokenType::Else),
            b'f' => {
                if unsafe { self.current.offset_from(self.start) } > 1 {
                    match unsafe { *self.start.add(1) } {
                        b'a' => self.check_keyword(
                            2,
                            3,
                            "lse".as_ptr() as *const AsciiChar,
                            TokenType::False,
                        ),
                        b'o' => self.check_keyword(
                            2,
                            1,
                            "r".as_ptr() as *const AsciiChar,
                            TokenType::For,
                        ),
                        b'u' => self.check_keyword(
                            2,
                            1,
                            "n".as_ptr() as *const AsciiChar,
                            TokenType::Fun,
                        ),
                        _ => TokenType::Identifier,
                    }
                } else {
                    TokenType::Identifier
                }
            }
            b'i' => self.check_keyword(1, 1, "f".as_ptr() as *const AsciiChar, TokenType::If),
            b'n' => self.check_keyword(1, 2, "il".as_ptr() as *const AsciiChar, TokenType::Nil),
            b'o' => self.check_keyword(1, 1, "r".as_ptr() as *const AsciiChar, TokenType::Or),
            b'p' => self.check_keyword(1, 4, "rint".as_ptr() as *const AsciiChar, TokenType::Print),
            b'r' => self.check_keyword(
                1,
                5,
                "eturn".as_ptr() as *const AsciiChar,
                TokenType::Return,
            ),
            b's' => self.check_keyword(1, 4, "uper".as_ptr() as *const AsciiChar, TokenType::Super),
            b't' => {
                if unsafe { self.current.offset_from(self.start) } > 1 {
                    match unsafe { *self.start.add(1) } {
                        b'h' => self.check_keyword(
                            2,
                            2,
                            "is".as_ptr() as *const AsciiChar,
                            TokenType::This,
                        ),
                        b'r' => self.check_keyword(
                            2,
                            2,
                            "ue".as_ptr() as *const AsciiChar,
                            TokenType::True,
                        ),
                        _ => TokenType::Identifier,
                    }
                } else {
                    TokenType::Identifier
                }
            }
            b'v' => self.check_keyword(1, 2, "ar".as_ptr() as *const AsciiChar, TokenType::Var),
            b'w' => self.check_keyword(1, 4, "hile".as_ptr() as *const AsciiChar, TokenType::While),
            _ => TokenType::Identifier,
        };

        self.make_token(identifier_type)
    }

    fn make_token(&self, ttype: TokenType) -> Token {
        Token {
            ttype: ttype,
            start: self.start,
            length: unsafe { self.current.offset_from(self.start) as usize },
            line: self.line,
        }
    }

    fn error_token(&self, message: &str) -> Token {
        Token {
            ttype: TokenType::Error,
            start: message.as_ptr() as *const AsciiChar,
            length: message.len(),
            line: self.line,
        }
    }

    fn check_keyword(
        &self,
        start: usize,
        length: usize,
        rest: *const AsciiChar,
        ttype: TokenType,
    ) -> TokenType {
        if unsafe { self.current.offset_from(self.start) } as usize != start + length {
            return TokenType::Identifier;
        }

        let a = unsafe { std::slice::from_raw_parts(self.start.add(start), length) };
        let b = unsafe { std::slice::from_raw_parts(rest, length) };
        if a != b {
            return TokenType::Identifier;
        }

        ttype
    }
}
