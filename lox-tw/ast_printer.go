package main

import (
	"fmt"
	"strconv"
	"strings"
)

type AstPrinter struct {
}

func (p AstPrinter) VisitBinaryExpr(expr BinaryExpr[string]) string {
	return "(" + expr.Operator.Lexeme + " " + expr.Left.Accept(p) + " " + expr.Right.Accept(p) + ")"
}

func (p AstPrinter) VisitGroupingExpr(expr GroupingExpr[string]) string {
	return "(group " + expr.Expression.Accept(p) + ")"
}

func (p AstPrinter) VisitLiteralExpr(expr LiteralExpr[string]) string {
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

func (p AstPrinter) VisitUnaryExpr(expr UnaryExpr[string]) string {
	return "(" + expr.Operator.Lexeme + " " + expr.Right.Accept(p) + ")"
}

func (p AstPrinter) VisitTernaryExpr(expr TernaryExpr[string]) string {
	return "(? " + expr.Condition.Accept(p) + " " + expr.TrueExpr.Accept(p) + " " + expr.FalseExpr.Accept(p) + ")"
}

func (p AstPrinter) VisitNothingExpr(expr NothingExpr[string]) string {
	return "(nothing)"
}
