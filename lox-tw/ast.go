/*
expression     → literal | unary | binary | grouping ;
literal        → NUMBER | STRING | "true" | "false" | "nil" ;
grouping       → "(" expression ")" ;
unary          → ( "-" | "!" ) expression ;
binary         → expression operator expression ;
operator       → "==" | "!=" | "<" | "<=" | ">" | ">=" | "+"  | "-"  | "*" | "/" ;
*/
package main

type Expr[T any] interface {
	Accept(visitor ExprVisitor[T]) T
}

type ExprVisitor[T any] interface {
	VisitBinaryExpr(expr BinaryExpr[T]) T
	VisitGroupingExpr(expr GroupingExpr[T]) T
	VisitLiteralExpr(expr LiteralExpr[T]) T
	VisitUnaryExpr(expr UnaryExpr[T]) T
	VisitTernaryExpr(expr TernaryExpr[T]) T
	VisitNothingExpr(expr NothingExpr[T]) T
}

type BinaryExpr[T any] struct {
	Left     Expr[T]
	Operator Token
	Right    Expr[T]
}

func (b BinaryExpr[T]) Accept(visitor ExprVisitor[T]) T {
	return visitor.VisitBinaryExpr(b)
}

type GroupingExpr[T any] struct {
	Expression Expr[T]
}

func (g GroupingExpr[T]) Accept(visitor ExprVisitor[T]) T {
	return visitor.VisitGroupingExpr(g)
}

type LiteralExpr[T any] struct {
	Value any
}

func (l LiteralExpr[T]) Accept(visitor ExprVisitor[T]) T {
	return visitor.VisitLiteralExpr(l)
}

type UnaryExpr[T any] struct {
	Operator Token
	Right    Expr[T]
}

func (u UnaryExpr[T]) Accept(visitor ExprVisitor[T]) T {
	return visitor.VisitUnaryExpr(u)
}

type TernaryExpr[T any] struct {
	Condition Expr[T]
	TrueExpr  Expr[T]
	FalseExpr Expr[T]
}

func (t TernaryExpr[T]) Accept(visitor ExprVisitor[T]) T {
	return visitor.VisitTernaryExpr(t)
}

type NothingExpr[T any] struct{}

func (n NothingExpr[T]) Accept(visitor ExprVisitor[T]) T {
	return visitor.VisitNothingExpr(n)
}
