use crate::types::{TokenType, token::Token};

pub struct Scanner<'src> {
    source: &'src [u8],
    start: usize,
    current: usize,
    line: isize,
}

impl<'src> Scanner<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            source: source.as_bytes(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn get_token(&mut self) -> Token<'src> {
        self.skip_noise();
        self.start = self.current;

        if self.is_end() {
            return self.make_token(TokenType::Eof);
        }

        let c = self.peek();
        self.advance();
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
            b':' => self.make_token(TokenType::Colon),
            b'/' => self.make_token(TokenType::Slash),
            b'*' => self.make_token(TokenType::Star),
            b'!' => {
                if !self.is_end() && self.peek() == b'=' {
                    self.advance();
                    self.make_token(TokenType::BangEqual)
                } else {
                    self.make_token(TokenType::Bang)
                }
            }
            b'=' => {
                if !self.is_end() && self.peek() == b'=' {
                    self.advance();
                    self.make_token(TokenType::EqualEqual)
                } else {
                    self.make_token(TokenType::Equal)
                }
            }
            b'<' => {
                if !self.is_end() && self.peek() == b'=' {
                    self.advance();
                    self.make_token(TokenType::LessEqual)
                } else {
                    self.make_token(TokenType::Less)
                }
            }
            b'>' => {
                if !self.is_end() && self.peek() == b'=' {
                    self.advance();
                    self.make_token(TokenType::GreaterEqual)
                } else {
                    self.make_token(TokenType::Greater)
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
            if self.is_end() {
                break;
            }

            match self.peek() {
                b' ' | b'\r' | b'\t' => {
                    self.advance();
                }
                b'\n' => {
                    self.advance();
                    self.line += 1;
                }
                b'/' => {
                    self.advance();

                    if self.is_end() || self.peek() != b'/' {
                        self.recede();
                        break;
                    }

                    // Skip comment
                    while !self.is_end() && self.peek() != b'\n' {
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }

    fn string(&mut self) -> Token<'src> {
        while !self.is_end() && self.peek() != b'"' {
            if self.peek() == b'\n' {
                self.line += 1;
            }
            self.advance();
        }

        if !self.is_end() && self.peek() == b'"' {
            self.advance();
            self.make_token(TokenType::String)
        } else {
            self.error_token("Unterminated string.")
        }
    }

    fn number(&mut self) -> Token<'src> {
        while !self.is_end() && self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.current + 1 < self.source.len()
            && (self.peek() == b'.' && self.peek_at(1).is_ascii_digit())
        {
            self.advance();
            while !self.is_end() && self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        self.make_token(TokenType::Number)
    }

    fn identifier(&mut self) -> Token<'src> {
        while !self.is_end() && (self.peek().is_ascii_alphanumeric() || self.peek() == b'_') {
            self.advance();
        }

        let lexeme = &self.source[self.start..self.current];
        let identifier_type = match lexeme.len() {
            2 => match lexeme[0] {
                b'i' if lexeme == b"if" => TokenType::If,
                b'o' if lexeme == b"or" => TokenType::Or,
                _ => TokenType::Identifier,
            },
            3 => match lexeme[0] {
                b'a' if lexeme == b"and" => TokenType::And,
                b'f' if lexeme == b"for" => TokenType::For,
                b'f' if lexeme == b"fun" => TokenType::Fun,
                b'n' if lexeme == b"nil" => TokenType::Nil,
                b'v' if lexeme == b"var" => TokenType::Var,
                _ => TokenType::Identifier,
            },
            4 => match lexeme[0] {
                b'e' if lexeme == b"else" => TokenType::Else,
                b't' if lexeme == b"this" => TokenType::This,
                b't' if lexeme == b"true" => TokenType::True,
                _ => TokenType::Identifier,
            },
            5 => match lexeme[0] {
                b'c' if lexeme == b"class" => TokenType::Class,
                b'f' if lexeme == b"false" => TokenType::False,
                b'p' if lexeme == b"print" => TokenType::Print,
                b's' if lexeme == b"super" => TokenType::Super,
                b'w' if lexeme == b"while" => TokenType::While,
                _ => TokenType::Identifier,
            },
            6 if lexeme[0] == b'r' && lexeme == b"return" => TokenType::Return,
            _ => TokenType::Identifier,
        };

        self.make_token(identifier_type)
    }
}

impl<'src> Scanner<'src> {
    fn peek(&self) -> u8 {
        self.source[self.current]
    }

    fn peek_at(&self, offset: usize) -> u8 {
        self.source[self.current + offset]
    }

    fn advance(&mut self) {
        self.current += 1;
    }

    fn recede(&mut self) {
        self.current -= 1;
    }

    fn is_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn make_token(&self, ttype: TokenType) -> Token<'src> {
        Token {
            ttype,
            lexeme: unsafe { str::from_utf8_unchecked(&self.source[self.start..self.current]) },
            line: self.line,
        }
    }

    fn error_token(&self, message: &'static str) -> Token<'src> {
        Token {
            ttype: TokenType::Error,
            lexeme: message,
            line: self.line,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    macro_rules! scanner_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (input, token_type) = $value;

                    let scanner = &mut Scanner::new(input);
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
        let scanner = &mut Scanner::new(input);

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
            assert_eq!(token.lexeme.len(), expected.1);
            assert_eq!(token.line, expected.2);
        }
    }

    #[cfg_attr(miri, ignore)]
    #[test]
    fn test_bench() {
        let input = include_str!("../../benchmark.lox");
        let input = input.repeat(200000);

        let start = Instant::now();

        let mut scanner = Scanner::new(&input);
        loop {
            let token = scanner.get_token();
            if token.ttype == TokenType::Eof {
                break;
            }
        }

        let string_time = start.elapsed();
        println!("Scanner took {:?}", string_time);
    }
}
