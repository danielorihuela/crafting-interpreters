package ast

import (
	"fmt"
	"strconv"
	"strings"
)

type AnyPrinter struct{}

func (p AnyPrinter) VisitGroupingExpr(expr GroupingExpr[any]) (any, error) {
	value, _ := expr.Expression.Accept(p)
	return fmt.Sprintf("(group %s)", value), nil
}

func (p AnyPrinter) VisitTernaryExpr(expr TernaryExpr[any]) (any, error) {
	condition, _ := expr.Condition.Accept(p)
	trueExpr, _ := expr.TrueExpr.Accept(p)
	falseExpr, _ := expr.FalseExpr.Accept(p)
	return fmt.Sprintf("(? %s %s %s)", condition, trueExpr, falseExpr), nil
}

func (p AnyPrinter) VisitBinaryExpr(expr BinaryExpr[any]) (any, error) {
	left, _ := expr.Left.Accept(p)
	right, _ := expr.Right.Accept(p)
	return fmt.Sprintf("(%s %s %s)", expr.Operator.Lexeme, left, right), nil
}

func (p AnyPrinter) VisitUnaryExpr(expr UnaryExpr[any]) (any, error) {
	right, _ := expr.Right.Accept(p)
	return fmt.Sprintf("(%s %s)", expr.Operator.Lexeme, right), nil
}

func (p AnyPrinter) VisitLogicalExpr(expr LogicalExpr[any]) (any, error) {
	left, _ := expr.Left.Accept(p)
	right, _ := expr.Right.Accept(p)
	return fmt.Sprintf("(%s %s %s)", expr.Operator.Lexeme, left, right), nil
}

func (p AnyPrinter) VisitLiteralExpr(expr LiteralExpr[any]) (any, error) {
	switch v := expr.Value.(type) {
	case float64:
		value := strconv.FormatFloat(v, 'f', -1, 64)
		if !strings.Contains(value, ".") {
			value = value + ".0"
		}
		return value, nil
	case nil:
		return "nil", nil
	default:
		return fmt.Sprintf("%v", v), nil
	}
}

func (p AnyPrinter) VisitNothingExpr(expr NothingExpr[any]) (any, error) {
	return "(nothing)", nil
}

func (p AnyPrinter) VisitVarExpr(expr VarExpr[any]) (any, error) {
	return fmt.Sprintf("(var %s)", expr.Name.Lexeme), nil
}

func (p AnyPrinter) VisitAssignExpr(expr AssignExpr[any]) (any, error) {
	return fmt.Sprintf("(%s = %s)", expr.Name.Lexeme, expr.Value), nil
}

func (p AnyPrinter) VisitCallExpr(expr CallExpr[any]) (any, error) {
	callee, _ := expr.Callee.Accept(p)
	var arguments []string
	for _, arg := range expr.Arguments {
		argStr, _ := arg.Accept(p)
		arguments = append(arguments, fmt.Sprintf("%v", argStr))
	}
	return fmt.Sprintf("(call %s (%s))", callee, strings.Join(arguments, " ")), nil
}

func (p AnyPrinter) VisitLambdaExpr(expr LambdaExpr[any]) (any, error) {
	var parameters []string
	for _, param := range expr.Parameters {
		parameters = append(parameters, param.Lexeme)
	}
	var bodyStrings []string
	for _, stmt := range expr.Body {
		bodyStrings = append(bodyStrings, fmt.Sprintf("%v", stmt))
	}
	return fmt.Sprintf("(lambda (%s) {\n%s\n})", strings.Join(parameters, " "), strings.Join(bodyStrings, "\n")), nil
}

func (p AnyPrinter) VisitGetExpr(expr GetExpr[any]) (any, error) {
	object, _ := expr.Object.Accept(p)
	return fmt.Sprintf("(%s.%s)", object, expr.Name.Lexeme), nil
}

func (p AnyPrinter) VisitSetExpr(expr SetExpr[any]) (any, error) {
	object, _ := expr.Object.Accept(p)
	value, _ := expr.Value.Accept(p)
	return fmt.Sprintf("(%s.%s = %s)", object, expr.Name.Lexeme, value), nil
}

func (p AnyPrinter) VisitThisExpr(expr ThisExpr[any]) (any, error) {
	return "this", nil
}
