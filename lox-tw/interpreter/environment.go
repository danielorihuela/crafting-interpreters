package interpreter

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
