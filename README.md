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
declaration           → variableDeclaration | statement
variableDeclaration   → "var" IDENTIFIER ("=" expression )? ";"

# Statements
statement             → expressionStatement | ifStatement | whileStatement | forStatement | printStatement | blockStatement | breakStatement
expressionStatement   → expression ";"
ifStatement           → "if" "(" expression ")" statement ( "else" statement )?
whileStatement        → "while" "(" expression ")" statement
forStatement          → "for" "(" ( variableDeclaration | expressionStatement | ";" ) expression? ";" expression? ";" ")" statement
printStatement        → "print" expression ";"
blockStatement        → "{" declaration* "}"
breakStatement        → "break" ";"

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
unary                 → ( "!" | "-" ) unary | primary
primary               → NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" | IDENTIFIER
```