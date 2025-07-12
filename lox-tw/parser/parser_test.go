package parser

import (
	"testing"

	"lox-tw/ast"
	"lox-tw/features"
	"lox-tw/scanner"
)

func TestParser(t *testing.T) {
	tests := []struct {
		input    string
		expected string
	}{
		{"1 + 2", "(+ 1.0 2.0)"},
		{"(3 * 4) - 5", "(- (group (* 3.0 4.0)) 5.0)"},
		{"!true", "(! true)"},
		{"nil == nil", "(== nil nil)"},
		{"(5 - (3 - 1)) + -1", "(+ (group (- 5.0 (group (- 3.0 1.0)))) (- 1.0))"},
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

func TestParserWithCommaOperator(t *testing.T) {
	if !features.COMMA_OPERATOR {
		t.Skip("COMMA_OPERATOR feature is not enabled")
	}

	tests := []struct {
		input    string
		expected string
	}{
		{"1, 2, 3", "(, (, 1.0 2.0) 3.0)"},
		{"(4, 5) + 6", "(+ (group (, 4.0 5.0)) 6.0)"},
		{"7 == (8, 9)", "(== 7.0 (group (, 8.0 9.0)))"},
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

func TestParseWithTernaryOperator(t *testing.T) {
	if !features.TERNARY_OPERATOR {
		t.Skip("TERNARY_OPERATOR feature is not enabled")
	}

	tests := []struct {
		input    string
		expected string
	}{
		{"true ? 1 : 2", "(? true 1.0 2.0)"},
		{"(3 > 2) ? 4 : 5", "(? (group (> 3.0 2.0)) 4.0 5.0)"},
		{"(6 == 6) ? (7 + 8) : (9 - 10)", "(? (group (== 6.0 6.0)) (group (+ 7.0 8.0)) (group (- 9.0 10.0)))"},
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

func TestParserWithCommaAndTernaryOperators(t *testing.T) {
	if !features.COMMA_OPERATOR || !features.TERNARY_OPERATOR {
		t.Skip("COMMA_OPERATOR and TERNARY_OPERATOR features are not both enabled")
	}

	tests := []struct {
		input    string
		expected string
	}{
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
