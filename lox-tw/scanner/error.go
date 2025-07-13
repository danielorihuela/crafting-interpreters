package scanner

import "fmt"

type ScannerError struct {
	Line    uint
	Message string
}

func (e *ScannerError) Error() string {
	return fmt.Sprintf("[line %d] Error: %s", e.Line, e.Message)
}
