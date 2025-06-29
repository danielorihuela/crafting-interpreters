package scanner

import "fmt"

type ScannerError struct {
	Line    uint
	Where   string
	Message string
}

func (e *ScannerError) Error() string {
	return fmt.Sprintf("[line %d] Scanner error %s: %s", e.Line, e.Where, e.Message)
}
