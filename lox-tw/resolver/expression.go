package resolver

import (
	"lox-tw/ast"
)

func (r *Resolver) VisitAssignExpr(expr ast.AssignExpr[any]) (any, error) {
	value, err := expr.Value.Accept(r)
	r.resolveLocal(expr, expr.Name)

	return value, err
}

func (r *Resolver) VisitGroupingExpr(expr ast.GroupingExpr[any]) (any, error) {
	return expr.Expression.Accept(r)
}

func (r *Resolver) VisitTernaryExpr(expr ast.TernaryExpr[any]) (any, error) {
	if _, err := expr.Condition.Accept(r); err != nil {
		return nil, err
	}

	if _, err := expr.TrueExpr.Accept(r); err != nil {
		return nil, err
	}

	if _, err := expr.FalseExpr.Accept(r); err != nil {
		return nil, err
	}

	return nil, nil
}

func (r *Resolver) VisitBinaryExpr(expr ast.BinaryExpr[any]) (any, error) {
	if _, err := expr.Left.Accept(r); err != nil {
		return nil, err
	}

	if _, err := expr.Right.Accept(r); err != nil {
		return nil, err
	}

	return nil, nil
}

func (r *Resolver) VisitUnaryExpr(expr ast.UnaryExpr[any]) (any, error) {
	return expr.Right.Accept(r)
}

func (r *Resolver) VisitLogicalExpr(expr ast.LogicalExpr[any]) (any, error) {
	if _, err := expr.Left.Accept(r); err != nil {
		return nil, err
	}

	if _, err := expr.Right.Accept(r); err != nil {
		return nil, err
	}

	return nil, nil
}

func (r *Resolver) VisitLiteralExpr(expr ast.LiteralExpr[any]) (any, error) {
	return nil, nil
}

func (r *Resolver) VisitNothingExpr(expr ast.NothingExpr[any]) (any, error) {
	return nil, nil
}

func (r *Resolver) VisitCallExpr(expr ast.CallExpr[any]) (any, error) {
	if _, err := expr.Callee.Accept(r); err != nil {
		return nil, err
	}

	for _, argument := range expr.Arguments {
		if _, err := argument.Accept(r); err != nil {
			return nil, err
		}
	}

	return nil, nil
}

func (r *Resolver) VisitLambdaExpr(expr ast.LambdaExpr[any]) (any, error) {
	r.beginScope()
	for _, param := range expr.Parameters {
		r.declare(param)
		r.define(param)
	}

	for _, bodyStmt := range expr.Body {
		if err := bodyStmt.Accept(r); err != nil {
			return nil, err
		}
	}

	r.endScope()

	return nil, nil
}

func (r *Resolver) VisitVarExpr(expr ast.VarExpr[any]) (any, error) {
	if len(r.scopes) == 0 {
		r.resolveLocal(expr, expr.Name)

		return nil, nil
	}

	if value, valueExists := r.scopes[len(r.scopes)-1][expr.Name.Lexeme]; valueExists && !value {
		return nil, &ResolverError{
			Token:   expr.Name,
			Message: "Can't read local variable in its own initializer.",
		}
	}

	r.resolveLocal(expr, expr.Name)

	return nil, nil
}

func (r *Resolver) VisitGetExpr(expr ast.GetExpr[any]) (any, error) {
	return expr.Object.Accept(r)
}

func (r *Resolver) VisitSetExpr(expr ast.SetExpr[any]) (any, error) {
	if _, err := expr.Object.Accept(r); err != nil {
		return nil, err
	}

	if _, err := expr.Value.Accept(r); err != nil {
		return nil, err
	}

	return nil, nil
}

func (r *Resolver) VisitThisExpr(expr ast.ThisExpr[any]) (any, error) {
	if r.currentClass == NONE_CLASS {
		return nil, &ResolverError{
			Token:   expr.Keyword,
			Message: "Can't use 'this' outside of a class.",
		}
	}

	r.resolveLocal(expr, expr.Keyword)

	return nil, nil
}
