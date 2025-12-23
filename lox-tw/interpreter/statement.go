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

func (i Interpreter) VisitClassStmt(stmt ast.ClassStmt[any]) error {
	var superclass *Class
	if stmt.Superclass != nil {
		superclassValue, err := stmt.Superclass.Accept(i)
		if err != nil {
			return err
		}

		var ok bool
		superclass, ok = superclassValue.(*Class)
		if !ok {
			return &RuntimeError{
				Token:   stmt.Superclass.Name,
				Message: "Superclass must be a class.",
			}
		}
	}

	i.environment.Define(stmt.Name.Lexeme, nil)

	if superclass != nil {
		i.environment = NewChildEnvironment(i.environment)
		i.environment.Define("super", superclass)
	}

	globalMethods := make(map[string]*Function)
	for _, method := range stmt.GlobalMethods {
		globalMethods[method.Name.Lexeme] = NewFunction(method, i.environment, false)
	}
	metaclass := NewClass(nil, stmt.Name.Lexeme+" metaclass", superclass, globalMethods)

	methods := make(map[string]*Function)
	for _, method := range stmt.Methods {
		methods[method.Name.Lexeme] = NewFunction(method, i.environment, method.Name.Lexeme == "init")
	}
	class := NewClass(metaclass, stmt.Name.Lexeme, superclass, methods)

	if superclass != nil {
		i.environment = i.environment.enclosing
	}

	i.environment.Assign(stmt.Name, class)

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
	function := NewFunction(stmt, i.environment, false)
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
