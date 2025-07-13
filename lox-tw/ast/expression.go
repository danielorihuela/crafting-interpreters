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

	VisitVarExpr(expr VarExpr[T]) (T, error)
}

type GroupingExpr[T any] struct {
	Expression Expr[T]
}

func (e GroupingExpr[T]) Accept(visitor ExprVisitor[T]) (T, error) {
	return visitor.VisitGroupingExpr(e)
}

type TernaryExpr[T any] struct {
	Condition Expr[T]
	TrueExpr  Expr[T]
	FalseExpr Expr[T]
}

func (e TernaryExpr[T]) Accept(visitor ExprVisitor[T]) (T, error) {
	return visitor.VisitTernaryExpr(e)
}

type BinaryExpr[T any] struct {
	Left     Expr[T]
	Operator token.Token
	Right    Expr[T]
}

func (e BinaryExpr[T]) Accept(visitor ExprVisitor[T]) (T, error) {
	return visitor.VisitBinaryExpr(e)
}

type UnaryExpr[T any] struct {
	Operator token.Token
	Right    Expr[T]
}

func (e UnaryExpr[T]) Accept(visitor ExprVisitor[T]) (T, error) {
	return visitor.VisitUnaryExpr(e)
}

type LiteralExpr[T any] struct {
	Value any
}

func (e LiteralExpr[T]) Accept(visitor ExprVisitor[T]) (T, error) {
	return visitor.VisitLiteralExpr(e)
}

type NothingExpr[T any] struct{}

func (e NothingExpr[T]) Accept(visitor ExprVisitor[T]) (T, error) {
	return visitor.VisitNothingExpr(e)
}

type VarExpr[T any] struct {
	Name token.Token
}

func (e VarExpr[T]) Accept(visitor ExprVisitor[T]) (T, error) {
	return visitor.VisitVarExpr(e)
}
