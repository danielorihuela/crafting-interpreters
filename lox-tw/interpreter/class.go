package interpreter

type Class struct {
	Name    string
	Methods map[string]*Function

	// metaclasses
	instance *Instance
}

func NewClass(metaclass *Class, name string, methods map[string]*Function) *Class {
	class := &Class{Name: name, Methods: methods, instance: nil}
	class.instance = NewInstance(metaclass)

	return class
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
