package interpreter

import "lox-tw/ast"

type Interpreter struct {
	environment *Environment
	exprToDepth map[ast.Expr[any]]int
}

func NewInterpreter(exprToDepth map[ast.Expr[any]]int) *Interpreter {
	return &Interpreter{
		environment: NewRootEnvironment(),
		exprToDepth: exprToDepth,
	}
}

func NewInterpreterWithEnv(env *Environment, exprToDepth map[ast.Expr[any]]int) *Interpreter {
	return &Interpreter{
		environment: env,
		exprToDepth: exprToDepth,
	}
}
