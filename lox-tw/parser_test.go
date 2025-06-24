package main

import "testing"

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
		tokens, _ := scanTokens(test.input)
		expr, _ := parseTokens(tokens)
		result := expr.Accept(AstPrinter{})
		if result != test.expected {
			t.Errorf("Expected '%s', got '%s'", test.expected, result)
		}
	}
}

func TestParserWithCommaOperator(t *testing.T) {
	if !COMMA_OPERATOR {
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
		tokens, _ := scanTokens(test.input)
		expr, _ := parseTokens(tokens)
		result := expr.Accept(AstPrinter{})
		if result != test.expected {
			t.Errorf("Expected '%s', got '%s'", test.expected, result)
		}
	}
}
