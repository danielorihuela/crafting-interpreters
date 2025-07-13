package parser

import (
	"testing"

	"lox-tw/ast"
	"lox-tw/scanner"
)

func TestParser(t *testing.T) {
	tests := []struct {
		input    string
		expected string
	}{
		// Basic expressions
		{"1 + 2", "(+ 1.0 2.0)"},
		{"(3 * 4) - 5", "(- (group (* 3.0 4.0)) 5.0)"},
		{"!true", "(! true)"},
		{"nil == nil", "(== nil nil)"},
		{"(5 - (3 - 1)) + -1", "(+ (group (- 5.0 (group (- 3.0 1.0)))) (- 1.0))"},

		// Comma operator
		{"1, 2, 3", "(, (, 1.0 2.0) 3.0)"},
		{"(4, 5) + 6", "(+ (group (, 4.0 5.0)) 6.0)"},
		{"7 == (8, 9)", "(== 7.0 (group (, 8.0 9.0)))"},

		// Ternary operator
		{"true ? 1 : 2", "(? true 1.0 2.0)"},
		{"(3 > 2) ? 4 : 5", "(? (group (> 3.0 2.0)) 4.0 5.0)"},
		{"(6 == 6) ? (7 + 8) : (9 - 10)", "(? (group (== 6.0 6.0)) (group (+ 7.0 8.0)) (group (- 9.0 10.0)))"},

		// Comma and ternary together
		{"1, 2 ? 3 : 4", "(, 1.0 (? 2.0 3.0 4.0))"},
		{"(5, 6) ? (7 + 8) : (9 - 10)", "(? (group (, 5.0 6.0)) (group (+ 7.0 8.0)) (group (- 9.0 10.0)))"},
	}

	for _, test := range tests {
		tokens, _ := scanner.ScanTokens(test.input)
		expr, _ := ParseTokens(tokens)
		result, _ := expr.Accept(ast.AnyPrinter{})
		if result != test.expected {
			t.Errorf("Expected '%s', got '%s'", test.expected, result)
		}
	}
}
