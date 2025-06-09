package main

import "fmt"

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
		return fmt.Sprintf("%g", v)
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
	return "(" + expr.Operator.Lexeme + expr.Right.Accept(p) + ")"
}
