package resolver

import (
	"lox-tw/ast"
)

func (r *Resolver) VisitVarStmt(stmt ast.VarStmt[any]) error {
	if err := r.declare(stmt.Name); err != nil {
		return err
	}

	if stmt.Initializer != nil {
		if _, err := stmt.Initializer.Accept(r); err != nil {
			return err
		}
	}
	r.define(stmt.Name)

	return nil
}

func (r *Resolver) VisitExpressionStmt(stmt ast.ExpressionStmt[any]) error {
	_, err := stmt.Expression.Accept(r)
	return err
}

func (r *Resolver) VisitIfStmt(stmt ast.IfStmt[any]) error {
	if _, err := stmt.Condition.Accept(r); err != nil {
		return err
	}

	if err := stmt.ThenBranch.Accept(r); err != nil {
		return err
	}

	if stmt.ElseBranch != nil {
		if err := stmt.ElseBranch.Accept(r); err != nil {
			return err
		}
	}

	return nil
}

func (r *Resolver) VisitWhileStmt(stmt ast.WhileStmt[any]) error {
	if _, err := stmt.Condition.Accept(r); err != nil {
		return err
	}

	if err := stmt.Body.Accept(r); err != nil {
		return err
	}

	return nil
}

func (r *Resolver) VisitPrintStmt(stmt ast.PrintStmt[any]) error {
	_, err := stmt.Expression.Accept(r)
	return err
}

func (r *Resolver) VisitClassStmt(stmt ast.ClassStmt[any]) error {
	enclosingClass := r.currentClass
	r.currentClass = CLASS

	if err := r.declare(stmt.Name); err != nil {
		return err
	}
	r.define(stmt.Name)

	r.beginScope()
	r.defineByLexeme("this")
	for _, method := range stmt.Methods {
		declaration := METHOD
		if method.Name.Lexeme == "init" {
			declaration = INITIALIZER
		}
		err := r.resolveFunction(method, declaration)
		if err != nil {
			return err
		}
	}
	r.endScope()

	r.currentClass = enclosingClass

	return nil
}

func (r *Resolver) VisitBlockStmt(stmt ast.BlockStmt[any]) error {
	r.beginScope()
	for _, statement := range stmt.Statements {
		if err := statement.Accept(r); err != nil {
			return err
		}
	}
	r.endScope()

	return nil
}

func (r *Resolver) VisitBreakStmt(stmt ast.BreakStmt[any]) error {
	return nil
}

func (r *Resolver) VisitFunctionStmt(stmt ast.FunctionStmt[any]) error {
	if err := r.declare(stmt.Name); err != nil {
		return err
	}
	r.define(stmt.Name)

	return r.resolveFunction(stmt, FUNCTION)
}

func (r *Resolver) resolveFunction(stmt ast.FunctionStmt[any], functionType FunctionType) error {
	previousFunction := r.currentFunction
	r.currentFunction = functionType
	r.beginScope()
	for _, param := range stmt.Parameters {
		if err := r.declare(param); err != nil {
			return err
		}

		r.define(param)
	}

	for _, bodyStmt := range stmt.Body {
		if err := bodyStmt.Accept(r); err != nil {
			return err
		}
	}

	r.endScope()
	r.currentFunction = previousFunction

	return nil
}

func (r *Resolver) VisitReturnStmt(stmt ast.ReturnStmt[any]) error {
	if r.currentFunction == NONE {
		return &ResolverError{
			Token:   stmt.Keyword,
			Message: "Can't return from top-level code.",
		}
	}

	if stmt.Value != nil {
		if r.currentFunction == INITIALIZER {
			return &ResolverError{
				Token:   stmt.Keyword,
				Message: "Can't return a value from an initializer.",
			}
		}

		if _, err := stmt.Value.Accept(r); err != nil {
			return err
		}
	}

	return nil
}
