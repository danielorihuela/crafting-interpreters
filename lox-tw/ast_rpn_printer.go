package main

import (
	"fmt"
	"strconv"
	"strings"
)

type AstRpnPrinter struct {
}

func (p AstRpnPrinter) VisitBinaryExpr(expr BinaryExpr[string]) string {
	return "(" + expr.Left.Accept(p) + " " + expr.Right.Accept(p) + " " + expr.Operator.Lexeme + ")"
}

func (p AstRpnPrinter) VisitGroupingExpr(expr GroupingExpr[string]) string {
	return "(group " + expr.Expression.Accept(p) + ")"
}

func (p AstRpnPrinter) VisitLiteralExpr(expr LiteralExpr[string]) string {
	switch v := expr.Value.(type) {
	case string:
		return "\"" + v + "\""
	case float64:
		value := strconv.FormatFloat(v, 'f', -1, 64)
		if !strings.Contains(value, ".") {
			value = value + ".0"
		}
		return value
	case bool:
		if v {
			return "true"
		} else {
			return "false"
		}
	case nil:
		return "nil"
	default:
		return fmt.Sprintf("%v", v)
	}
}

func (p AstRpnPrinter) VisitUnaryExpr(expr UnaryExpr[string]) string {
	return "(" + expr.Operator.Lexeme + " " + expr.Right.Accept(p) + ")"
}
