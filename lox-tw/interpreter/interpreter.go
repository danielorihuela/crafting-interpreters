package interpreter

import (
	"fmt"
	"strconv"
	"strings"

	"lox-tw/ast"
	"lox-tw/token"
)

type Interpreter struct {
	environment *Environment
}

func NewInterpreter() *Interpreter {
	return &Interpreter{
		environment: NewEnvironment(),
	}
}

func (i Interpreter) VisitAssignExpr(expr ast.AssignExpr[any]) (any, error) {
	value, err := expr.Value.Accept(i)
	if err != nil {
		return nil, err
	}

	i.environment.Assign(expr.Name, value)
	return value, nil
}

func (i Interpreter) VisitVarStmt(stmt ast.VarStmt[any]) error {
	var value any = nil
	if stmt.Initializer != nil {
		var err error
		value, err = stmt.Initializer.Accept(i)
		if err != nil {
			return err
		}
	}

	i.environment.Define(stmt.Name.Lexeme, value)
	return nil
}

func (i Interpreter) VisitVarExpr(expr ast.VarExpr[any]) (any, error) {
	return i.environment.Get(expr.Name.Lexeme)
}

func (i Interpreter) VisitExpressionStmt(stmt ast.ExpressionStmt[any]) error {
	_, err := stmt.Expression.Accept(i)
	return err
}

func (i Interpreter) VisitPrintStmt(stmt ast.PrintStmt[any]) error {
	value, err := stmt.Expression.Accept(i)
	if err != nil {
		return err
	}

	switch v := value.(type) {
	case nil:
		fmt.Println("nil")
	case float64:
		finalValue := strconv.FormatFloat(v, 'f', -1, 64)
		if !strings.Contains(finalValue, ".") {
			finalValue = finalValue + ".0"
		}
		fmt.Println(finalValue)
	default:
		fmt.Println(v)
	}

	return nil
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
			Message: "Operands must be either both numbers or both strings for addition",
		}
	case token.MINUS, token.SLASH, token.STAR,
		token.GREATER, token.GREATER_EQUAL, token.LESS, token.LESS_EQUAL,
		token.BANG_EQUAL, token.EQUAL_EQUAL:
		return computeOpFloats(leftValue, rightValue, expr.Operator)
	}

	return nil, nil
}

func computeOpFloats(leftValue, rightValue any, operator token.Token) (any, error) {
	left, ok := leftValue.(float64)
	if !ok {
		return leftValue, &RuntimeError{
			Token:   operator,
			Message: "Invalid left operand for " + operator.Lexeme,
		}
	}

	right, ok := rightValue.(float64)
	if !ok {
		return rightValue, &RuntimeError{
			Token:   operator,
			Message: "Invalid right operand for " + operator.Lexeme,
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
				Message: "Invalid number for unary minus",
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
