package main

import (
	"fmt"
	"os"
)

func report(line uint, where, message string) {
	fmt.Fprintf(os.Stderr, "[line %d] Error %s: %s\n", line, where, message)
}
