package interpreter

import (
	"fmt"

	"lox-tw/token"
)

type RuntimeError struct {
	Token   token.Token
	Message string
}

func (e *RuntimeError) Error() string {
	return fmt.Sprintf("Runtime error at '%s': %s", e.Token.Lexeme, e.Message)
}
