use std::{cell::RefCell, mem::transmute, rc::Rc};

use crate::{
    memory::heap::ObjId,
    scanner::Scanner,
    types::{
        TokenType,
        opcode::OpCode,
        token::Token,
        value::{
            Value,
            function::ObjFunction,
            obj::{HeapObj, Obj},
        },
    },
    vm::VM,
};

pub struct Parser<'src> {
    scanner: &'src mut Scanner<'src>,
    compiler: Option<Rc<RefCell<Compiler>>>,

    current_class: Option<Rc<RefCell<ClassCompiler>>>,

    current: Token<'src>,
    previous: Token<'src>,

    vm: &'src mut VM,

    had_error: bool,
    panic_mode: bool,
}

impl<'src> Parser<'src> {
    pub fn new(
        scanner: &'src mut Scanner<'src>,
        compiler: Option<Rc<RefCell<Compiler>>>,
        vm: &'src mut VM,
    ) -> Self {
        Self {
            scanner,
            compiler,
            current_class: None,
            current: Token::default(),
            previous: Token::default(),
            vm,
            had_error: false,
            panic_mode: false,
        }
    }

    pub fn compile(&mut self) -> ObjId {
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
            ObjId::null()
        }
    }

    fn advance(&mut self) {
        self.previous = self.current.clone();

        loop {
            self.current = self.scanner.get_token();
            if self.current.ttype != TokenType::Error {
                break;
            }

            self.error_at_current(self.current.lexeme);
        }
    }

    fn declaration(&mut self) {
        if self.match_type(TokenType::Class) {
            self.class_declaration();
        } else if self.match_type(TokenType::Fun) {
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

    fn class_declaration(&mut self) {
        self.consume(TokenType::Identifier, "Expect class name.");
        let class_name = self.previous.clone();
        let name_constant = self.identifier_constant();
        self.declare_variable();

        self.emit_bytes(OpCode::Class, name_constant);
        self.define_variable(name_constant);

        let class_compiler = ClassCompiler {
            enclosing: self.current_class.clone(),
            has_superclass: false,
        };
        self.current_class = Some(Rc::new(RefCell::new(class_compiler)));

        if self.match_type(TokenType::Less) {
            self.consume(TokenType::Identifier, "Expect superclass name.");
            self.variable(false);

            if self.identifiers_equal(&class_name, &self.previous) {
                self.error("A class can't inherit from itself.");
            }

            self.begin_scope();
            self.add_local_for(self.synthetic_token("super"));
            self.define_variable(0);

            self.named_variable_with(&class_name, false);
            self.emit_byte(OpCode::Inherit);
            self.current_class
                .as_mut()
                .expect("class exists")
                .borrow_mut()
                .has_superclass = true;
        }

        self.named_variable_with(&class_name, false);
        self.consume(TokenType::LeftBrace, "Expect '{' before class body.");
        while self.current.ttype != TokenType::RightBrace && self.current.ttype != TokenType::Eof {
            self.method();
        }
        self.consume(TokenType::RightBrace, "Expect '}' after class body.");
        self.emit_byte(OpCode::Pop);

        if let Some(class_compiler_rc) = self.current_class.take() {
            let class_compiler = class_compiler_rc.borrow();

            if class_compiler.has_superclass {
                self.end_scope();
            }

            self.current_class = class_compiler.enclosing.clone();
        } else {
            self.current_class = None;
        }
    }

    fn synthetic_token(&self, text: &'static str) -> Token<'src> {
        Token {
            ttype: TokenType::Identifier,
            lexeme: text,
            line: self.previous.line,
        }
    }

    fn method(&mut self) {
        self.consume(TokenType::Identifier, "Expect method name.");
        let constant = self.identifier_constant();

        if self.previous.lexeme == "init" {
            self.function(FunctionType::Initializer);
        } else {
            self.function(FunctionType::Method);
        }

        self.emit_bytes(OpCode::Method, constant);
    }

    fn fun_declaration(&mut self) {
        let global = self.parse_variable("Expect function name.");
        self.mark_initialized();
        self.function(FunctionType::Function);
        self.define_variable(global);
    }

    fn function(&mut self, ftype: FunctionType) {
        let compiler = Rc::new(RefCell::new(Compiler::new(
            ftype,
            self.compiler.clone(),
            self.previous.lexeme,
            self.vm,
        )));
        self.compiler = Some(compiler.clone());

        self.begin_scope();

        self.consume(TokenType::LeftParen, "Expect '(' after function name.");

        if self.current.ttype != TokenType::RightParen {
            loop {
                let arity = {
                    let curr_compiler = compiler.borrow();
                    let HeapObj::Function(function) = &mut self.vm.heap[curr_compiler.function]
                    else {
                        panic!("Expected a function object");
                    };
                    function.arity += 1;

                    function.arity
                };

                if arity > 255 {
                    self.error_at_current("Can't have more than 255 parameters.");
                }

                let constant = self.parse_variable("Expect parameter name.");
                self.define_variable(constant);

                if !self.match_type(TokenType::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expect ')' after parameters.");
        self.consume(TokenType::LeftBrace, "Expect '{' before function body.");
        self.block();

        let function = self.end_compiler();
        let constant = self.make_constant(Value::Obj(Obj::Function(function)));
        self.emit_bytes(OpCode::Closure, constant);

        let upvalue_count = {
            let HeapObj::Function(function) = &self.vm.heap[function] else {
                panic!("Expected a function object");
            };
            function.upvalue_count
        };

        let curr_compiler = compiler.borrow_mut();
        for i in 0..upvalue_count {
            let upvalue = &curr_compiler.upvalues[i];
            self.emit_byte(if upvalue.is_local { 1 } else { 0 });
            self.emit_byte(upvalue.index);
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
            "Expect ';' after variable declaration.",
        );
        self.define_variable(global);
    }

    fn declare_variable(&mut self) {
        if self.compiler.as_ref().unwrap().borrow().scope_depth == 0 {
            return;
        }

        let mut error = false;
        {
            let local_curr = self.compiler.as_ref().unwrap().borrow();
            for i in (0..local_curr.local_count).rev() {
                let local = &local_curr.locals[i];
                if local.depth != -1 && local.depth < local_curr.scope_depth {
                    break;
                }

                if local.name == self.previous.lexeme {
                    error = true;
                }
            }
        }

        if error {
            self.error("Already a variable with this name in this scope.");
        }

        self.add_local();
    }

    fn identifiers_equal(&self, a: &Token, b: &Token) -> bool {
        a.lexeme == b.lexeme
    }

    fn add_local(&mut self) {
        let local_count = {
            let compiler = self.compiler.as_ref().unwrap().borrow();
            compiler.local_count
        };

        if local_count >= (u8::MAX as usize + 1) {
            self.error("Too many local variables in function.");
            return;
        }

        {
            let mut compiler = self.compiler.as_ref().unwrap().borrow_mut();
            let local = &mut compiler.locals[local_count];
            local.name = self.previous.lexeme.to_string();
            local.depth = -1;
            local.is_captured = false;
            compiler.local_count += 1;
        }
    }

    fn add_local_for(&mut self, name: Token<'src>) {
        if self.compiler.as_ref().unwrap().borrow().local_count == (u8::MAX as usize + 1) {
            self.error("Too many local variables in function.");
            return;
        }

        let local_count = self.compiler.as_ref().unwrap().borrow().local_count;
        let compiler = &mut self.compiler.as_ref().unwrap().borrow_mut();
        let local = &mut compiler.locals[local_count];
        local.name = name.lexeme.to_string();
        local.depth = -1;
        local.is_captured = false;
        compiler.local_count += 1;
    }

    fn parse_variable(&mut self, error_message: &str) -> u8 {
        self.consume(TokenType::Identifier, error_message);

        self.declare_variable();
        if self.compiler.as_ref().unwrap().borrow().scope_depth > 0 {
            return 0;
        }

        self.identifier_constant()
    }

    fn identifier_constant(&mut self) -> u8 {
        let id = self.vm.allocate_string(self.previous.lexeme);
        let value = Value::Obj(Obj::String(id));
        self.make_constant(value)
    }

    fn define_variable(&mut self, global: u8) {
        if self.compiler.as_ref().unwrap().borrow().scope_depth > 0 {
            self.mark_initialized();
            return;
        }

        self.emit_bytes(OpCode::DefineGlobal, global);
    }

    fn mark_initialized(&mut self) {
        if self.compiler.as_ref().unwrap().borrow().scope_depth == 0 {
            return;
        }

        let local_count = self.compiler.as_ref().unwrap().borrow().local_count;
        let scope_depth = self.compiler.as_ref().unwrap().borrow().scope_depth;
        let compiler = &mut self.compiler.as_ref().unwrap().borrow_mut();
        let local = &mut compiler.locals[local_count - 1];
        local.depth = scope_depth;
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
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        self.emit_byte(OpCode::Print);
    }

    fn if_statement(&mut self) {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

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
        if self.compiler.as_ref().unwrap().borrow().ftype == FunctionType::Script {
            self.error("Can't return from top-level code.");
        }

        if self.match_type(TokenType::Semicolon) {
            self.emit_return();
        } else {
            if self.compiler.as_ref().unwrap().borrow().ftype == FunctionType::Initializer {
                self.error("Can't return a value from an initializer.");
            }

            self.expression();
            self.consume(TokenType::Semicolon, "Expect ';' after return value.");
            self.emit_byte(OpCode::Return);
        }
    }

    fn while_statement(&mut self) {
        let loop_start = self.current_chunk_code_count();
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

        let exit_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::Pop);
        self.statement();
        self.emit_loop(loop_start);

        self.patch_jump(exit_jump);
        self.emit_byte(OpCode::Pop);
    }

    fn for_statement(&mut self) {
        self.begin_scope();
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.");
        if self.match_type(TokenType::Semicolon) {
            // No initializer.
        } else if self.match_type(TokenType::Var) {
            self.var_declaration();
        } else {
            self.expression_statement();
        }

        let mut loop_start = self.current_chunk_code_count();
        let mut exit_jump = -1;
        if !self.match_type(TokenType::Semicolon) {
            self.expression();
            self.consume(TokenType::Semicolon, "Expect ';' after loop condition.");

            exit_jump = self.emit_jump(OpCode::JumpIfFalse) as isize;
            self.emit_byte(OpCode::Pop);
        }

        if !self.match_type(TokenType::RightParen) {
            let body_jump = self.emit_jump(OpCode::Jump);
            let increment_start = self.current_chunk_code_count();
            self.expression();
            self.emit_byte(OpCode::Pop);
            self.consume(TokenType::RightParen, "Expect ')' after for clauses.");

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

        let offset = self.current_chunk_code_count() - loop_start + 2;
        if offset > u16::MAX as usize {
            self.error("Loop body too large.");
        }

        self.emit_byte(((offset >> 8) & 0xff) as u8);
        self.emit_byte((offset & 0xff) as u8);
    }

    fn emit_jump(&mut self, instruction: OpCode) -> usize {
        self.emit_byte(instruction);
        self.emit_byte(0xff);
        self.emit_byte(0xff);

        self.current_chunk_code_count() - 2
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.current_chunk_code_count() - offset - 2;
        if jump > u16::MAX as usize {
            self.error("Too much code to jump over.");
        }

        let HeapObj::Function(function) =
            &mut self.vm.heap[self.compiler.as_ref().unwrap().borrow().function]
        else {
            panic!("Expected a function object");
        };
        function.chunk.code[offset] = ((jump >> 8) & 0xff) as u8;
        function.chunk.code[offset + 1] = (jump & 0xff) as u8;
    }

    fn begin_scope(&mut self) {
        self.compiler.as_ref().unwrap().borrow_mut().scope_depth += 1;
    }

    fn block(&mut self) {
        while self.current.ttype != TokenType::RightBrace && self.current.ttype != TokenType::Eof {
            self.declaration();
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.");
    }

    fn end_scope(&mut self) {
        self.compiler.as_ref().unwrap().borrow_mut().scope_depth -= 1;

        while self.compiler.as_ref().unwrap().borrow().local_count > 0
            && self.compiler.as_ref().unwrap().borrow().locals
                [self.compiler.as_ref().unwrap().borrow().local_count - 1]
                .depth
                > self.compiler.as_ref().unwrap().borrow().scope_depth
        {
            if self.compiler.as_ref().unwrap().borrow().locals
                [self.compiler.as_ref().unwrap().borrow().local_count - 1]
                .is_captured
            {
                self.emit_byte(OpCode::CloseUpvalue);
            } else {
                self.emit_byte(OpCode::Pop);
            }
            self.compiler.as_ref().unwrap().borrow_mut().local_count -= 1;
        }
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after expression.");
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
            self.error("Expect expression.");
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
            self.error("Invalid assignment target.");
        }
    }

    fn consume(&mut self, ttype: TokenType, message: &str) {
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

    fn end_compiler(&mut self) -> ObjId {
        self.emit_return();

        #[cfg(debug_assertions)]
        {
            if !self.had_error {
                let function_id = self.compiler.as_ref().unwrap().borrow().function;
                let HeapObj::Function(function) = &self.vm.heap[function_id] else {
                    panic!("Expected a function object");
                };

                let name = {
                    if function.name.is_null() {
                        "script".to_string()
                    } else {
                        let HeapObj::String(name) = &self.vm.heap[function.name] else {
                            panic!("Expected a string object");
                        };
                        name.to_string()
                    }
                };

                function.chunk.disassemble(&name, self.vm);
            }
        }

        let function = self.compiler.as_ref().unwrap().borrow().function;
        let enclosing = self.compiler.as_ref().unwrap().borrow().enclosing.clone();
        if self.compiler.as_ref().unwrap().borrow().enclosing.is_none() {
            return function;
        }

        self.compiler = enclosing;

        function
    }

    fn number(&mut self, _can_assign: bool) {
        let value = self.previous.lexeme;
        let value: f64 = value.parse().unwrap();
        self.emit_constant(Value::Number(value));
    }

    fn grouping(&mut self, _can_assign: bool) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn unary(&mut self, _can_assign: bool) {
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

    fn literal(&mut self, _can_assign: bool) {
        match self.previous.ttype.clone() {
            TokenType::False => self.emit_byte(OpCode::False),
            TokenType::True => self.emit_byte(OpCode::True),
            TokenType::Nil => self.emit_byte(OpCode::Nil),
            _ => unreachable!(),
        }
    }

    fn dot(&mut self, can_assign: bool) {
        self.consume(TokenType::Identifier, "Expect property name after '.'.");
        let name = self.identifier_constant();

        if can_assign && self.match_type(TokenType::Equal) {
            self.expression();
            self.emit_bytes(OpCode::SetProperty, name);
        } else if self.match_type(TokenType::LeftParen) {
            let arg_count = self.argument_list();
            self.emit_bytes(OpCode::Invoke, name);
            self.emit_byte(arg_count);
        } else {
            self.emit_bytes(OpCode::GetProperty, name);
        }
    }

    fn string(&mut self, _can_assign: bool) {
        let id = self
            .vm
            .allocate_string(&self.previous.lexeme[1..self.previous.lexeme.len() - 1]);
        let obj = Value::Obj(Obj::String(id));
        self.emit_constant(obj);
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(can_assign);
    }

    fn this(&mut self, _can_assign: bool) {
        if self.current_class.is_none() {
            self.error("Can't use 'this' outside of a class.");
            return;
        }
        self.variable(false);
    }

    fn super_(&mut self, _can_assign: bool) {
        if self.current_class.is_none() {
            self.error("Can't use 'super' outside of a class.");
        } else if !self.current_class.as_ref().unwrap().borrow().has_superclass {
            self.error("Can't use 'super' in a class with no superclass.");
        }

        self.consume(TokenType::Dot, "Expect '.' after 'super'.");
        self.consume(TokenType::Identifier, "Expect superclass method name.");
        let name = self.identifier_constant();

        self.named_variable_with(&self.synthetic_token("this"), false);
        if self.match_type(TokenType::LeftParen) {
            let arg_count = self.argument_list();
            self.named_variable_with(&self.synthetic_token("super"), false);
            self.emit_bytes(OpCode::SuperInvoke, name);
            self.emit_byte(arg_count);
        } else {
            self.named_variable_with(&self.synthetic_token("super"), false);
            self.emit_bytes(OpCode::GetSuper, name);
        }
    }

    fn named_variable_with(&mut self, name: &Token<'src>, can_assign: bool) {
        let set_opcode;
        let get_opcode;
        let mut arg = resolve_local(self, self.compiler.clone(), name);
        if arg != -1 {
            set_opcode = OpCode::SetLocal;
            get_opcode = OpCode::GetLocal;
        } else {
            arg = self.resolve_upvalue(self.compiler.clone(), name);
            if arg != -1 {
                set_opcode = OpCode::SetUpvalue;
                get_opcode = OpCode::GetUpvalue;
            } else {
                let id = self.vm.allocate_string(name.lexeme);
                arg = self.make_constant(Value::Obj(Obj::String(id))) as isize;
                set_opcode = OpCode::SetGlobal;
                get_opcode = OpCode::GetGlobal;
            }
        }

        if can_assign && self.match_type(TokenType::Equal) {
            self.expression();
            self.emit_bytes(set_opcode, arg as u8);
        } else {
            self.emit_bytes(get_opcode, arg as u8);
        }
    }

    fn named_variable(&mut self, can_assign: bool) {
        let set_opcode;
        let get_opcode;
        let name = self.previous.clone();
        let mut arg = resolve_local(self, self.compiler.clone(), &name);
        if arg != -1 {
            set_opcode = OpCode::SetLocal;
            get_opcode = OpCode::GetLocal;
        } else {
            arg = self.resolve_upvalue(self.compiler.clone(), &name);
            if arg != -1 {
                set_opcode = OpCode::SetUpvalue;
                get_opcode = OpCode::GetUpvalue;
            } else {
                arg = self.identifier_constant() as isize;
                set_opcode = OpCode::SetGlobal;
                get_opcode = OpCode::GetGlobal;
            }
        }

        if can_assign && self.match_type(TokenType::Equal) {
            self.expression();
            self.emit_bytes(set_opcode, arg as u8);
        } else {
            self.emit_bytes(get_opcode, arg as u8);
        }
    }

    fn resolve_upvalue(
        &mut self,
        compiler: Option<Rc<RefCell<Compiler>>>,
        name: &Token<'src>,
    ) -> isize {
        if compiler.as_ref().unwrap().borrow().enclosing.is_none() {
            return -1;
        }

        let enclosing = compiler.as_ref().unwrap().borrow().enclosing.clone();
        let local = resolve_local(self, enclosing.clone(), name);
        if local != -1 {
            enclosing.as_ref().unwrap().borrow_mut().locals[local as usize].is_captured = true;
            match add_upvalue(compiler, self.vm, local as u8, true) {
                Ok(index) => return index,
                Err(message) => {
                    self.error(&message);
                    return 0;
                }
            }
        }

        let upvalue = self.resolve_upvalue(enclosing, name);
        if upvalue != -1 {
            match add_upvalue(compiler, self.vm, upvalue as u8, false) {
                Ok(index) => return index,
                Err(message) => {
                    self.error(&message);
                    return 0;
                }
            }
        }

        -1
    }

    fn and(&mut self, _can_assign: bool) {
        let end_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::Pop);
        self.parse_precedence(Precedence::And);
        self.patch_jump(end_jump);
    }

    fn or(&mut self, _can_assign: bool) {
        let else_jump = self.emit_jump(OpCode::JumpIfFalse);
        let end_jump = self.emit_jump(OpCode::Jump);

        self.patch_jump(else_jump);
        self.emit_byte(OpCode::Pop);

        self.parse_precedence(Precedence::Or);
        self.patch_jump(end_jump);
    }

    fn call(&mut self, _can_assign: bool) {
        let arg_count = self.argument_list();
        self.emit_bytes(OpCode::Call, arg_count);
    }

    fn argument_list(&mut self) -> u8 {
        let mut arg_count = 0;
        if self.current.ttype != TokenType::RightParen {
            loop {
                self.expression();
                if arg_count == 255 {
                    self.error("Can't have more than 255 arguments.");
                }
                arg_count += 1;

                if !self.match_type(TokenType::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expect ')' after arguments.");

        arg_count
    }

    fn get_rule(&self, ttype: TokenType, _can_assign: bool) -> ParseRule {
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
            TokenType::Dot => ParseRule {
                prefix: None,
                infix: Some(|parser, can_assign| parser.dot(can_assign)),
                precedence: Precedence::Call,
            },
            TokenType::This => ParseRule {
                prefix: Some(|parser, can_assign| parser.this(can_assign)),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Super => ParseRule {
                prefix: Some(|parser, can_assign| parser.super_(can_assign)),
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
        let function = self.compiler.as_ref().unwrap().borrow().function;
        let function_ptr = {
            let HeapObj::Function(function) = &mut self.vm.heap[function] else {
                panic!("Expected a function object");
            };
            function as *mut ObjFunction
        };
        unsafe {
            (*function_ptr)
                .chunk
                .write(b.into(), self.previous.line as usize);
        }
    }

    fn emit_return(&mut self) {
        if self.compiler.as_ref().unwrap().borrow().ftype.clone() == FunctionType::Initializer {
            self.emit_bytes(OpCode::GetLocal, 0);
        } else {
            self.emit_byte(OpCode::Nil);
        }

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
        let function = self.compiler.as_ref().unwrap().borrow().function;
        let function_ptr = {
            let HeapObj::Function(function) = &mut self.vm.heap[function] else {
                panic!("Expected a function object");
            };
            function as *mut ObjFunction
        };
        let constant = unsafe { (*function_ptr).chunk.add_constant(value, self.vm) };
        if constant > u8::MAX as usize {
            self.error("Too many constants in one chunk.");
            return 0;
        }

        constant as u8
    }

    fn error_at_current(&mut self, message: &str) {
        let token = &self.current.clone();
        self.error_at(token, message);
    }

    fn error(&mut self, message: &str) {
        let token = &self.previous.clone();
        self.error_at(token, message);
    }

    fn error_at(&mut self, token: &Token<'src>, message: &str) {
        if self.panic_mode {
            return;
        }

        self.panic_mode = true;
        eprint!("[line {}] Error", token.line);

        if token.ttype == TokenType::Eof {
            eprint!(" at end");
        } else if token.ttype == TokenType::Error {
        } else {
            eprint!(" at '{}'", token.lexeme);
        }

        eprintln!(": {message}");
        self.had_error = true;
    }

    fn current_chunk_code_count(&self) -> usize {
        let HeapObj::Function(function) =
            &self.vm.heap[self.compiler.as_ref().unwrap().borrow().function]
        else {
            panic!("Expected a function object");
        };
        function.chunk.code.len()
    }
}

fn resolve_local<'src>(
    parser: &mut Parser<'src>,
    compiler: Option<Rc<RefCell<Compiler>>>,
    name: &Token<'src>,
) -> isize {
    for i in (0..compiler.as_ref().unwrap().borrow().local_count).rev() {
        let local = &compiler.as_ref().unwrap().borrow().locals[i];
        if local.name == name.lexeme {
            if local.depth == -1 {
                parser.error("Can't read local variable in its own initializer.");
            }
            return i as isize;
        }
    }

    -1
}

fn add_upvalue(
    compiler: Option<Rc<RefCell<Compiler>>>,
    vm: &mut VM,
    index: u8,
    is_local: bool,
) -> Result<isize, String> {
    let function_id = compiler.as_ref().unwrap().borrow().function;
    let upvalue_count = {
        let HeapObj::Function(function) = &vm.heap[function_id] else {
            panic!("Expected a function object");
        };
        function.upvalue_count
    };

    for i in 0..upvalue_count {
        let upvalue = &compiler.as_ref().unwrap().borrow().upvalues[i];
        if upvalue.index == index && upvalue.is_local == is_local {
            return Ok(i as isize);
        }
    }

    if upvalue_count == (u8::MAX as usize + 1) {
        return Err("Too many closure variables in function.".to_string());
    }

    compiler.as_ref().unwrap().borrow_mut().upvalues[upvalue_count].is_local = is_local;
    compiler.as_ref().unwrap().borrow_mut().upvalues[upvalue_count].index = index;

    let HeapObj::Function(function) = &mut vm.heap[function_id] else {
        panic!("Expected a function object");
    };
    let count = function.upvalue_count;
    function.upvalue_count += 1;

    Ok(count as isize)
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

    upvalues: [Upvalue; u8::MAX as usize + 1],

    pub function: ObjId,
    ftype: FunctionType,

    pub enclosing: Option<Rc<RefCell<Compiler>>>,
}

pub struct ClassCompiler {
    enclosing: Option<Rc<RefCell<ClassCompiler>>>,
    has_superclass: bool,
}

#[derive(Clone, Copy)]
struct Upvalue {
    index: u8,
    is_local: bool,
}

#[derive(Clone)]
pub struct Local {
    name: String,
    depth: i8,
    is_captured: bool,
}

impl<'src> Compiler {
    pub fn new(
        ftype: FunctionType,
        enclosing: Option<Rc<RefCell<Compiler>>>,
        data: &str,
        vm: &'src mut VM,
    ) -> Self {
        let mut local = Local {
            name: String::new(),
            depth: 0,
            is_captured: false,
        };

        if ftype != FunctionType::Function {
            local.name = "this".to_string();
        } else {
            local.name = String::new();
        }

        let mut locals = [0; u8::MAX as usize + 1].map(|_| local.clone());
        locals[0].depth = 0;
        let compiler = Self {
            locals,
            local_count: 1,
            scope_depth: 0,
            upvalues: [Upvalue {
                index: 0,
                is_local: false,
            }; u8::MAX as usize + 1],
            function: vm.allocate(ObjFunction::new()),
            ftype: ftype.clone(),
            enclosing,
        };

        if ftype != FunctionType::Script {
            let name_id = vm.allocate_string(data);
            let HeapObj::Function(function) = &mut vm.heap[compiler.function] else {
                panic!("Expected a function object");
            };
            function.name = name_id;
        }

        compiler
    }
}

#[derive(PartialEq, Clone)]
pub enum FunctionType {
    Function,
    Script,
    Method,
    Initializer,
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::types::chunk::Chunk;
    use crate::types::value::obj::HeapObj;

    macro_rules! parse_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (input, bytes) = $value;

                    let scanner = &mut Scanner::new(input);
                    let mut vm = VM::new();
                    let compiler = Rc::new(RefCell::new(Compiler::new(FunctionType::Script, None, "", &mut vm)));
                    let parser = &mut Parser::new(scanner, Some(compiler), &mut vm);

                    let function = parser.compile();

                    let mut expected = Chunk::default();
                    for byte in bytes {
                        expected.write(byte.into(), 1);
                    }
                    let HeapObj::Function(function_obj) = &vm.heap[function] else {
                        panic!("Expected a function object");
                    };
                    assert_eq!(&function_obj.chunk.code, &expected.code);
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
