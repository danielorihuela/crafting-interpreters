package interpreter

import (
	"fmt"
	"strconv"

	"lox-tw/ast"
	"lox-tw/utils"
)

func (i Interpreter) VisitVarStmt(stmt ast.VarStmt[any]) error {
	var value any = nil
	if stmt.Initializer != nil {
		var err error
		value, err = stmt.Initializer.Accept(i)
		if err != nil {
			return err
		}
	}

	i.environment.Define(stmt.Name.Lexeme, value)
	return nil
}

func (i Interpreter) VisitVarExpr(expr ast.VarExpr[any]) (any, error) {
	return i.environment.Get(expr.Name)
}

func (i Interpreter) VisitExpressionStmt(stmt ast.ExpressionStmt[any]) error {
	_, err := stmt.Expression.Accept(i)
	return err
}

func (i Interpreter) VisitIfStmt(stmt ast.IfStmt[any]) error {
	condition, err := stmt.Condition.Accept(i)
	if err != nil {
		return err
	}

	if utils.IsTruthy(condition) {
		return stmt.ThenBranch.Accept(i)
	} else if stmt.ElseBranch != nil {
		return stmt.ElseBranch.Accept(i)
	}

	return nil
}

func (i Interpreter) VisitWhileStmt(stmt ast.WhileStmt[any]) error {
	for {
		condition, err := stmt.Condition.Accept(i)
		if err != nil {
			return err
		}

		if !utils.IsTruthy(condition) {
			break
		}

		err = stmt.Body.Accept(i)
		if err != nil {
			return err
		}
	}

	return nil
}

func (i Interpreter) VisitPrintStmt(stmt ast.PrintStmt[any]) error {
	value, err := stmt.Expression.Accept(i)
	if err != nil {
		return err
	}

	switch v := value.(type) {
	case nil:
		fmt.Println("nil")
	case float64:
		finalValue := strconv.FormatFloat(v, 'f', -1, 64)
		fmt.Println(finalValue)
	default:
		fmt.Println(v)
	}

	return nil
}

func (i Interpreter) VisitBlockStmt(stmt ast.BlockStmt[any]) error {
	previousEnv := i.environment
	i.environment = NewEnvironment().WithParent(previousEnv)

	var err error = nil
	for _, statement := range stmt.Statements {
		err = statement.Accept(i)
	}

	return err
}
