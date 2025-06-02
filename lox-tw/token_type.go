package main

type TokenType uint8

const (
	NOTHING TokenType = iota
)

// Single-character tokens
const (
	LEFT_PAREN TokenType = iota + NOTHING + 1
	RIGHT_PAREN
	LEFT_BRACE
	RIGHT_BRACE
	COMMA
	DOT
	MINUS
	PLUS
	SEMICOLON
	SLASH
	STAR
)

// Comparison operators
const (
	BANG TokenType = iota + STAR + 1
	BANG_EQUAL
	EQUAL
	EQUAL_EQUAL
	GREATER
	GREATER_EQUAL
	LESS
	LESS_EQUAL
)

// Literals
const (
	IDENTIFIER TokenType = iota + LESS_EQUAL + 1
	STRING
	NUMBER
)

// Keywords
const (
	AND TokenType = iota + NUMBER + 1
	CLASS
	ELSE
	FALSE
	FUN
	FOR
	IF
	NIL
	OR
	PRINT
	RETURN
	SUPER
	THIS
	TRUE
	VAR
	WHILE

	EOF
)

func (t TokenType) String() string {
	return [...]string{
		"NOTHING",
		"LEFT_PAREN",
		"RIGHT_PAREN",
		"LEFT_BRACE",
		"RIGHT_BRACE",
		"COMMA",
		"DOT",
		"MINUS",
		"PLUS",
		"SEMICOLON",
		"SLASH",
		"STAR",
		"BANG",
		"BANG_EQUAL",
		"EQUAL",
		"EQUAL_EQUAL",
		"GREATER",
		"GREATER_EQUAL",
		"LESS",
		"LESS_EQUAL",
		"IDENTIFIER",
		"STRING",
		"NUMBER",
		"AND",
		"CLASS",
		"ELSE",
		"FALSE",
		"FUN",
		"FOR",
		"IF",
		"NIL",
		"OR",
		"PRINT",
		"RETURN",
		"SUPER",
		"THIS",
		"TRUE",
		"VAR",
		"WHILE",
		"EOF",
	}[t]
}

// If the value does not exist, NOTHING will be returned.
func TrySingleCharTokenType(c byte) TokenType {
	return map[byte]TokenType{
		'(': LEFT_PAREN,
		')': RIGHT_PAREN,
		'{': LEFT_BRACE,
		'}': RIGHT_BRACE,
		',': COMMA,
		'.': DOT,
		'-': MINUS,
		'+': PLUS,
		';': SEMICOLON,
		'/': SLASH,
		'*': STAR,
	}[c]
}

// If the value does not exist, NOTHING will be returned.
func TryComparisonOperatorTokenType(a, b byte) (TokenType, uint) {
	comparisonOperatorToTokenType := map[string]TokenType{
		"!":  BANG,
		"!=": BANG_EQUAL,
		"=":  EQUAL,
		"==": EQUAL_EQUAL,
		">":  GREATER,
		">=": GREATER_EQUAL,
		"<":  LESS,
		"<=": LESS_EQUAL,
	}
	tokenType := comparisonOperatorToTokenType[string([]byte{a, b})]
	if tokenType != 0 {
		return tokenType, 2
	}

	return comparisonOperatorToTokenType[string(a)], 1
}

// If the value does not exist, NOTHING will be returned.
func TryKeywordTokenType(lexeme string) TokenType {
	return map[string]TokenType{
		"and":    AND,
		"class":  CLASS,
		"else":   ELSE,
		"false":  FALSE,
		"for":    FOR,
		"fun":    FUN,
		"if":     IF,
		"nil":    NIL,
		"or":     OR,
		"print":  PRINT,
		"return": RETURN,
		"super":  SUPER,
		"this":   THIS,
		"true":   TRUE,
		"var":    VAR,
		"while":  WHILE,
	}[lexeme]
}
