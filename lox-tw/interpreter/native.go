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
	return float64(time.Now().UnixMilli()), nil
}

func (c Clock) String() string {
	return "<native fn>"
}

type Function struct {
	declaration   ast.FunctionStmt[any]
	closure       *Environment
	isInitializer bool
}

func (f Function) String() string {
	return "<fn " + f.declaration.Name.Lexeme + ">"
}

func NewFunction(function ast.FunctionStmt[any], closure *Environment, isInitializer bool) *Function {
	return &Function{
		declaration:   function,
		closure:       closure,
		isInitializer: isInitializer,
	}
}

func (f *Function) Arity() int {
	return len(f.declaration.Parameters)
}

func (f *Function) Call(interpreter Interpreter, arguments []any) (any, error) {
	env := NewChildEnvironment(f.closure)
	newInterpreter := NewInterpreterWithEnv(env, interpreter.exprToDepth)

	for i, param := range f.declaration.Parameters {
		newInterpreter.environment.Define(param.Lexeme, arguments[i])
	}

	err := executeBlock(f.declaration.Body, newInterpreter)
	switch err := err.(type) {
	case *ReturnError:
		if f.isInitializer {
			return f.closure.GetAtByLexeme(0, "this")
		}

		return err.Value, nil
	default:
		if f.isInitializer {
			return f.closure.GetAtByLexeme(0, "this")
		}
		return nil, err
	}

	return nil, nil
}

func (f *Function) Bind(instance *Instance) *Function {
	env := NewChildEnvironment(f.closure)
	env.Define("this", instance)
	return NewFunction(f.declaration, env, f.isInitializer)
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

type Lambda struct {
	declaration ast.LambdaExpr[any]
	closure     *Environment
}

func (l Lambda) String() string {
	return "<lambda fn>"
}

func NewLambda(lambda ast.LambdaExpr[any], closure *Environment) *Lambda {
	return &Lambda{
		declaration: lambda,
		closure:     closure,
	}
}

func (l *Lambda) Arity() int {
	return len(l.declaration.Parameters)
}

func (l *Lambda) Call(interpreter Interpreter, arguments []any) (any, error) {
	env := NewChildEnvironment(l.closure)
	newInterpreter := NewInterpreterWithEnv(env, interpreter.exprToDepth)

	for i, param := range l.declaration.Parameters {
		newInterpreter.environment.Define(param.Lexeme, arguments[i])
	}

	err := executeBlock(l.declaration.Body, newInterpreter)
	switch err := err.(type) {
	case *ReturnError:
		return err.Value, nil
	default:
		return nil, err
	}

	return nil, nil
}
