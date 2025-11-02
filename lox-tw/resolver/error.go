package resolver

import (
	"fmt"
	"lox-tw/token"
)

type ResolverError struct {
	Token   token.Token
	Message string
}

func (e *ResolverError) Error() string {
	return fmt.Sprintf("[line %d] Error at '%s': %s", e.Token.Line, e.Token.Lexeme, e.Message)
}
