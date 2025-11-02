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
	parentEnv := i.environment
	i.environment = NewChildEnvironment(parentEnv)

	for _, statement := range stmt.Statements {
		err := statement.Accept(i)
		if err != nil {
			return err
		}
	}

	return nil
}

func (i Interpreter) VisitBreakStmt(stmt ast.BreakStmt[any]) error {
	return &BreakError{}
}

func (i Interpreter) VisitFunctionStmt(stmt ast.FunctionStmt[any]) error {
	function := NewFunction(stmt, i.environment)
	i.environment.Define(stmt.Name.Lexeme, function)
	return nil
}

func (i Interpreter) VisitReturnStmt(stmt ast.ReturnStmt[any]) error {
	var value any = nil
	if stmt.Value != nil {
		var err error
		value, err = stmt.Value.Accept(i)
		if err != nil {
			return err
		}
	}

	return &ReturnError{Value: value}
}
