package ast

import (
	"fmt"
)

type RpnPrinter struct {
	printer Printer
}

func (p RpnPrinter) VisitGroupingExpr(expr GroupingExpr[string]) (string, error) {
	return p.printer.VisitGroupingExpr(expr)
}

func (p RpnPrinter) VisitTernaryExpr(expr TernaryExpr[string]) (string, error) {
	return p.printer.VisitTernaryExpr(expr)
}

func (p RpnPrinter) VisitBinaryExpr(expr BinaryExpr[string]) (string, error) {
	left, _ := expr.Left.Accept(p)
	right, _ := expr.Right.Accept(p)
	return fmt.Sprintf("(%s %s %s)", left, right, expr.Operator.Lexeme), nil
}

func (p RpnPrinter) VisitUnaryExpr(expr UnaryExpr[string]) (string, error) {
	return p.printer.VisitUnaryExpr(expr)
}

func (p RpnPrinter) VisitLiteralExpr(expr LiteralExpr[string]) (string, error) {
	return p.printer.VisitLiteralExpr(expr)
}

func (p RpnPrinter) VisitNothingExpr(expr NothingExpr[string]) (string, error) {
	return p.printer.VisitNothingExpr(expr)
}
