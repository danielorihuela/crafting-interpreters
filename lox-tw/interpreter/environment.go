package interpreter

import "lox-tw/token"

type Environment struct {
	variables map[string]any
}

func NewEnvironment() *Environment {
	return &Environment{
		variables: make(map[string]any),
	}
}

func (env *Environment) Define(name string, value any) {
	env.variables[name] = value
}

func (env *Environment) Get(name string) (any, error) {
	value, exists := env.variables[name]

	if !exists {
		return nil, &RuntimeError{
			Message: "Undefined variable '" + name + "'",
		}
	}

	return value, nil
}

func (env *Environment) Assign(name token.Token, value any) error {
	_, exists := env.variables[name.Lexeme]

	if !exists {
		return &RuntimeError{
			Message: "Undefined variable '" + name.Lexeme + "'",
		}
	}

	env.variables[name.Lexeme] = value

	return nil
}
