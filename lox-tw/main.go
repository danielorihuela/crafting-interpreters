package main

import (
	"fmt"
	"os"

	"bufio"
)

func main() {
	arguments := os.Args[1:]

	if len(arguments) > 1 {
		fmt.Println("Usage: lox-tw [script]")
		os.Exit(64)
	} else if len(arguments) == 1 {
		runFile(arguments[0])
	} else {
		runPrompt()
	}
}

func runFile(path string) {
	content, err := os.ReadFile(path)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error reading file: %v\n", err)
		os.Exit(66)
	}

	err = run(string(content))
	if err != nil {
		os.Exit(65)
	}
}

func runPrompt() {
	fmt.Println("Entering interactive mode. Type 'Control-D' to quit.")
	fmt.Print("> ")

	stdin := bufio.NewScanner(os.Stdin)
	for stdin.Scan() {
		line := stdin.Text()
		if line == "" {
			break
		}

		run(line)
		fmt.Print("> ")
	}

	if err := stdin.Err(); err != nil {
		fmt.Fprintf(os.Stderr, "Error reading input: %v\n", err)
		os.Exit(1)
	}
}

func run(source string) error {
	tokens, err := scanTokens(source)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error scanning tokens: %v\n", err)
		return err
	}

	for _, token := range tokens {
		fmt.Println(token.String())
	}

	return nil
}
