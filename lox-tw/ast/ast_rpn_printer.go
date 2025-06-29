package ast

import (
	"fmt"
	"strconv"
	"strings"
)

type AstRpnPrinter struct{}

func (p AstRpnPrinter) VisitGroupingExpr(expr GroupingExpr[string]) (string, error) {
	value, _ := expr.Expression.Accept(p)
	return fmt.Sprintf("(group %s)", value), nil
}

func (p AstRpnPrinter) VisitTernaryExpr(expr TernaryExpr[string]) (string, error) {
	condition, _ := expr.Condition.Accept(p)
	trueExpr, _ := expr.TrueExpr.Accept(p)
	falseExpr, _ := expr.FalseExpr.Accept(p)
	return fmt.Sprintf("(? %s %s %s)", condition, trueExpr, falseExpr), nil
}

func (p AstRpnPrinter) VisitBinaryExpr(expr BinaryExpr[string]) (string, error) {
	left, _ := expr.Left.Accept(p)
	right, _ := expr.Right.Accept(p)
	return fmt.Sprintf("(%s %s %s)", left, right, expr.Operator.Lexeme), nil
}

func (p AstRpnPrinter) VisitUnaryExpr(expr UnaryExpr[string]) (string, error) {
	right, _ := expr.Right.Accept(p)
	return fmt.Sprintf("(%s %s)", expr.Operator.Lexeme, right), nil
}

func (p AstRpnPrinter) VisitLiteralExpr(expr LiteralExpr[string]) (string, error) {
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

func (p AstRpnPrinter) VisitNothingExpr(expr NothingExpr[string]) (string, error) {
	return "(nothing)", nil
}
