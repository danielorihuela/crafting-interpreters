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
	VisitExpressionStmt(stmt ExpressionStmt[T]) error
	VisitPrintStmt(stmt PrintStmt[T]) error
}

type ExpressionStmt[T any] struct {
	Expression Expr[T]
}

func (g ExpressionStmt[T]) Accept(visitor StmtVisitor[T]) error {
	return visitor.VisitExpressionStmt(g)
}

type PrintStmt[T any] struct {
	Expression Expr[T]
}

func (t PrintStmt[T]) Accept(visitor StmtVisitor[T]) error {
	return visitor.VisitPrintStmt(t)
}
