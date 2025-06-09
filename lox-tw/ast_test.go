package main

import "testing"

func TestAstPrinter(t *testing.T) {
	expression := BinaryExpr[string]{
		Left: UnaryExpr[string]{
			Operator: Token{Type: MINUS, Lexeme: "-", Line: 1, Position: 1},
			Right:    LiteralExpr[string]{Value: 123},
		},
		Operator: Token{Type: STAR, Lexeme: "*", Line: 1, Position: 5},
		Right: GroupingExpr[string]{
			Expression: LiteralExpr[string]{Value: 45.67},
		},
	}

	expected := "(* (-123) (group 45.67))"

	result := expression.Accept(AstPrinter{})
	if result != expected {
		t.Errorf("Expected '%s', got '%s'", expected, result)
	}
}

func TestAstRpnPrinter(t *testing.T) {
	expression := BinaryExpr[string]{
		Left: UnaryExpr[string]{
			Operator: Token{Type: MINUS, Lexeme: "-", Line: 1, Position: 1},
			Right:    LiteralExpr[string]{Value: 123},
		},
		Operator: Token{Type: STAR, Lexeme: "*", Line: 1, Position: 5},
		Right: GroupingExpr[string]{
			Expression: LiteralExpr[string]{Value: 45.67},
		},
	}

	expected := "((-123) (group 45.67) *)"

	result := expression.Accept(AstRpnPrinter{})
	if result != expected {
		t.Errorf("Expected '%s', got '%s'", expected, result)
	}
}
