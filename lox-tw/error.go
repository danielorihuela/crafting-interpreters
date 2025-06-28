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
	return fmt.Sprintf("[line %d] Scanner error %s: %s", e.Line, e.Where, e.Message)
}

type ParserError struct {
	Token   Token
	Message string
}

func (e *ParserError) Error() string {
	return fmt.Sprintf("Parser error at '%s': %s", e.Token.Lexeme, e.Message)
}

type RuntimeError struct {
	Token   Token
	Message string
}

func (e *RuntimeError) Error() string {
	return fmt.Sprintf("Runtime error at '%s': %s", e.Token.Lexeme, e.Message)
}
