package interpreter

import (
	"time"

	"lox-tw/ast"
)

type Clock struct{}

func (c Clock) Arity() int {
	return 0
}

func (c Clock) Call(interpreter Interpreter, arguments []any) (any, error) {
	return time.Now().UnixMilli(), nil
}

func (c Clock) String() string {
	return "<native fn>"
}

type Function struct {
	declaration ast.FunctionStmt[any]
	closure     *Environment
}

func (f Function) String() string {
	return "<fn " + f.declaration.Name.Lexeme + ">"
}

func NewFunction(function ast.FunctionStmt[any], closure *Environment) *Function {
	return &Function{
		declaration: function,
		closure:     closure,
	}
}

func (f *Function) Arity() int {
	return len(f.declaration.Parameters)
}

func (f *Function) Call(interpreter Interpreter, arguments []any) (any, error) {
	newInterpreter := NewInterpreter()
	newInterpreter.environment.WithParent(f.closure)

	for i, param := range f.declaration.Parameters {
		newInterpreter.environment.Define(param.Lexeme, arguments[i])
	}

	err := executeBlock(f.declaration.Body, newInterpreter)
	switch err := err.(type) {
	case *ReturnError:
		return err.Value, nil
	default:
		return nil, err
	}

	return nil, nil
}

func executeBlock(statements []ast.Stmt[any], interpreter *Interpreter) error {
	for _, statement := range statements {
		err := statement.Accept(interpreter)
		if err != nil {
			return err
		}
	}

	return nil
}