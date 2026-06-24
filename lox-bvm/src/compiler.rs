use std::{ffi::CString, mem::transmute, slice::from_raw_parts, str::from_utf8_unchecked};

use crate::{
    AsciiChar,
    collections::hashtable::HashTable,
    scanner::Scanner,
    types::{
        TokenType,
        opcode::OpCode,
        token::Token,
        value::{Value, function::ObjFunction, obj::Obj, string::ObjString},
    },
};

pub struct Parser<'a> {
    scanner: &'a mut Scanner,
    compiler: *mut Compiler,

    current: Token,
    previous: Token,

    objects: *mut *mut Obj,
    strings: *mut HashTable,

    had_error: bool,
    panic_mode: bool,
}

impl<'a> Parser<'a> {
    pub fn new(
        scanner: &'a mut Scanner,
        compiler: *mut Compiler,
        objects: *mut *mut Obj,
        strings: *mut HashTable,
    ) -> Self {
        Self {
            scanner,
            compiler,
            current: Token::default(),
            previous: Token::default(),
            objects,
            strings,
            had_error: false,
            panic_mode: false,
        }
    }

    pub fn compile(&mut self) -> *mut ObjFunction {
        self.had_error = false;
        self.panic_mode = false;

        self.advance();

        while !self.match_type(TokenType::Eof) {
            self.declaration();
        }

        let function = self.end_compiler();

        if !self.had_error {
            function
        } else {
            std::ptr::null_mut()
        }
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
        if self.match_type(TokenType::Fun) {
            self.fun_declaration();
        } else if self.match_type(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.panic_mode {
            self.synchronize();
        }
    }

    fn fun_declaration(&mut self) {
        let global = self.parse_variable("Expect function name.");
        self.mark_initialized();
        self.function(FunctionType::Function);
        self.define_variable(global);
    }

    fn function(&mut self, ftype: FunctionType) {
        let compiler = Compiler::new(
            ftype,
            self.objects,
            self.strings,
            self.compiler,
            self.previous.start,
            self.previous.length,
        );
        self.compiler = Box::into_raw(Box::new(compiler));

        self.begin_scope();

        self.consume(
            TokenType::LeftParen,
            CString::new("Expect '(' after function name.")
                .unwrap()
                .as_bytes_with_nul()
                .as_ptr(),
        );

        if self.current.ttype != TokenType::RightParen {
            loop {
                unsafe { (*(*self.compiler).function).arity += 1 };
                if unsafe { (*(*self.compiler).function).arity } > 255 {
                    self.error_at_current(
                        CString::new("Can't have more than 255 parameters.")
                            .unwrap()
                            .as_bytes_with_nul()
                            .as_ptr(),
                    );
                }

                let constant = self.parse_variable("Expect parameter name.");
                self.define_variable(constant);

                if !self.match_type(TokenType::Comma) {
                    break;
                }
            }
        }

        self.consume(
            TokenType::RightParen,
            CString::new("Expect ')' after parameters.")
                .unwrap()
                .as_bytes_with_nul()
                .as_ptr(),
        );
        self.consume(
            TokenType::LeftBrace,
            CString::new("Expect '{' before function body.")
                .unwrap()
                .as_bytes_with_nul()
                .as_ptr(),
        );
        self.block();

        let function = self.end_compiler();
        let constant = self.make_constant(Value::from(function));
        self.emit_bytes(OpCode::Constant, constant);
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

    fn declare_variable(&mut self) {
        if unsafe { (*self.compiler).scope_depth } == 0 {
            return;
        }

        for i in (0..unsafe { (*self.compiler).local_count }).rev() {
            let local = &unsafe { &(*self.compiler).locals[i] };
            if local.depth != -1 && local.depth < unsafe { (*self.compiler).scope_depth } {
                break;
            }

            if self.identifiers_equal(&local.name, &self.previous) {
                self.error(
                    CString::new("Already a variable with this name in this scope.")
                        .unwrap()
                        .as_bytes_with_nul()
                        .as_ptr(),
                );
            }
        }

        self.add_local();
    }

    fn identifiers_equal(&self, a: &Token, b: &Token) -> bool {
        if a.length != b.length {
            return false;
        }

        let a_slice = unsafe { from_raw_parts(a.start, a.length) };
        let b_slice = unsafe { from_raw_parts(b.start, b.length) };

        a_slice == b_slice
    }

    fn add_local(&mut self) {
        if unsafe { (*self.compiler).local_count } == (u8::MAX as usize + 1) {
            self.error(
                CString::new("Too many local variables in function.")
                    .unwrap()
                    .as_bytes_with_nul()
                    .as_ptr(),
            );
            return;
        }

        let local = &mut unsafe { &mut (*self.compiler).locals[(*self.compiler).local_count] };
        local.name = self.previous.clone();
        local.depth = -1;
        unsafe { (*self.compiler).local_count += 1 };
    }

    fn parse_variable(&mut self, error_message: &str) -> u8 {
        self.consume(
            TokenType::Identifier,
            CString::new(error_message)
                .unwrap()
                .as_bytes_with_nul()
                .as_ptr(),
        );

        self.declare_variable();
        if unsafe { (*self.compiler).scope_depth } > 0 {
            return 0;
        }

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
        if unsafe { (*self.compiler).scope_depth } > 0 {
            self.mark_initialized();
            return;
        }

        self.emit_bytes(OpCode::DefineGlobal, global);
    }

    fn mark_initialized(&mut self) {
        if unsafe { (*self.compiler).scope_depth } == 0 {
            return;
        }

        let local = &mut unsafe { &mut (*self.compiler).locals[(*self.compiler).local_count - 1] };
        local.depth = unsafe { (*self.compiler).scope_depth };
    }

    fn statement(&mut self) {
        if self.match_type(TokenType::Print) {
            self.print_statement();
        } else if self.match_type(TokenType::If) {
            self.if_statement();
        } else if self.match_type(TokenType::Return) {
            self.return_statement();
        } else if self.match_type(TokenType::While) {
            self.while_statement();
        } else if self.match_type(TokenType::For) {
            self.for_statement();
        } else if self.match_type(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
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

    fn if_statement(&mut self) {
        self.consume(
            TokenType::LeftParen,
            CString::new("Expect '(' after 'if'.")
                .unwrap()
                .as_bytes_with_nul()
                .as_ptr(),
        );
        self.expression();
        self.consume(
            TokenType::RightParen,
            CString::new("Expect ')' after condition.")
                .unwrap()
                .as_bytes_with_nul()
                .as_ptr(),
        );

        let then_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::Pop);
        self.statement();

        let else_jump = self.emit_jump(OpCode::Jump);

        self.patch_jump(then_jump);
        self.emit_byte(OpCode::Pop);

        if self.match_type(TokenType::Else) {
            self.statement();
        }
        self.patch_jump(else_jump);
    }

    fn return_statement(&mut self) {
        if unsafe { &(*self.compiler).ftype } == &FunctionType::Script {
            self.error(
                CString::new("Can't return from top-level code.")
                    .unwrap()
                    .as_bytes_with_nul()
                    .as_ptr(),
            );
        }

        if self.match_type(TokenType::Semicolon) {
            self.emit_return();
        } else {
            self.expression();
            self.consume(
                TokenType::Semicolon,
                CString::new("Expect ';' after return value.")
                    .unwrap()
                    .as_bytes_with_nul()
                    .as_ptr(),
            );
            self.emit_byte(OpCode::Return);
        }
    }

    fn while_statement(&mut self) {
        let loop_start = unsafe { (*(*self.compiler).function).chunk.code.count };
        self.consume(
            TokenType::LeftParen,
            CString::new("Expect '(' after 'while'.")
                .unwrap()
                .as_bytes_with_nul()
                .as_ptr(),
        );
        self.expression();
        self.consume(
            TokenType::RightParen,
            CString::new("Expect ')' after condition.")
                .unwrap()
                .as_bytes_with_nul()
                .as_ptr(),
        );

        let exit_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::Pop);
        self.statement();
        self.emit_loop(loop_start);

        self.patch_jump(exit_jump);
        self.emit_byte(OpCode::Pop);
    }

    fn for_statement(&mut self) {
        self.begin_scope();
        self.consume(
            TokenType::LeftParen,
            CString::new("Expect '(' after 'for'.")
                .unwrap()
                .as_bytes_with_nul()
                .as_ptr(),
        );
        if self.match_type(TokenType::Semicolon) {
            // No initializer.
        } else if self.match_type(TokenType::Var) {
            self.var_declaration();
        } else {
            self.expression_statement();
        }

        let mut loop_start = unsafe { (*(*self.compiler).function).chunk.code.count };
        let mut exit_jump = -1;
        if !self.match_type(TokenType::Semicolon) {
            self.expression();
            self.consume(
                TokenType::Semicolon,
                CString::new("Expect ';' after loop condition.")
                    .unwrap()
                    .as_bytes_with_nul()
                    .as_ptr(),
            );

            exit_jump = self.emit_jump(OpCode::JumpIfFalse) as isize;
            self.emit_byte(OpCode::Pop);
        }

        if !self.match_type(TokenType::RightParen) {
            let body_jump = self.emit_jump(OpCode::Jump);
            let increment_start = unsafe { (*(*self.compiler).function).chunk.code.count };
            self.expression();
            self.emit_byte(OpCode::Pop);
            self.consume(
                TokenType::RightParen,
                CString::new("Expect ')' after for clauses.")
                    .unwrap()
                    .as_bytes_with_nul()
                    .as_ptr(),
            );

            self.emit_loop(loop_start);
            loop_start = increment_start;
            self.patch_jump(body_jump);
        }

        self.statement();
        self.emit_loop(loop_start);
        if exit_jump != -1 {
            self.patch_jump(exit_jump as usize);
            self.emit_byte(OpCode::Pop);
        }
        self.end_scope();
    }

    fn emit_loop(&mut self, loop_start: usize) {
        self.emit_byte(OpCode::Loop);

        let offset = unsafe { (*(*self.compiler).function).chunk.code.count - loop_start + 2 };
        if offset > u16::MAX as usize {
            self.error(
                CString::new("Loop body too large.")
                    .unwrap()
                    .as_bytes_with_nul()
                    .as_ptr(),
            );
        }

        self.emit_byte(((offset >> 8) & 0xff) as u8);
        self.emit_byte((offset & 0xff) as u8);
    }

    fn emit_jump(&mut self, instruction: OpCode) -> usize {
        self.emit_byte(instruction);
        self.emit_byte(0xff);
        self.emit_byte(0xff);

        unsafe { (*(*self.compiler).function).chunk.code.count - 2 }
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = unsafe { (*(*self.compiler).function).chunk.code.count - offset - 2 };
        if jump > u16::MAX as usize {
            self.error(
                CString::new("Too much code to jump over.")
                    .unwrap()
                    .as_bytes_with_nul()
                    .as_ptr(),
            );
        }

        unsafe {
            (&mut (*(*self.compiler).function).chunk.code)[offset] = ((jump >> 8) & 0xff) as u8;
            (&mut (*(*self.compiler).function).chunk.code)[offset + 1] = (jump & 0xff) as u8;
        }
    }

    fn begin_scope(&mut self) {
        unsafe { (*self.compiler).scope_depth += 1 };
    }

    fn block(&mut self) {
        while self.current.ttype != TokenType::RightBrace && self.current.ttype != TokenType::Eof {
            self.declaration();
        }

        self.consume(
            TokenType::RightBrace,
            CString::new("Expect '}' after block.")
                .unwrap()
                .as_bytes_with_nul()
                .as_ptr(),
        );
    }

    fn end_scope(&mut self) {
        unsafe { (*self.compiler).scope_depth -= 1 };

        while unsafe {
            (*self.compiler).local_count > 0
                && (*self.compiler).locals[(*self.compiler).local_count - 1].depth
                    > (*self.compiler).scope_depth
        } {
            self.emit_byte(OpCode::Pop);
            unsafe { (*self.compiler).local_count -= 1 };
        }
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

    fn end_compiler(&mut self) -> *mut ObjFunction {
        self.emit_return();

        #[cfg(debug_assertions)]
        {
            if !self.had_error {
                let name = if unsafe { (*(*self.compiler).function).name.is_null() } {
                    "script".to_string()
                } else {
                    let name = unsafe { (*(*self.compiler).function).name };
                    let name = Value::from(name);
                    name.to_string()
                };
                unsafe { (*(*self.compiler).function).chunk.disassemble(&name) };
            }
        }

        let function = unsafe { (*self.compiler).function };
        if unsafe { (*self.compiler).enclosing.is_null() } {
            return function;
        }

        self.compiler = unsafe { &mut *(*self.compiler).enclosing };

        function
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
        let mut set_opcode = OpCode::SetGlobal;
        let mut get_opcode = OpCode::GetGlobal;
        let mut arg = self.resolve_local();
        if arg != -1 {
            set_opcode = OpCode::SetLocal;
            get_opcode = OpCode::GetLocal;
        } else {
            arg = self.identifier_constant() as isize;
            set_opcode = OpCode::SetGlobal;
            get_opcode = OpCode::GetGlobal;
        }

        if can_assign && self.match_type(TokenType::Equal) {
            self.expression();
            self.emit_bytes(set_opcode, arg as u8);
        } else {
            self.emit_bytes(get_opcode, arg as u8);
        }
    }

    fn resolve_local(&mut self) -> isize {
        for i in (0..unsafe { (*self.compiler).local_count }).rev() {
            let local = unsafe { &(*self.compiler).locals[i] };
            if self.identifiers_equal(&local.name, &self.previous) {
                if local.depth == -1 {
                    self.error(
                        CString::new("Can't read local variable in its own initializer.")
                            .unwrap()
                            .as_bytes_with_nul()
                            .as_ptr(),
                    );
                }
                return i as isize;
            }
        }

        -1
    }

    fn and(&mut self, can_assign: bool) {
        let end_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::Pop);
        self.parse_precedence(Precedence::And);
        self.patch_jump(end_jump);
    }

    fn or(&mut self, can_assign: bool) {
        let else_jump = self.emit_jump(OpCode::JumpIfFalse);
        let end_jump = self.emit_jump(OpCode::Jump);

        self.patch_jump(else_jump);
        self.emit_byte(OpCode::Pop);

        self.parse_precedence(Precedence::Or);
        self.patch_jump(end_jump);
    }

    fn call(&mut self, can_assign: bool) {
        let arg_count = self.argument_list();
        self.emit_bytes(OpCode::Call, arg_count);
    }

    fn argument_list(&mut self) -> u8 {
        let mut arg_count = 0;
        if self.current.ttype != TokenType::RightParen {
            loop {
                self.expression();
                if arg_count == 255 {
                    self.error(
                        CString::new("Can't have more than 255 arguments.")
                            .unwrap()
                            .as_bytes_with_nul()
                            .as_ptr(),
                    );
                }
                arg_count += 1;

                if !self.match_type(TokenType::Comma) {
                    break;
                }
            }
        }

        self.consume(
            TokenType::RightParen,
            CString::new("Expect ')' after arguments.")
                .unwrap()
                .as_bytes_with_nul()
                .as_ptr(),
        );

        arg_count
    }

    fn get_rule(&self, ttype: TokenType, can_assign: bool) -> ParseRule {
        match ttype {
            TokenType::LeftParen => ParseRule {
                prefix: Some(|parser, can_assign| parser.grouping(can_assign)),
                infix: Some(|parser, can_assign| parser.call(can_assign)),
                precedence: Precedence::Call,
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
            TokenType::And => ParseRule {
                prefix: None,
                infix: Some(|parser, can_assign| parser.and(can_assign)),
                precedence: Precedence::And,
            },
            TokenType::Or => ParseRule {
                prefix: None,
                infix: Some(|parser, can_assign| parser.or(can_assign)),
                precedence: Precedence::Or,
            },
            _ => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        }
    }

    fn emit_byte(&mut self, b: impl Into<u8>) {
        unsafe {
            (*(*self.compiler).function)
                .chunk
                .write(b.into(), self.previous.line as usize)
        }
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::Nil);
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
        let constant = unsafe { (*(*self.compiler).function).chunk.add_constant(value) };
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

pub struct Compiler {
    locals: [Local; u8::MAX as usize + 1],
    local_count: usize,
    scope_depth: i8,

    function: *mut ObjFunction,
    ftype: FunctionType,

    enclosing: *mut Compiler,
}

#[derive(Clone)]
pub struct Local {
    name: Token,
    depth: i8,
}

impl Compiler {
    pub fn new(
        ftype: FunctionType,
        objects: *mut *mut Obj,
        strings: *mut HashTable,
        enclosing: *mut Compiler,
        chars: *const AsciiChar,
        length: usize,
    ) -> Self {
        let local = Local {
            name: Token::default(),
            depth: 0,
        };
        let mut locals = [0; u8::MAX as usize + 1].map(|_| local.clone());
        locals[0].depth = 0;
        locals[0].name.start = b"" as *const AsciiChar;
        locals[0].name.length = 0;
        let compiler = Self {
            locals,
            local_count: 1,
            scope_depth: 0,
            function: ObjFunction::new(objects),
            ftype: ftype.clone(),
            enclosing,
        };

        if ftype != FunctionType::Script {
            unsafe { (*compiler.function).name = ObjString::new(chars, length, objects, strings) }
        }

        compiler
    }
}

#[derive(PartialEq, Clone)]
pub enum FunctionType {
    Function,
    Script,
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::collections::hashtable::HashTable;
    use crate::types::chunk::Chunk;
    use crate::types::value::obj::free_object;

    macro_rules! parse_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (input, bytes) = $value;

                    let mut objects: *mut Obj = std::ptr::null_mut();
                    let mut strings = HashTable::new();

                    let source = CString::new(input).unwrap();
                    let scanner = &mut Scanner::new(source.as_bytes_with_nul().as_ptr());
                    let compiler = &mut Compiler::new(FunctionType::Script, &mut objects, &mut strings, std::ptr::null_mut(), std::ptr::null(), 0);
                    let parser = &mut Parser::new(scanner, compiler, &mut objects, &mut strings);

                    let function = parser.compile();

                    let mut expected = Chunk::default();
                    for byte in bytes {
                        expected.write(byte.into(), 1);
                    }
                    assert_eq!(unsafe { &(*function).chunk.code }, &expected.code);

                    let mut object = objects;
                    while !object.is_null() {
                        let next = unsafe { (*object).next.0 };
                        unsafe { free_object(object) };
                        object = next;
                    }

                    strings.free();
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
                OpCode::Nil.into(),
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
                OpCode::Nil.into(),
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
                OpCode::Nil.into(),
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
                OpCode::Nil.into(),
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
                OpCode::Nil.into(),
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
                OpCode::Nil.into(),
                OpCode::Return.into(),
            ]
        ),
        literal: (
            "true;",
            vec![
                OpCode::True,
                OpCode::Pop.into(),
                OpCode::Nil.into(),
                OpCode::Return.into(),
            ]
        ),
        not: (
            "!true;",
            vec![
                OpCode::True,
                OpCode::Not,
                OpCode::Pop.into(),
                OpCode::Nil.into(),
                OpCode::Return.into(),
            ]
        ),
        not_equal: (
            "2 != 3;",
            vec![
                OpCode::Constant.into(), 0,
                OpCode::Constant.into(), 1,
                OpCode::Equal.into(),
                OpCode::Not.into(),
                OpCode::Pop.into(),
                OpCode::Nil.into(),
                OpCode::Return.into(),
            ]
        ),
        equal: (
            "2 == 3;",
            vec![
                OpCode::Constant.into(), 0,
                OpCode::Constant.into(), 1,
                OpCode::Equal.into(),
                OpCode::Pop.into(),
                OpCode::Nil.into(),
                OpCode::Return.into(),
            ]
        ),
        greater: (
            "2 > 3;",
            vec![
                OpCode::Constant.into(), 0,
                OpCode::Constant.into(), 1,
                OpCode::Greater.into(),
                OpCode::Pop.into(),
                OpCode::Nil.into(),
                OpCode::Return.into(),
            ]
        ),
        greater_equal: (
            "2 >= 3;",
            vec![
                OpCode::Constant.into(), 0,
                OpCode::Constant.into(), 1,
                OpCode::Less.into(),
                OpCode::Not.into(),
                OpCode::Pop.into(),
                OpCode::Nil.into(),
                OpCode::Return.into(),
            ]
        ),
        less: (
            "2 < 3;",
            vec![
                OpCode::Constant.into(), 0,
                OpCode::Constant.into(), 1,
                OpCode::Less.into(),
                OpCode::Pop.into(),
                OpCode::Nil.into(),
                OpCode::Return.into(),
            ]
        ),
        less_equal: (
            "2 <= 3;",
            vec![
                OpCode::Constant.into(), 0,
                OpCode::Constant.into(), 1,
                OpCode::Greater.into(),
                OpCode::Not.into(),
                OpCode::Pop.into(),
                OpCode::Nil.into(),
                OpCode::Return.into(),
            ]
        ),
    }
}
