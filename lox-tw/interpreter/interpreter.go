package interpreter

import (
	"fmt"
	"strconv"

	"lox-tw/ast"
	"lox-tw/token"
)

type Interpreter struct{}

func (i Interpreter) VisitExpressionStmt(stmt ast.ExpressionStmt[string]) error {
	_, err := stmt.Expression.Accept(i)
	return err
}

func (i Interpreter) VisitPrintStmt(stmt ast.PrintStmt[string]) error {
	value, err := stmt.Expression.Accept(i)
	if err != nil {
		return err
	}

	fmt.Println(value)

	return nil
}

func (i Interpreter) VisitGroupingExpr(expr ast.GroupingExpr[string]) (string, error) {
	return expr.Expression.Accept(i)
}

func (i Interpreter) VisitTernaryExpr(expr ast.TernaryExpr[string]) (string, error) {
	conditionValue, err := expr.Condition.Accept(i)
	if err != nil {
		return conditionValue, err
	}

	if conditionValue == "true" {
		return expr.TrueExpr.Accept(i)
	} else {
		return expr.FalseExpr.Accept(i)
	}
}

func (i Interpreter) VisitBinaryExpr(expr ast.BinaryExpr[string]) (string, error) {
	leftValue, err := expr.Left.Accept(i)
	if err != nil {
		return leftValue, err
	}

	rightValue, err := expr.Right.Accept(i)
	if err != nil {
		return rightValue, err
	}

	switch expr.Operator.Type {
	case token.COMMA:
		return rightValue, nil
	case token.PLUS:
		result, err := computeOpFloats(leftValue, rightValue, expr.Operator)
		if err == nil {
			return result, nil
		}

		if leftValue[0] == '"' && leftValue[len(leftValue)-1] == '"' && rightValue[0] == '"' && rightValue[len(rightValue)-1] == '"' {
			return leftValue + rightValue, nil
		}

		return "", &RuntimeError{
			Token:   expr.Operator,
			Message: "Operands must be either both numbers or both strings for addition",
		}
	case token.MINUS, token.SLASH, token.STAR,
		token.GREATER, token.GREATER_EQUAL, token.LESS, token.LESS_EQUAL,
		token.BANG_EQUAL, token.EQUAL_EQUAL:
		return computeOpFloats(leftValue, rightValue, expr.Operator)
	}

	return "nil", nil
}

func computeOpFloats(leftValue, rightValue string, operator token.Token) (string, error) {
	left, err := strconv.ParseFloat(leftValue, 64)
	if err != nil {
		return leftValue, &RuntimeError{
			Token:   operator,
			Message: "Invalid left operand for " + operator.Lexeme,
		}
	}

	right, err := strconv.ParseFloat(rightValue, 64)
	if err != nil {
		return rightValue, &RuntimeError{
			Token:   operator,
			Message: "Invalid right operand for " + operator.Lexeme,
		}
	}

	operations := map[token.TokenType]func(float64, float64) string{
		token.MINUS:         func(a, b float64) string { return strconv.FormatFloat(a-b, 'f', -1, 64) },
		token.PLUS:          func(a, b float64) string { return strconv.FormatFloat(a+b, 'f', -1, 64) },
		token.SLASH:         func(a, b float64) string { return strconv.FormatFloat(a/b, 'f', -1, 64) },
		token.STAR:          func(a, b float64) string { return strconv.FormatFloat(a*b, 'f', -1, 64) },
		token.GREATER:       func(a, b float64) string { return strconv.FormatBool(a > b) },
		token.GREATER_EQUAL: func(a, b float64) string { return strconv.FormatBool(a >= b) },
		token.LESS:          func(a, b float64) string { return strconv.FormatBool(a < b) },
		token.LESS_EQUAL:    func(a, b float64) string { return strconv.FormatBool(a <= b) },
		token.BANG_EQUAL:    func(a, b float64) string { return strconv.FormatBool(a != b) },
		token.EQUAL_EQUAL:   func(a, b float64) string { return strconv.FormatBool(a == b) },
	}

	fn, exists := operations[operator.Type]
	if !exists {
		return operator.String(), &RuntimeError{
			Token:   operator,
			Message: "Operator is not supported",
		}
	}

	return fn(left, right), nil
}

func (i Interpreter) VisitUnaryExpr(expr ast.UnaryExpr[string]) (string, error) {
	rightValue, err := expr.Right.Accept(i)
	if err != nil {
		return rightValue, err
	}

	switch expr.Operator.Type {
	case token.MINUS:
		parsedValue, err := strconv.ParseFloat(rightValue, 64)
		if err != nil {
			return rightValue, &RuntimeError{
				Token:   expr.Operator,
				Message: "Invalid number for unary minus",
			}
		}
		return strconv.FormatFloat(-parsedValue, 'f', -1, 64), nil
	case token.BANG:
		parsedValue, err := strconv.ParseBool(rightValue)
		if err != nil {
			return rightValue, &RuntimeError{
				Token:   expr.Operator,
				Message: "Invalid boolean for unary bang",
			}
		}
		return strconv.FormatBool(!parsedValue), nil
	}

	return "nil", nil
}

func (i Interpreter) VisitLiteralExpr(expr ast.LiteralExpr[string]) (string, error) {
	return fmt.Sprintf("%v", expr.Value), nil
}

func (i Interpreter) VisitNothingExpr(expr ast.NothingExpr[string]) (string, error) {
	return "nil", nil
}
