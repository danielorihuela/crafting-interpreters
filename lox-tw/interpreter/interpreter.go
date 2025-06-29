package interpreter

import (
	"fmt"
	"strconv"

	"lox-tw/ast"
	"lox-tw/token"
)

type Interpreter struct{}

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
	case token.MINUS:
		left, right, err := operandsAreNumbers(leftValue, rightValue, expr.Operator)
		if err != nil {
			return expr.Operator.String(), err
		}
		return strconv.FormatFloat(left-right, 'f', -1, 64), nil
	case token.PLUS:
		left, leftErr := strconv.ParseFloat(leftValue, 64)
		right, rightErr := strconv.ParseFloat(rightValue, 64)
		if leftErr == nil && rightErr == nil {
			return strconv.FormatFloat(left+right, 'f', -1, 64), nil
		}

		if leftValue[0] == '"' && leftValue[len(leftValue)-1] == '"' && rightValue[0] == '"' && rightValue[len(rightValue)-1] == '"' {
			return leftValue + rightValue, nil
		}

		return "", &RuntimeError{
			Token:   expr.Operator,
			Message: "Operands must be either both numbers or both strings for addition",
		}
	case token.SLASH:
		left, right, err := operandsAreNumbers(leftValue, rightValue, expr.Operator)
		if err != nil {
			return expr.Operator.String(), err
		}
		return strconv.FormatFloat(left/right, 'f', -1, 64), nil
	case token.STAR:
		left, right, err := operandsAreNumbers(leftValue, rightValue, expr.Operator)
		if err != nil {
			return expr.Operator.String(), err
		}
		return strconv.FormatFloat(left*right, 'f', -1, 64), nil
	case token.GREATER:
		left, right, err := operandsAreNumbers(leftValue, rightValue, expr.Operator)
		if err != nil {
			return expr.Operator.String(), err
		}
		return strconv.FormatBool(left > right), nil
	case token.GREATER_EQUAL:
		left, right, err := operandsAreNumbers(leftValue, rightValue, expr.Operator)
		if err != nil {
			return expr.Operator.String(), err
		}
		return strconv.FormatBool(left >= right), nil
	case token.LESS:
		left, right, err := operandsAreNumbers(leftValue, rightValue, expr.Operator)
		if err != nil {
			return expr.Operator.String(), err
		}
		return strconv.FormatBool(left < right), nil
	case token.LESS_EQUAL:
		left, right, err := operandsAreNumbers(leftValue, rightValue, expr.Operator)
		if err != nil {
			return expr.Operator.String(), err
		}
		return strconv.FormatBool(left <= right), nil
	case token.BANG_EQUAL:
		left, right, err := operandsAreNumbers(leftValue, rightValue, expr.Operator)
		if err != nil {
			return expr.Operator.String(), err
		}
		return strconv.FormatBool(left != right), nil
	case token.EQUAL_EQUAL:
		left, right, err := operandsAreNumbers(leftValue, rightValue, expr.Operator)
		if err != nil {
			return expr.Operator.String(), err
		}
		return strconv.FormatBool(left == right), nil
	}

	return "nil", nil
}

func operandsAreNumbers(leftValue, rightValue string, operator token.Token) (float64, float64, error) {
	left, err := strconv.ParseFloat(leftValue, 64)
	if err != nil {
		return 0, 0, &RuntimeError{
			Token:   operator,
			Message: "Invalid left operand for " + operator.Lexeme,
		}
	}

	right, err := strconv.ParseFloat(rightValue, 64)
	if err != nil {
		return 0, 0, &RuntimeError{
			Token:   operator,
			Message: "Invalid right operand for " + operator.Lexeme,
		}
	}

	return left, right, nil
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
