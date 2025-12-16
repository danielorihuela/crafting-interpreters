package parser

import (
	"fmt"
	"lox-tw/token"
)

type ParserError struct {
	Token   token.Token
	Message string
}

func (e *ParserError) Error() string {
	if e.Token.Type == token.EOF {
		return fmt.Sprintf("[line %d] Error at end: %s", e.Token.Line, e.Message)
	}
	return fmt.Sprintf("[line %d] Error at '%s': %s", e.Token.Line, e.Token.Lexeme, e.Message)
}
