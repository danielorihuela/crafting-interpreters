package token

import (
	"fmt"
	"strconv"
	"strings"
)

type Token struct {
	Type    TokenType
	Lexeme  string
	Literal any

	Position uint
	Line     uint
}

func NilToken(position, line uint) Token {
	return Token{
		Type:     NOTHING,
		Lexeme:   "",
		Literal:  nil,
		Line:     line,
		Position: position,
	}
}

func EofToken(position, line uint) Token {
	return Token{
		Type:     EOF,
		Lexeme:   "",
		Literal:  nil,
		Line:     line,
		Position: position,
	}
}

func StringToken(lexeme string, position, line uint) Token {
	return Token{
		Type:     STRING,
		Lexeme:   lexeme,
		Literal:  lexeme[1 : len(lexeme)-1], // Remove quotes
		Line:     line,
		Position: position,
	}
}

func NumberToken(lexeme string, position, line uint) Token {
	value, _ := strconv.ParseFloat(lexeme, 64)
	return Token{
		Type:     NUMBER,
		Lexeme:   lexeme,
		Literal:  value,
		Line:     line,
		Position: position,
	}
}

func LeftParenToken(line, pos uint) Token {
	return Token{
		Type:     LEFT_PAREN,
		Lexeme:   "(",
		Literal:  nil,
		Line:     line,
		Position: pos,
	}
}

func RightParenToken(line, pos uint) Token {
	return Token{
		Type:     RIGHT_PAREN,
		Lexeme:   ")",
		Literal:  nil,
		Line:     line,
		Position: pos,
	}
}

func (t Token) String() string {
	return fmt.Sprintf("%s %s %s", t.Type.String(), t.Lexeme, t.literalToString())
}

func (t Token) literalToString() string {
	if t.Literal == nil {
		return "null"
	}
	if str, ok := t.Literal.(string); ok {
		return str
	}
	if num, ok := t.Literal.(float64); ok {
		literal := strconv.FormatFloat(num, 'f', -1, 64)
		if !strings.Contains(literal, ".") {
			literal = literal + ".0"
		}
		return literal
	}
	return fmt.Sprintf("%v", t.Literal)
}
