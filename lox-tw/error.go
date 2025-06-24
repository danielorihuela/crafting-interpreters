package main

import (
	"fmt"
)

type ScannerError struct {
	Line    uint
	Where   string
	Message string
}

func (e *ScannerError) Error() string {
	return fmt.Sprintf("[line %d] Error %s: %s", e.Line, e.Where, e.Message)
}

type ParserError struct {
	Token   Token
	Message string
}

func (e *ParserError) Error() string {
	return fmt.Sprintf("[line %d] Error at '%s': %s", e.Token.Line, e.Token.Lexeme, e.Message)
}
