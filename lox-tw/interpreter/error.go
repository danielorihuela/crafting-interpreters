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
	return fmt.Sprintf("%s\n[line %d]", e.Message, e.Token.Line)
}
