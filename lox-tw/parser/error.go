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
	return fmt.Sprintf("Parser error at '%d' (%s, %s): %s", e.Token.Position, e.Token.Type, e.Token.Lexeme, e.Message)
}
