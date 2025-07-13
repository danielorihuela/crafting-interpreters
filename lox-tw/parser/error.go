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
	return fmt.Sprintf("[line %d] Error at '%s': %s", e.Token.Line, e.Token.Lexeme, e.Message)
}
