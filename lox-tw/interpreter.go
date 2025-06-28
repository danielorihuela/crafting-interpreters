package main

import (
	"fmt"
	"strconv"
)

type Interpreter struct{}

func (i Interpreter) VisitGroupingExpr(expr GroupingExpr[string]) (string, error) {
	return expr.Expression.Accept(i)
}

func (i Interpreter) VisitTernaryExpr(expr TernaryExpr[string]) (string, error) {
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

func (i Interpreter) VisitBinaryExpr(expr BinaryExpr[string]) (string, error) {
	leftValue, err := expr.Left.Accept(i)
	if err != nil {
		return leftValue, err
	}

	rightValue, err := expr.Right.Accept(i)
	if err != nil {
		return rightValue, err
	}

	switch expr.Operator.Type {
	case COMMA:
		return rightValue, nil
	case MINUS:
		left, right, err := operandsAreNumbers(leftValue, rightValue, expr.Operator)
		if err != nil {
			return expr.Operator.String(), err
		}
		return strconv.FormatFloat(left-right, 'f', -1, 64), nil
	case PLUS:
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
	case SLASH:
		left, right, err := operandsAreNumbers(leftValue, rightValue, expr.Operator)
		if err != nil {
			return expr.Operator.String(), err
		}
		return strconv.FormatFloat(left/right, 'f', -1, 64), nil
	case STAR:
		left, right, err := operandsAreNumbers(leftValue, rightValue, expr.Operator)
		if err != nil {
			return expr.Operator.String(), err
		}
		return strconv.FormatFloat(left*right, 'f', -1, 64), nil
	case GREATER:
		left, right, err := operandsAreNumbers(leftValue, rightValue, expr.Operator)
		if err != nil {
			return expr.Operator.String(), err
		}
		return strconv.FormatBool(left > right), nil
	case GREATER_EQUAL:
		left, right, err := operandsAreNumbers(leftValue, rightValue, expr.Operator)
		if err != nil {
			return expr.Operator.String(), err
		}
		return strconv.FormatBool(left >= right), nil
	case LESS:
		left, right, err := operandsAreNumbers(leftValue, rightValue, expr.Operator)
		if err != nil {
			return expr.Operator.String(), err
		}
		return strconv.FormatBool(left < right), nil
	case LESS_EQUAL:
		left, right, err := operandsAreNumbers(leftValue, rightValue, expr.Operator)
		if err != nil {
			return expr.Operator.String(), err
		}
		return strconv.FormatBool(left <= right), nil
	case BANG_EQUAL:
		left, right, err := operandsAreNumbers(leftValue, rightValue, expr.Operator)
		if err != nil {
			return expr.Operator.String(), err
		}
		return strconv.FormatBool(left != right), nil
	case EQUAL_EQUAL:
		left, right, err := operandsAreNumbers(leftValue, rightValue, expr.Operator)
		if err != nil {
			return expr.Operator.String(), err
		}
		return strconv.FormatBool(left == right), nil
	}

	return "nil", nil
}

func operandsAreNumbers(leftValue, rightValue string, operator Token) (float64, float64, error) {
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

func (i Interpreter) VisitUnaryExpr(expr UnaryExpr[string]) (string, error) {
	rightValue, err := expr.Right.Accept(i)
	if err != nil {
		return rightValue, err
	}

	switch expr.Operator.Type {
	case MINUS:
		parsedValue, err := strconv.ParseFloat(rightValue, 64)
		if err != nil {
			return rightValue, &RuntimeError{
				Token:   expr.Operator,
				Message: "Invalid number for unary minus",
			}
		}
		return strconv.FormatFloat(-parsedValue, 'f', -1, 64), nil
	case BANG:
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

func (i Interpreter) VisitLiteralExpr(expr LiteralExpr[string]) (string, error) {
	return fmt.Sprintf("%v", expr.Value), nil
}

func (i Interpreter) VisitNothingExpr(expr NothingExpr[string]) (string, error) {
	return "nil", nil
}
