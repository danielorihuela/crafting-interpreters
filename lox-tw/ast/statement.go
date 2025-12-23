package ast

import "lox-tw/token"

type Stmt[T any] interface {
	Accept(visitor StmtVisitor[T]) error
}

type StmtVisitor[T any] interface {
	VisitVarStmt(stmt VarStmt[T]) error
	VisitExpressionStmt(stmt ExpressionStmt[T]) error
	VisitIfStmt(stmt IfStmt[T]) error
	VisitWhileStmt(stmt WhileStmt[T]) error
	VisitPrintStmt(stmt PrintStmt[T]) error
	VisitClassStmt(stmt ClassStmt[T]) error
	VisitBlockStmt(stmt BlockStmt[T]) error
	VisitBreakStmt(stmt BreakStmt[T]) error
	VisitFunctionStmt(stmt FunctionStmt[T]) error
	VisitReturnStmt(stmt ReturnStmt[T]) error
}

type VarStmt[T any] struct {
	Name        token.Token
	Initializer Expr[T]
}

func (e VarStmt[T]) Accept(visitor StmtVisitor[T]) error {
	return visitor.VisitVarStmt(e)
}

type ExpressionStmt[T any] struct {
	Expression Expr[T]
}

func (e ExpressionStmt[T]) Accept(visitor StmtVisitor[T]) error {
	return visitor.VisitExpressionStmt(e)
}

type IfStmt[T any] struct {
	Condition  Expr[T]
	ThenBranch Stmt[T]
	ElseBranch Stmt[T]
}

func (e IfStmt[T]) Accept(visitor StmtVisitor[T]) error {
	return visitor.VisitIfStmt(e)
}

type WhileStmt[T any] struct {
	Condition Expr[T]
	Body      Stmt[T]
}

func (e WhileStmt[T]) Accept(visitor StmtVisitor[T]) error {
	return visitor.VisitWhileStmt(e)
}

type PrintStmt[T any] struct {
	Expression Expr[T]
}

func (e PrintStmt[T]) Accept(visitor StmtVisitor[T]) error {
	return visitor.VisitPrintStmt(e)
}

type ClassStmt[T any] struct {
	Name          token.Token
	Superclass    *VarExpr[T]
	Methods       []FunctionStmt[T]
	GlobalMethods []FunctionStmt[T]
}

func (e ClassStmt[T]) Accept(visitor StmtVisitor[T]) error {
	return visitor.VisitClassStmt(e)
}

type BlockStmt[T any] struct {
	Statements []Stmt[T]
}

func (e BlockStmt[T]) Accept(visitor StmtVisitor[T]) error {
	return visitor.VisitBlockStmt(e)
}

type BreakStmt[T any] struct {
}

func (e BreakStmt[T]) Accept(visitor StmtVisitor[T]) error {
	return visitor.VisitBreakStmt(e)
}

type FunctionStmt[T any] struct {
	Name       token.Token
	Parameters []token.Token
	Body       []Stmt[T]
}

func (e FunctionStmt[T]) Accept(visitor StmtVisitor[T]) error {
	return visitor.VisitFunctionStmt(e)
}

type ReturnStmt[T any] struct {
	Keyword token.Token
	Value   Expr[T]
}

func (e ReturnStmt[T]) Accept(visitor StmtVisitor[T]) error {
	return visitor.VisitReturnStmt(e)
}
