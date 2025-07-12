/*
program        → statement* EOF ;
statement      → exprStmt | printStmt ;
exprStmt       → expression ";" ;
printStmt      → "print" expression ";" ;
*/
package ast

type Stmt[T any] interface {
	Accept(visitor StmtVisitor[T]) error
}

type StmtVisitor[T any] interface {
	VisitExpression(stmt Expression[T]) error
	VisitPrint(stmt Print[T]) error
}

type Expression[T any] struct {
	Expression Expr[T]
}

func (g Expression[T]) Accept(visitor StmtVisitor[T]) error {
	return visitor.VisitExpression(g)
}

type Print[T any] struct {
	Expression Expr[T]
}

func (t Print[T]) Accept(visitor StmtVisitor[T]) error {
	return visitor.VisitPrint(t)
}
