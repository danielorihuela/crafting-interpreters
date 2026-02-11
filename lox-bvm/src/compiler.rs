use crate::{AsciiChar, scanner::Scanner, token::TokenType};

pub fn compile(source: *const AsciiChar) {
    let mut line = -1;
    let mut scanner = Scanner::new(source);
    loop {
        let token = scanner.get_token();
        if token.line != line {
            print!("{:4} ", token.line);
            line = token.line;
        } else {
            print!("    | ");
        }

        let lexeme = unsafe { std::slice::from_raw_parts(token.start, token.length) };
        println!("{:2} '{:?}'", token.ttype, lexeme);

        if token.ttype == TokenType::Eof {
            break;
        }
    }
}
