package interpreter

type Class struct {
	Name    string
	Methods map[string]*Function
}

func NewClass(name string, methods map[string]*Function) *Class {
	return &Class{Name: name, Methods: methods}
}

func (c *Class) FindMethod(name string) *Function {
	if method, ok := c.Methods[name]; ok {
		return method
	}
	return nil
}

func (c *Class) String() string {
	return c.Name
}

func (c *Class) Arity() int {
	initializer := c.FindMethod("init")
	if initializer == nil {
		return 0
	}
	return initializer.Arity()
}

func (c *Class) Call(interpreter Interpreter, arguments []any) (any, error) {
	instance := NewInstance(c)
	initializer := c.FindMethod("init")
	if initializer != nil {
		_, err := initializer.Bind(instance).Call(interpreter, arguments)
		if err != nil {
			return nil, err
		}
	}

	return instance, nil
}
