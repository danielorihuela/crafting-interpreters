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
statement             → expressionStatement | printStatement
expressionStatement   → expression ";"
printStatement        → "print" expression ";"

# Expressions
expression            → comma
assignment            → IDENTIFIER "=" assignment | comma
comma                 → ternary ( "," ternary )*
ternary               → equality "?" expression ":" expression
equality              → comparison ( ( "!=" | "==" ) comparison )*
comparison            → term ( ( ">" | ">=" | "<" | "<=" ) term )*
term                  → factor ( ( "-" | "+" ) factor )*
factor                → unary ( ( "/" | "*" ) unary )*
unary                 → ( "!" | "-" ) unary | primary
primary               → NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" | IDENTIFIER
```