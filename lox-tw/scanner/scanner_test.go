package scanner

import (
	"lox-tw/token"
	"testing"
)

func TestScanTokens(t *testing.T) {
	tests := []struct {
		name     string
		source   string
		expected []token.Token
	}{
		{
			name:   "Empty Source",
			source: "",
			expected: []token.Token{
				token.EofToken(1, 1),
			},
		},
		{
			name:   "Single Character",
			source: "+",
			expected: []token.Token{
				{Type: token.PLUS, Lexeme: "+", Literal: nil, Line: 1, Position: 1},
				token.EofToken(2, 1),
			},
		},
		{
			name:   "Multiple Characters",
			source: "var x = 10;",
			expected: []token.Token{
				{Type: token.VAR, Lexeme: "var", Literal: nil, Line: 1, Position: 3},
				{Type: token.IDENTIFIER, Lexeme: "x", Literal: nil, Line: 1, Position: 5},
				{Type: token.EQUAL, Lexeme: "=", Literal: nil, Line: 1, Position: 7},
				{Type: token.NUMBER, Lexeme: "10", Literal: 10.0, Line: 1, Position: 10},
				{Type: token.SEMICOLON, Lexeme: ";", Literal: nil, Line: 1, Position: 11},
				token.EofToken(12, 1),
			},
		},
		{
			name:   "String Literal",
			source: `"Hello, World!"`,
			expected: []token.Token{
				{Type: token.STRING, Lexeme: `"Hello, World!"`, Literal: "Hello, World!", Line: 1, Position: 15},
				token.EofToken(16, 1),
			},
		},
		{
			name:   "Comment",
			source: "// This is a comment\nvar y = 20;",
			expected: []token.Token{
				{Type: token.VAR, Lexeme: "var", Literal: nil, Line: 2, Position: 24},
				{Type: token.IDENTIFIER, Lexeme: "y", Literal: nil, Line: 2, Position: 26},
				{Type: token.EQUAL, Lexeme: "=", Literal: nil, Line: 2, Position: 28},
				{Type: token.NUMBER, Lexeme: "20", Literal: 20.0, Line: 2, Position: 31},
				{Type: token.SEMICOLON, Lexeme: ";", Literal: nil, Line: 2, Position: 32},
				token.EofToken(33, 2),
			},
		},
		{
			name:   "Multi-line Comment",
			source: "/* This is a\nmulti-line comment */\nvar z = 30;",
			expected: []token.Token{
				{Type: token.VAR, Lexeme: "var", Literal: nil, Line: 3, Position: 38},
				{Type: token.IDENTIFIER, Lexeme: "z", Literal: nil, Line: 3, Position: 40},
				{Type: token.EQUAL, Lexeme: "=", Literal: nil, Line: 3, Position: 42},
				{Type: token.NUMBER, Lexeme: "30", Literal: 30.0, Line: 3, Position: 45},
				{Type: token.SEMICOLON, Lexeme: ";", Literal: nil, Line: 3, Position: 46},
				token.EofToken(47, 3),
			},
		},
		{
			name:   "Complex Expression",
			source: "if (x > 10) { print \"x is greater than 10\"; } else { print \"x is not greater than 10\"; }",
			expected: []token.Token{
				{Type: token.IF, Lexeme: "if", Literal: nil, Line: 1, Position: 2},
				{Type: token.LEFT_PAREN, Lexeme: "(", Literal: nil, Line: 1, Position: 4},
				{Type: token.IDENTIFIER, Lexeme: "x", Literal: nil, Line: 1, Position: 5},
				{Type: token.GREATER, Lexeme: ">", Literal: nil, Line: 1, Position: 7},
				{Type: token.NUMBER, Lexeme: "10", Literal: 10.0, Line: 1, Position: 10},
				{Type: token.RIGHT_PAREN, Lexeme: ")", Literal: nil, Line: 1, Position: 11},
				{Type: token.LEFT_BRACE, Lexeme: "{", Literal: nil, Line: 1, Position: 13},
				{Type: token.PRINT, Lexeme: "print", Literal: nil, Line: 1, Position: 19},
				{Type: token.STRING, Lexeme: "\"x is greater than 10\"", Literal: "x is greater than 10", Line: 1, Position: 42},
				{Type: token.SEMICOLON, Lexeme: ";", Literal: nil, Line: 1, Position: 43},
				{Type: token.RIGHT_BRACE, Lexeme: "}", Literal: nil, Line: 1, Position: 45},
				{Type: token.ELSE, Lexeme: "else", Literal: nil, Line: 1, Position: 50},
				{Type: token.LEFT_BRACE, Lexeme: "{", Literal: nil, Line: 1, Position: 52},
				{Type: token.PRINT, Lexeme: "print", Literal: nil, Line: 1, Position: 58},
				{Type: token.STRING, Lexeme: "\"x is not greater than 10\"", Literal: "x is not greater than 10", Line: 1, Position: 85},
				{Type: token.SEMICOLON, Lexeme: ";", Literal: nil, Line: 1, Position: 86},
				{Type: token.RIGHT_BRACE, Lexeme: "}", Literal: nil, Line: 1, Position: 88},
				token.EofToken(89, 1),
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tokens, err := ScanTokens(tt.source)
			if err != nil {
				t.Fatalf("Error scanning tokens: %v", err)
			}
			if len(tokens) != len(tt.expected) {
				t.Fatalf("Expected %d tokens, got %d", len(tt.expected), len(tokens))
			}
			for i, token := range tokens {
				if token != tt.expected[i] {
					t.Errorf("Expected token %d to be %v, got %v", i, tt.expected[i], token)
				}
			}
		})
	}
}
