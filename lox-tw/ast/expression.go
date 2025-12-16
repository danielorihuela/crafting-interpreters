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
	VisitCallExpr(expr CallExpr[T]) (T, error)
	VisitGetExpr(expr GetExpr[T]) (T, error)
	VisitSetExpr(expr SetExpr[T]) (T, error)
	VisitThisExpr(expr ThisExpr[T]) (T, error)
	VisitLogicalExpr(expr LogicalExpr[T]) (T, error)
	VisitLiteralExpr(expr LiteralExpr[T]) (T, error)
	VisitNothingExpr(expr NothingExpr[T]) (T, error)
	VisitVarExpr(expr VarExpr[T]) (T, error)
	VisitAssignExpr(expr AssignExpr[T]) (T, error)
	VisitLambdaExpr(expr LambdaExpr[T]) (T, error)
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

type CallExpr[T any] struct {
	Callee      Expr[T]
	Parenthesis token.Token
	Arguments   []Expr[T]
}

func (e CallExpr[T]) Accept(visitor ExprVisitor[T]) (T, error) {
	return visitor.VisitCallExpr(e)
}

type GetExpr[T any] struct {
	Object Expr[T]
	Name   token.Token
}

func (e GetExpr[T]) Accept(visitor ExprVisitor[T]) (T, error) {
	return visitor.VisitGetExpr(e)
}

type SetExpr[T any] struct {
	Object Expr[T]
	Name   token.Token
	Value  Expr[T]
}

func (e SetExpr[T]) Accept(visitor ExprVisitor[T]) (T, error) {
	return visitor.VisitSetExpr(e)
}

type ThisExpr[T any] struct {
	Keyword token.Token
}

func (e ThisExpr[T]) Accept(visitor ExprVisitor[T]) (T, error) {
	return visitor.VisitThisExpr(e)
}

type LogicalExpr[T any] struct {
	Left     Expr[T]
	Operator token.Token
	Right    Expr[T]
}

func (e LogicalExpr[T]) Accept(visitor ExprVisitor[T]) (T, error) {
	return visitor.VisitLogicalExpr(e)
}

type LiteralExpr[T any] struct {
	Value any
}

func (e LiteralExpr[T]) Accept(visitor ExprVisitor[T]) (T, error) {
	return visitor.VisitLiteralExpr(e)
}

type NothingExpr[T any] struct {
}

func (e NothingExpr[T]) Accept(visitor ExprVisitor[T]) (T, error) {
	return visitor.VisitNothingExpr(e)
}

type VarExpr[T any] struct {
	Name token.Token
}

func (e VarExpr[T]) Accept(visitor ExprVisitor[T]) (T, error) {
	return visitor.VisitVarExpr(e)
}

type AssignExpr[T any] struct {
	Name  token.Token
	Value Expr[T]
}

func (e AssignExpr[T]) Accept(visitor ExprVisitor[T]) (T, error) {
	return visitor.VisitAssignExpr(e)
}

type LambdaExpr[T any] struct {
	Parameters []token.Token
	Body       []Stmt[T]
}

func (e LambdaExpr[T]) Accept(visitor ExprVisitor[T]) (T, error) {
	return visitor.VisitLambdaExpr(e)
}
