package interpreter

type Callable interface {
	Call(interpreter Interpreter, arguments []any) (any, error)
	Arity() int
}
