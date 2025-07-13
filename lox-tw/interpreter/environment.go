package interpreter

import "lox-tw/token"

type Environment struct {
	enclosing *Environment
	variables map[string]any
}

func NewEnvironment() *Environment {
	return &Environment{
		enclosing: nil,
		variables: make(map[string]any),
	}
}

func (env *Environment) WithParent(parent *Environment) *Environment {
	env.enclosing = parent
	return env
}

func (env *Environment) Define(name string, value any) {
	env.variables[name] = value
}

func (env *Environment) Get(name token.Token) (any, error) {
	value, exists := env.variables[name.Lexeme]

	if !exists {
		if env.enclosing != nil {
			return env.enclosing.Get(name)
		}

		return nil, &RuntimeError{
			Token:   name,
			Message: "Undefined variable '" + name.Lexeme + "'.",
		}
	}

	return value, nil
}

func (env *Environment) Assign(name token.Token, value any) error {
	_, exists := env.variables[name.Lexeme]

	if !exists {
		if env.enclosing != nil {
			return env.enclosing.Assign(name, value)
		}

		return &RuntimeError{
			Token:   name,
			Message: "Undefined variable '" + name.Lexeme + "'.",
		}
	}

	env.variables[name.Lexeme] = value

	return nil
}
