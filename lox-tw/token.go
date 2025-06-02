package main

import (
	"fmt"
	"strconv"
	"strings"
)

type Token struct {
	Type    TokenType
	Lexeme  string
	Literal any
	Line    uint
}

func NilToken() Token {
	return Token{
		Type:    NOTHING,
		Lexeme:  "",
		Literal: nil,
		Line:    0,
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
