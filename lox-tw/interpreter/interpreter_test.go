package interpreter

import (
	"testing"

	"lox-tw/parser"
	"lox-tw/scanner"
)

func TestExprEvaluationFloats(t *testing.T) {
	tests := []struct {
		name     string
		expr     string
		expected float64
	}{
		{"Float Addition Grouping", "(1.5 + 2.5)", 4.0},
		{"Float Addition", "1.5 + 2.5", 4.0},
		{"Float Subtraction", "5.5 - 3.2", 2.3},
		{"Float Multiplication", "2.0 * 3.0", 6.0},
		{"Float Division", "6.0 / 2.0", 3.0},
		{"Unary Negation", "-6.0", -6.0},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tokens, _ := scanner.ScanTokens(tt.expr)
			expr, _ := parser.ParseTokens(tokens)
			result, err := expr.Accept(Interpreter{})
			if err != nil {
				t.Errorf("Error evaluating expression: %v", err)
			}
			result = result.(float64)
			if result != tt.expected {
				t.Errorf("Expected %f, got %f", tt.expected, result)
			}
		})
	}
}

func TestExprEvaluationBools(t *testing.T) {
	tests := []struct {
		name     string
		expr     string
		expected bool
	}{
		{"Float Greater", "5.5 > 3.2", true},
		{"Float Greater Equal", "5.5 >= 5.5", true},
		{"Float Less", "3.2 < 5.5", true},
		{"Float Less Equal", "3.2 <= 3.2", true},
		{"Float Bang Equal", "1.1 != 2.2", true},
		{"Float Equal Equal", "1.1 == 1.1", true},
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
				t.Errorf("Expected %t, got %t", tt.expected, result)
			}
		})
	}
}

func TestExprEvaluationStrings(t *testing.T) {
	tests := []struct {
		name     string
		expr     string
		expected string
	}{
		{"String Concatenation", "\"Hello\" + \" World\"", "Hello World"},
		{"String ternary true", "true ? \"yes\" : \"no\"", "yes"},
		{"String ternary false", "false ? \"yes\" : \"no\"", "no"},
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
