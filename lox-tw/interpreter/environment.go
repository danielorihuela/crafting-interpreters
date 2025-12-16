package interpreter

import "lox-tw/token"

type Environment struct {
	global    *Environment
	enclosing *Environment
	variables map[string]any
}

func NewRootEnvironment() *Environment {
	environment := &Environment{
		global:    nil,
		enclosing: nil,
		variables: make(map[string]any),
	}
	environment.global = environment
	environment.Define("clock", Clock{})

	return environment
}

func NewChildEnvironment(parent *Environment) *Environment {
	return &Environment{
		global:    parent.global,
		enclosing: parent,
		variables: make(map[string]any),
	}
}

func (env *Environment) Define(name string, value any) {
	env.variables[name] = value
}

func (env *Environment) Get(name token.Token) (any, error) {
	if value, exists := env.variables[name.Lexeme]; exists {
		return value, nil
	}

	if env.enclosing == nil {
		return nil, &RuntimeError{
			Token:   name,
			Message: "Undefined variable '" + name.Lexeme + "'.",
		}
	}

	return env.enclosing.Get(name)
}

func (env *Environment) GetAt(distance int, name token.Token) (any, error) {
	return env.GetAtByLexeme(distance, name.Lexeme)
}

func (env *Environment) GetAtByLexeme(distance int, name string) (any, error) {
	return env.ancestor(distance).variables[name], nil
}

func (env *Environment) ancestor(distance int) *Environment {
	currentEnv := env
	for i := 0; i < distance; i++ {
		currentEnv = currentEnv.enclosing
	}

	return currentEnv
}

func (env *Environment) GetGlobal(name token.Token) (any, error) {
	return env.global.Get(name)
}

func (env *Environment) Assign(name token.Token, value any) error {
	if _, exists := env.variables[name.Lexeme]; exists {
		env.variables[name.Lexeme] = value
		return nil
	}

	if env.enclosing == nil {
		return &RuntimeError{
			Token:   name,
			Message: "Undefined variable '" + name.Lexeme + "'.",
		}
	}

	return env.enclosing.Assign(name, value)
}

func (env *Environment) AssignAt(distance int, name token.Token, value any) error {
	env.ancestor(distance).variables[name.Lexeme] = value
	return nil
}

func (env *Environment) AssignGlobal(name token.Token, value any) error {
	return env.global.Assign(name, value)
}
