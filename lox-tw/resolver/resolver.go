package resolver

import (
	"lox-tw/ast"
	"lox-tw/token"
)

type FunctionType uint8

const (
	NONE FunctionType = iota
	FUNCTION
)

type Resolver struct {
	scopes          []map[string]bool
	currentFunction FunctionType
	ExprToDepth     map[ast.Expr[any]]int
}

func NewResolver() *Resolver {
	return &Resolver{
		scopes:      make([]map[string]bool, 0),
		ExprToDepth: make(map[ast.Expr[any]]int),
	}
}

func (r *Resolver) beginScope() {
	r.scopes = append(r.scopes, make(map[string]bool))
}

func (r *Resolver) endScope() {
	r.scopes = r.scopes[:len(r.scopes)-1]
}

func (r *Resolver) declare(name token.Token) error {
	if len(r.scopes) == 0 {
		return nil
	}

	_, valueExists := r.scopes[len(r.scopes)-1][name.Lexeme]
	if valueExists {
		return &ResolverError{
			Token:   name,
			Message: "Already a variable with this name in this scope.",
		}
	}

	r.scopes[len(r.scopes)-1][name.Lexeme] = false

	return nil
}

func (r *Resolver) define(name token.Token) {
	if len(r.scopes) == 0 {
		return
	}

	scope := r.scopes[len(r.scopes)-1]
	scope[name.Lexeme] = true
}

func (r *Resolver) resolveLocal(expr ast.Expr[any], name token.Token) {
	for i := len(r.scopes) - 1; i >= 0; i-- {
		if _, valueExists := r.scopes[i][name.Lexeme]; valueExists {
			r.ExprToDepth[expr] = len(r.scopes) - 1 - i
			return
		}
	}
}
