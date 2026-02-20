use crate::{
    AsciiChar,
    types::{TokenType, token::Token},
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

    pub fn get_token(&mut self) -> Token {
        self.skip_noise();
        self.start = self.current;

        if self.is_end() {
            return self.make_token(TokenType::Eof);
        }

        let c = self.advance();
        match c {
            x if TokenType::is_single_byte(x) => {
                self.make_token(TokenType::single_byte(x).expect("Already matched"))
            }
            x if TokenType::is_comparison(x) => {
                if self.peek() == b'=' {
                    self.advance();
                    self.make_token(TokenType::comparison(x, Some(b'=')).expect("Already matched"))
                } else {
                    self.make_token(TokenType::comparison(x, None).expect("Already matched"))
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
            b'a' => self.check_keyword(1, 2, "nd".as_ptr(), TokenType::And),
            b'c' => self.check_keyword(1, 4, "lass".as_ptr(), TokenType::Class),
            b'e' => self.check_keyword(1, 3, "lse".as_ptr(), TokenType::Else),
            b'f' => {
                if unsafe { self.current.offset_from(self.start) } > 1 {
                    match unsafe { *self.start.add(1) } {
                        b'a' => self.check_keyword(2, 3, "lse".as_ptr(), TokenType::False),
                        b'o' => self.check_keyword(2, 1, "r".as_ptr(), TokenType::For),
                        b'u' => self.check_keyword(2, 1, "n".as_ptr(), TokenType::Fun),
                        _ => TokenType::Identifier,
                    }
                } else {
                    TokenType::Identifier
                }
            }
            b'i' => self.check_keyword(1, 1, "f".as_ptr(), TokenType::If),
            b'n' => self.check_keyword(1, 2, "il".as_ptr(), TokenType::Nil),
            b'o' => self.check_keyword(1, 1, "r".as_ptr(), TokenType::Or),
            b'p' => self.check_keyword(1, 4, "rint".as_ptr(), TokenType::Print),
            b'r' => self.check_keyword(1, 5, "eturn".as_ptr(), TokenType::Return),
            b's' => self.check_keyword(1, 4, "uper".as_ptr(), TokenType::Super),
            b't' => {
                if unsafe { self.current.offset_from(self.start) } > 1 {
                    match unsafe { *self.start.add(1) } {
                        b'h' => self.check_keyword(2, 2, "is".as_ptr(), TokenType::This),
                        b'r' => self.check_keyword(2, 2, "ue".as_ptr(), TokenType::True),
                        _ => TokenType::Identifier,
                    }
                } else {
                    TokenType::Identifier
                }
            }
            b'v' => self.check_keyword(1, 2, "ar".as_ptr(), TokenType::Var),
            b'w' => self.check_keyword(1, 4, "hile".as_ptr(), TokenType::While),
            _ => TokenType::Identifier,
        };

        self.make_token(identifier_type)
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

impl Scanner {
    fn peek(&self) -> AsciiChar {
        self.peek_at(0)
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
        self.peek() == b'\0'
    }

    fn make_token(&self, ttype: TokenType) -> Token {
        Token {
            ttype,
            start: self.start,
            length: unsafe { self.current.offset_from(self.start) as usize },
            line: self.line,
        }
    }

    fn error_token(&self, message: &str) -> Token {
        Token {
            ttype: TokenType::Error,
            start: message.as_ptr(),
            length: message.len(),
            line: self.line,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use super::*;

    macro_rules! scanner_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (input, token_type) = $value;

                    let source = CString::new(input).unwrap();
                    let scanner = &mut Scanner::new(source.as_bytes_with_nul().as_ptr());
                    let token = scanner.get_token();
                    assert_eq!(token.ttype, token_type);
                }
            )*
        }
    }

    scanner_tests!(
        // Single-character tokens
        left_paren: ("(", TokenType::LeftParen),
        right_paren: (")", TokenType::RightParen),
        left_brace: ("{", TokenType::LeftBrace),
        right_brace: ("}", TokenType::RightBrace),
        comma: (",", TokenType::Comma),
        dot: (".", TokenType::Dot),
        minus: ("-", TokenType::Minus),
        plus: ("+", TokenType::Plus),
        semicolon: (";", TokenType::Semicolon),
        question_mark: ("?", TokenType::QuestionMark),
        slash: ("/", TokenType::Slash),
        star: ("*", TokenType::Star),

        // Comparison operators
        bang: ("!", TokenType::Bang),
        bang_equal: ("!=", TokenType::BangEqual),
        equal: ("=", TokenType::Equal),
        equal_equal: ("==", TokenType::EqualEqual),
        greater: (">", TokenType::Greater),
        greater_equal: (">=", TokenType::GreaterEqual),
        less: ("<", TokenType::Less),
        less_equal: ("<=", TokenType::LessEqual),

        // Keywords
        and: ("and", TokenType::And),
        class: ("class", TokenType::Class),
        else_t: ("else", TokenType::Else),
        false_t: ("false", TokenType::False),
        fun: ("fun", TokenType::Fun),
        for_t: ("for", TokenType::For),
        if_t: ("if", TokenType::If),
        nil: ("nil", TokenType::Nil),
        or: ("or", TokenType::Or),
        print: ("print", TokenType::Print),
        return_t: ("return", TokenType::Return),
        super_t: ("super", TokenType::Super),
        this_t: ("this", TokenType::This),
        true_t: ("true", TokenType::True),
        var: ("var", TokenType::Var),
        while_t: ("while", TokenType::While),

        unterminated_string: ("\"error", TokenType::Error),
        unexpected_character: ("#", TokenType::Error),
        eof: ("", TokenType::Eof),

        number: ("123", TokenType::Number),
        float: ("123.45", TokenType::Number),
        string: ("\"text\"", TokenType::String),

        skip_comment: ("// Skip this part\nfun", TokenType::Fun),
    );

    #[test]
    fn complex_expression() {
        let input = "if (value >= 10) {\n print value;\n }";
        let source = CString::new(input).unwrap();
        let scanner = &mut Scanner::new(source.as_bytes_with_nul().as_ptr());

        let tokens = vec![
            (TokenType::If, 2, 1),
            (TokenType::LeftParen, 1, 1),
            (TokenType::Identifier, 5, 1),
            (TokenType::GreaterEqual, 2, 1),
            (TokenType::Number, 2, 1),
            (TokenType::RightParen, 1, 1),
            (TokenType::LeftBrace, 1, 1),
            (TokenType::Print, 5, 2),
            (TokenType::Identifier, 5, 2),
            (TokenType::Semicolon, 1, 2),
            (TokenType::RightBrace, 1, 3),
            (TokenType::Eof, 0, 3),
        ];

        for expected in tokens {
            let token = scanner.get_token();
            assert_eq!(token.ttype, expected.0);
            assert_eq!(token.length, expected.1);
            assert_eq!(token.line, expected.2);
        }
    }
}
