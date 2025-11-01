# crafting-interpreters

Tree-walker interpreter implementation of the Crafting Interpreters book.

## Develop

```
nix develop .#go
go run .
```

## Test suite

```
nix run .#test
```

## Supported Grammar

```
program               → declaration* EOF

# Declarations
declaration           → functionDeclaration | variableDeclaration | statement
functionDeclaration   → "fun" function
function              → IDENTIFIER "(" parameters? ")" block
parameters            → IDENTIFIER ( "," IDENTIFIER )*
variableDeclaration   → "var" IDENTIFIER ("=" expression )? ";"

# Statements
statement             → expressionStatement | ifStatement | whileStatement | forStatement | printStatement | blockStatement | breakStatement | returnStatement
expressionStatement   → expression ";"
ifStatement           → "if" "(" expression ")" statement ( "else" statement )?
whileStatement        → "while" "(" expression ")" statement
forStatement          → "for" "(" ( variableDeclaration | expressionStatement | ";" ) expression? ";" expression? ";" ")" statement
printStatement        → "print" expression ";"
blockStatement        → "{" declaration* "}"
breakStatement        → "break" ";"
returnStatement       → "return" expression? ";"

# Expressions
expression            → assignment
assignment            → IDENTIFIER "=" assignment | comma
comma                 → ternary ( "," ternary )*
ternary               → logic_or "?" expression ":" expression
logic_or              → logic_and ( "or" logic_and )*
logic_and             → equality ( "or" equality )*
equality              → comparison ( ( "!=" | "==" ) comparison )*
comparison            → term ( ( ">" | ">=" | "<" | "<=" ) term )*
term                  → factor ( ( "-" | "+" ) factor )*
factor                → unary ( ( "/" | "*" ) unary )*
unary                 → ( "!" | "-" ) unary | call
call                  → primary ( "(" arguments? ")" )*
arguments             → expression ( "," expression )*
primary               → NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" | IDENTIFIER | lambda
lambda                → "fun (" parameters? ")" block
```