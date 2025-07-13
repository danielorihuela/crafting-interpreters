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
