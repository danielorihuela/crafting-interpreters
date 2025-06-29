package ast

import (
	"fmt"
	"strconv"
	"strings"
)

type Printer struct{}

func (p Printer) VisitGroupingExpr(expr GroupingExpr[string]) (string, error) {
	value, _ := expr.Expression.Accept(p)
	return fmt.Sprintf("(group %s)", value), nil
}

func (p Printer) VisitTernaryExpr(expr TernaryExpr[string]) (string, error) {
	condition, _ := expr.Condition.Accept(p)
	trueExpr, _ := expr.TrueExpr.Accept(p)
	falseExpr, _ := expr.FalseExpr.Accept(p)
	return fmt.Sprintf("(? %s %s %s)", condition, trueExpr, falseExpr), nil
}

func (p Printer) VisitBinaryExpr(expr BinaryExpr[string]) (string, error) {
	left, _ := expr.Left.Accept(p)
	right, _ := expr.Right.Accept(p)
	return fmt.Sprintf("(%s %s %s)", expr.Operator.Lexeme, left, right), nil
}

func (p Printer) VisitUnaryExpr(expr UnaryExpr[string]) (string, error) {
	right, _ := expr.Right.Accept(p)
	return fmt.Sprintf("(%s %s)", expr.Operator.Lexeme, right), nil
}

func (p Printer) VisitLiteralExpr(expr LiteralExpr[string]) (string, error) {
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

func (p Printer) VisitNothingExpr(expr NothingExpr[string]) (string, error) {
	return "(nothing)", nil
}
