package interpreter

import (
	"lox-tw/ast"
	"lox-tw/token"
)

func (i Interpreter) VisitAssignExpr(expr ast.AssignExpr[any]) (any, error) {
	value, err := expr.Value.Accept(i)
	if err != nil {
		return nil, err
	}

	err = i.environment.Assign(expr.Name, value)
	if err != nil {
		return nil, err
	}

	return value, nil
}

func (i Interpreter) VisitGroupingExpr(expr ast.GroupingExpr[any]) (any, error) {
	return expr.Expression.Accept(i)
}

func (i Interpreter) VisitTernaryExpr(expr ast.TernaryExpr[any]) (any, error) {
	conditionValue, err := expr.Condition.Accept(i)
	if err != nil {
		return conditionValue, err
	}

	if conditionValue == nil {
		return expr.FalseExpr.Accept(i)
	}

	value, ok := conditionValue.(bool)
	if ok && !value {
		return expr.FalseExpr.Accept(i)
	}

	return expr.TrueExpr.Accept(i)
}

func (i Interpreter) VisitBinaryExpr(expr ast.BinaryExpr[any]) (any, error) {
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

		left, ok := leftValue.(string)
		right, ok2 := rightValue.(string)
		if ok && ok2 {
			return left + right, nil
		}

		return nil, &RuntimeError{
			Token:   expr.Operator,
			Message: "Operands must be two numbers or two strings.",
		}
	case token.MINUS, token.SLASH, token.STAR, token.GREATER, token.GREATER_EQUAL, token.LESS, token.LESS_EQUAL:
		return computeOpFloats(leftValue, rightValue, expr.Operator)
	case token.BANG_EQUAL:
		return leftValue != rightValue, nil
	case token.EQUAL_EQUAL:
		return leftValue == rightValue, nil
	}

	return nil, nil
}

func computeOpFloats(leftValue, rightValue any, operator token.Token) (any, error) {
	left, ok := leftValue.(float64)
	if !ok {
		return leftValue, &RuntimeError{
			Token:   operator,
			Message: "Operands must be numbers.",
		}
	}

	right, ok := rightValue.(float64)
	if !ok {
		return rightValue, &RuntimeError{
			Token:   operator,
			Message: "Operands must be numbers.",
		}
	}

	operations := map[token.TokenType]func(float64, float64) any{
		token.MINUS:         func(a, b float64) any { return a - b },
		token.PLUS:          func(a, b float64) any { return a + b },
		token.SLASH:         func(a, b float64) any { return a / b },
		token.STAR:          func(a, b float64) any { return a * b },
		token.GREATER:       func(a, b float64) any { return a > b },
		token.GREATER_EQUAL: func(a, b float64) any { return a >= b },
		token.LESS:          func(a, b float64) any { return a < b },
		token.LESS_EQUAL:    func(a, b float64) any { return a <= b },
		token.BANG_EQUAL:    func(a, b float64) any { return a != b },
		token.EQUAL_EQUAL:   func(a, b float64) any { return a == b },
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

func (i Interpreter) VisitUnaryExpr(expr ast.UnaryExpr[any]) (any, error) {
	rightValue, err := expr.Right.Accept(i)
	if err != nil {
		return rightValue, err
	}

	switch expr.Operator.Type {
	case token.MINUS:
		parsedValue, ok := rightValue.(float64)
		if !ok {
			return rightValue, &RuntimeError{
				Token:   expr.Operator,
				Message: "Operand must be a number.",
			}
		}
		return -parsedValue, nil
	case token.BANG:
		parsedValue, ok := rightValue.(bool)
		if !ok {
			return rightValue, &RuntimeError{
				Token:   expr.Operator,
				Message: "Invalid boolean for unary bang",
			}
		}
		return !parsedValue, nil
	}

	return nil, nil
}

func (i Interpreter) VisitLiteralExpr(expr ast.LiteralExpr[any]) (any, error) {
	return expr.Value, nil
}

func (i Interpreter) VisitNothingExpr(expr ast.NothingExpr[any]) (any, error) {
	return nil, nil
}
