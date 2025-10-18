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
		expected any
	}{
		// Float operations
		{"Float addition", "1.5 + 2.7", 4.2},
		{"Float subtraction", "5.5 - 3.2", 2.3},
		{"Float multiplication", "2.0 * 3.2", 6.4},
		{"Float division", "6.0 / 1.9", 3.1578947368421053},

		{"Float negation", "-6.0", -6.0},

		{"Float greater", "5.5 > 3.2", true},
		{"Float greater", "5.5 < 3.2", false},
		{"Float greater equal", "5.5 >= 4.5", true},
		{"Float greater equal", "5.5 <= 4.5", false},

		{"Float less", "3.2 < 5.5", true},
		{"Float less", "3.2 > 5.5", false},
		{"Float less equal", "2.2 <= 3.2", true},
		{"Float less equal", "2.2 >= 3.2", false},

		{"Float bang equal", "1.1 != 2.2", true},
		{"Float bang equal", "1.1 == 2.2", false},
		{"Float equal equal", "1.1 == 1.1", true},
		{"Float equal equal", "1.1 != 1.1", false},

		// String operations
		{"String concatenation", "\"Hello\" + \" World\"", "Hello World"},

		// Comma operations
		{"Comma with floats", "1.0, 2.0, 3.0", 3.0},
		{"Comma with mixed types", "1.0, 3.0, \"Hello\"", "Hello"},
		{"Comma with grouping", "(1.0, 2.0), 3.0", 3.0},

		// Ternary operations
		{"Ternary true", "true ? 1.0 : 2.0", 1.0},
		{"Ternary false", "false ? 1.0 : 2.0", 2.0},

		{"Ternary nested first operand true true", "true ? true ? 1.0 : 2.0 : 3.0", 1.0},
		{"Ternary nested first operand true false", "true ? false ? 1.0 : 2.0 : 3.0", 2.0},
		{"Ternary nested first operand false true", "false ? true ? 1.0 : 2.0 : 3.0", 3.0},

		{"Ternary nested second operand", "true ? 3.0 : false ? 1.0 : 2.0", 3.0},
		{"Ternary nested second operand with grouping", "(true ? 3.0 : false) ? 1.0 : 2.0", 1.0},

		{"Ternary with float", "1.0 > 0.0 ? 1.0 : 2.0", 1.0},
		{"Ternary with float false", "1.0 < 0.0 ? 1.0 : 2.0", 2.0},

		// Logical expressions
		{"Logical AND true true", "true and true", true},
		{"Logical AND true false", "true and false", false},
		{"Logical AND false true", "false and true", false},
		{"Logical AND false false", "false and false", false},

		{"Logical OR true true", "true or true", true},
		{"Logical OR true false", "true or false", true},
		{"Logical OR false true", "false or true", true},
		{"Logical OR false false", "false or false", false},

		{"Logical AND truthy truthy", "1.0 and true", true},
		{"Logical AND truthy falsy", "1.0 and false", false},
		{"Logical AND falsy truthy", "nil and 1.0", nil},
		{"Logical AND falsy falsy", "nil and nil", nil},

		{"Logical OR truthy truthy", "1.0 or true", 1.0},
		{"Logical OR truthy falsy", "1.0 or nil", 1.0},
		{"Logical OR falsy truthy", "nil or 1.0", 1.0},
		{"Logical OR falsy falsy", "nil or nil", nil},

		// Complex expressions
		{"Complex expression", "((1.0 + 2.0) * 3.0 > 5.0 ? -4.0 : 6.0) / 2.0", -2.0},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tokens, _ := scanner.ScanTokens(tt.expr)
			expr, _ := parser.ParseTokensToExpression(tokens)
			result, err := expr.Accept(Interpreter{})
			if err != nil {
				t.Errorf("Error evaluating expression: %v", err)
			}
			if result != tt.expected {
				t.Errorf("Expected %f, got %f", tt.expected, result)
			}
		})
	}
}
