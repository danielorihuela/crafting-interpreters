package interpreter

import (
	"fmt"
	"os"

	"lox-tw/ast"
	"lox-tw/token"
	"lox-tw/utils"
)

func (i Interpreter) VisitAssignExpr(expr ast.AssignExpr[any]) (any, error) {
	value, err := expr.Value.Accept(i)
	if err != nil {
		return nil, err
	}

	if depth, ok := i.exprToDepth[expr]; ok {
		if err = i.environment.AssignAt(depth, expr.Name, value); err != nil {
			return nil, err
		}
		return value, nil
	}

	if err = i.environment.AssignGlobal(expr.Name, value); err != nil {
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

	if value, ok := conditionValue.(bool); ok && !value {
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
		if result, err := computeOpFloats(leftValue, rightValue, expr.Operator); err == nil {
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
		return !utils.IsTruthy(rightValue), nil
	}

	return nil, nil
}

func (i Interpreter) VisitLogicalExpr(expr ast.LogicalExpr[any]) (any, error) {
	leftValue, err := expr.Left.Accept(i)
	if err != nil {
		return leftValue, err
	}

	leftBool := utils.IsTruthy(leftValue)
	if expr.Operator.Type == token.OR && leftBool {
		return leftValue, nil
	}

	if expr.Operator.Type == token.AND && !leftBool {
		return leftValue, nil
	}

	return expr.Right.Accept(i)
}

func (i Interpreter) VisitLiteralExpr(expr ast.LiteralExpr[any]) (any, error) {
	return expr.Value, nil
}

func (i Interpreter) VisitNothingExpr(expr ast.NothingExpr[any]) (any, error) {
	return nil, nil
}

func (i Interpreter) VisitCallExpr(expr ast.CallExpr[any]) (any, error) {
	callee, err := expr.Callee.Accept(i)
	if err != nil {
		return callee, err
	}
	function, ok := callee.(Callable)
	if !ok {
		return nil, &RuntimeError{
			Token:   expr.Parenthesis,
			Message: "Can only call functions and classes.",
		}
	}

	arguments := []any{}
	for _, arg := range expr.Arguments {
		argValue, err := arg.Accept(i)
		if err != nil {
			return argValue, err
		}
		arguments = append(arguments, argValue)
	}

	if len(arguments) != function.Arity() {
		return nil, &RuntimeError{
			Token:   expr.Parenthesis,
			Message: fmt.Sprintf("Expected %d arguments but got %d.", function.Arity(), len(arguments)),
		}
	}

	return function.Call(i, arguments)
}

func (i Interpreter) VisitLambdaExpr(expr ast.LambdaExpr[any]) (any, error) {
	return NewLambda(expr, i.environment), nil
}

func (i Interpreter) VisitVarExpr(expr ast.VarExpr[any]) (any, error) {
	if depth, ok := i.exprToDepth[expr]; ok {
		return i.environment.GetAt(depth, expr.Name)
	}

	return i.environment.GetGlobal(expr.Name)
}

func (i Interpreter) VisitGetExpr(expr ast.GetExpr[any]) (any, error) {
	object, err := expr.Object.Accept(i)
	if err != nil {
		return nil, err
	}

	if instance, ok := object.(*Instance); ok {
		return instance.Get(expr.Name)
	}

	if os.Getenv("METACLASSES_ENABLED") == "true" {
		if classInstance, ok := object.(*Class); ok && classInstance.instance != nil {
			return classInstance.instance.Get(expr.Name)
		}
	}

	return nil, &RuntimeError{
		Token:   expr.Name,
		Message: "Only instances have properties.",
	}
}

func (i Interpreter) VisitSetExpr(expr ast.SetExpr[any]) (any, error) {
	object, err := expr.Object.Accept(i)
	if err != nil {
		return nil, err
	}

	instance, ok := object.(*Instance)
	if !ok {
		return nil, &RuntimeError{
			Token:   expr.Name,
			Message: "Only instances have fields.",
		}
	}

	value, err := expr.Value.Accept(i)
	if err != nil {
		return nil, err
	}

	instance.Set(expr.Name, value)

	return value, nil
}

func (i Interpreter) VisitThisExpr(expr ast.ThisExpr[any]) (any, error) {
	if depth, ok := i.exprToDepth[expr]; ok {
		return i.environment.GetAt(depth, expr.Keyword)
	}

	return i.environment.GetGlobal(expr.Keyword)
}
