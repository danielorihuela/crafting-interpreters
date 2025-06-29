/*
expression     → literal | unary | binary | grouping ;
literal        → NUMBER | STRING | "true" | "false" | "nil" ;
grouping       → "(" expression ")" ;
unary          → ( "-" | "!" ) expression ;
binary         → expression operator expression ;
operator       → "==" | "!=" | "<" | "<=" | ">" | ">=" | "+"  | "-"  | "*" | "/" ;
*/
package ast

import "lox-tw/token"

type Expr[T any] interface {
	Accept(visitor ExprVisitor[T]) (T, error)
}

type ExprVisitor[T any] interface {
	VisitGroupingExpr(expr GroupingExpr[T]) (T, error)
	VisitTernaryExpr(expr TernaryExpr[T]) (T, error)
	VisitBinaryExpr(expr BinaryExpr[T]) (T, error)
	VisitUnaryExpr(expr UnaryExpr[T]) (T, error)
	VisitLiteralExpr(expr LiteralExpr[T]) (T, error)
	VisitNothingExpr(expr NothingExpr[T]) (T, error)
}

type GroupingExpr[T any] struct {
	Expression Expr[T]
}

func (g GroupingExpr[T]) Accept(visitor ExprVisitor[T]) (T, error) {
	return visitor.VisitGroupingExpr(g)
}

type TernaryExpr[T any] struct {
	Condition Expr[T]
	TrueExpr  Expr[T]
	FalseExpr Expr[T]
}

func (t TernaryExpr[T]) Accept(visitor ExprVisitor[T]) (T, error) {
	return visitor.VisitTernaryExpr(t)
}

type BinaryExpr[T any] struct {
	Left     Expr[T]
	Operator token.Token
	Right    Expr[T]
}

func (b BinaryExpr[T]) Accept(visitor ExprVisitor[T]) (T, error) {
	return visitor.VisitBinaryExpr(b)
}

type UnaryExpr[T any] struct {
	Operator token.Token
	Right    Expr[T]
}

func (u UnaryExpr[T]) Accept(visitor ExprVisitor[T]) (T, error) {
	return visitor.VisitUnaryExpr(u)
}

type LiteralExpr[T any] struct {
	Value any
}

func (l LiteralExpr[T]) Accept(visitor ExprVisitor[T]) (T, error) {
	return visitor.VisitLiteralExpr(l)
}

type NothingExpr[T any] struct{}

func (n NothingExpr[T]) Accept(visitor ExprVisitor[T]) (T, error) {
	return visitor.VisitNothingExpr(n)
}
