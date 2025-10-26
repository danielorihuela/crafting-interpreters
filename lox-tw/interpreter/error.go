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

type BreakError struct{}

func (e *BreakError) Error() string {
	return "Break statement encountered"
}

type ReturnError struct {
	Value any
}

func (e *ReturnError) Error() string {
	return "Return statement encountered"
}
