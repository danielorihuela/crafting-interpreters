package interpreter

import (
	"lox-tw/token"
)

type Instance struct {
	class  *Class
	fields map[string]any
}

func NewInstance(class *Class) *Instance {
	return &Instance{class: class, fields: make(map[string]any)}
}

func (i *Instance) String() string {
	return i.class.Name + " instance"
}

func (i *Instance) Get(name token.Token) (any, error) {
	value, ok := i.fields[name.Lexeme]
	if ok && value != nil {
		return value, nil
	}

	if method := i.class.FindMethod(name.Lexeme); method != nil {
		method = method.Bind(i)
		return method, nil
	}

	return nil, &RuntimeError{
		Token:   name,
		Message: "Undefined property '" + name.Lexeme + "'."}
}

func (i *Instance) Set(name token.Token, value any) {
	i.fields[name.Lexeme] = value
}
