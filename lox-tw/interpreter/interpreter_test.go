package interpreter

import (
	"testing"

	"lox-tw/parser"
	"lox-tw/scanner"
)

func TestExprEvaluation(t *testing.T) {
	tests := []struct {
		name     string
		expr     string
		expected string
	}{
		{"Grouping", "(1 + 2)", "3"},
		{"Ternary True", "true ? \"yes\" : \"no\"", "yes"},
		{"Ternary False", "false ? \"yes\" : \"no\"", "no"},
		{"Binary Plus", "1 + 2", "3"},
		{"Binary Minus", "5 - 3", "2"},
		{"Binary Slash", "6 / 2", "3"},
		{"Binary Star", "2 * 3", "6"},
		{"Binary Greater", "5 > 3", "true"},
		{"Binary Greater Equal", "5 >= 5", "true"},
		{"Binary Less", "3 < 5", "true"},
		{"Binary Less Equal", "3 <= 3", "true"},
		{"Binary Bang Equal", "1 != 2", "true"},
		{"Binary Equal Equal", "1 == 1", "true"},
		{"Binary Comma", "1, 2", "2"},
		{"Unary Negation", "-5", "-5"},
		{"Unary Not", "!true", "false"},
		{"Unary Minus", "-(3 + 2)", "-5"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tokens, _ := scanner.ScanTokens(tt.expr)
			expr, _ := parser.ParseTokens(tokens)
			result, err := expr.Accept(Interpreter{})
			if err != nil {
				t.Errorf("Error evaluating expression: %v", err)
			}
			if result != tt.expected {
				t.Errorf("Expected %s, got %s", tt.expected, result)
			}
		})
	}
}
