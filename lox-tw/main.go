package main

import (
	"fmt"
	"os"

	"bufio"

	"lox-tw/ast"
	"lox-tw/interpreter"
	"lox-tw/parser"
	"lox-tw/scanner"
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
	switch err.(type) {
	case *scanner.ScannerError:
		os.Exit(65)
	case *parser.ParserError:
		os.Exit(65)
	case *interpreter.RuntimeError:
		os.Exit(70)
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
	tokens, err := scanner.ScanTokens(source)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error scanning tokens: %v\n", err)
		return err
	}

	if os.Getenv("CHAPTER_04") == "1" {
		for _, token := range tokens {
			fmt.Println(token.String())
		}
	} else if os.Getenv("CHAPTER_06") == "1" {
		expr, _ := parser.ParseTokens(tokens)
		ast, _ := expr.Accept(ast.AstPrinter{})
		fmt.Println(ast)
	} else if os.Getenv("CHAPTER_07") == "1" {
		expr, _ := parser.ParseTokens(tokens)
		result, _ := expr.Accept(interpreter.Interpreter{})
		fmt.Println(result)
	} else {
		expr, err := parser.ParseTokens(tokens)
		if err != nil {
			return err
		}

		result, err := expr.Accept(interpreter.Interpreter{})
		if err != nil {
			return err
		}

		fmt.Println(result)
	}

	return nil
}
