package main

import (
	"fmt"
	"os"

	"bufio"

	"lox-tw/ast"
	"lox-tw/interpreter"
	"lox-tw/parser"
	"lox-tw/resolver"
	"lox-tw/scanner"
)

var RUNTIME_ERROR = false

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
	case *resolver.ResolverError:
		os.Exit(65)
	}

	if RUNTIME_ERROR {
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
	chapter := os.Getenv("CHAPTER")
	switch chapter {
	case "4":
		chapter_4_run(source)
	case "6":
		chapter_6_run(source)
	case "7":
		chapter_7_run(source)
	case "8":
		return chapter_8_run(source)
	case "9":
		return chapter_9_run(source)
	case "10":
		return chapter_10_run(source)
	case "11":
		return chapter_11_run(source)
	default:
		return chapter_11_run(source)
	}

	return nil
}

func chapter_4_run(source string) {
	tokens, err := scanner.ScanTokens(source)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error scanning tokens: %v\n", err)
	}

	for _, token := range tokens {
		fmt.Println(token.String())
	}
}

func chapter_6_run(source string) {
	tokens, err := scanner.ScanTokens(source)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error scanning tokens: %v\n", err)
	}

	expr, _ := parser.ParseTokensToExpression(tokens)
	ast, _ := expr.Accept(ast.AnyPrinter{})
	fmt.Println(ast)
}

func chapter_7_run(source string) {
	tokens, err := scanner.ScanTokens(source)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error scanning tokens: %v\n", err)
	}

	expr, _ := parser.ParseTokensToExpression(tokens)
	result, _ := expr.Accept(interpreter.Interpreter{})
	fmt.Println(result)
}

func chapter_8_run(source string) error {
	tokens, err := scanner.ScanTokens(source)
	if err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return err
	}

	stmts, err := parser.ParseTokensToStmts(tokens)
	if err != nil {
		return err
	}

	codeResolver := resolver.NewResolver()
	for _, stmt := range stmts {
		err := stmt.Accept(codeResolver)
		if err != nil {
			fmt.Fprintf(os.Stderr, "%v\n", err)
			return err
		}
	}

	codeInterpreter := interpreter.NewInterpreter(codeResolver.ExprToDepth)
	for _, stmt := range stmts {
		err := stmt.Accept(codeInterpreter)
		if err != nil {
			fmt.Fprintf(os.Stderr, "%v\n", err)
			RUNTIME_ERROR = true
		}
	}

	return nil
}

func chapter_9_run(source string) error {
	tokens, err := scanner.ScanTokens(source)
	if err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return err
	}

	stmts, err := parser.ParseTokensToStmts(tokens)
	if err != nil {
		return err
	}

	codeResolver := resolver.NewResolver()
	for _, stmt := range stmts {
		err := stmt.Accept(codeResolver)
		if err != nil {
			fmt.Fprintf(os.Stderr, "%v\n", err)
			return err
		}
	}

	codeInterpreter := interpreter.NewInterpreter(codeResolver.ExprToDepth)
	for _, stmt := range stmts {
		err := stmt.Accept(codeInterpreter)
		if _, ok := err.(*interpreter.BreakError); ok {
			continue
		}
		if err != nil {
			fmt.Fprintf(os.Stderr, "%v\n", err)
			RUNTIME_ERROR = true
		}
	}

	return nil
}

func chapter_10_run(source string) error {
	return chapter_9_run(source)
}

func chapter_11_run(source string) error {
	tokens, err := scanner.ScanTokens(source)
	if err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return err
	}

	stmts, err := parser.ParseTokensToStmts(tokens)
	if err != nil {
		return err
	}

	codeResolver := resolver.NewResolver()
	for _, stmt := range stmts {
		err := stmt.Accept(codeResolver)
		if err != nil {
			fmt.Fprintf(os.Stderr, "%v\n", err)
			return err
		}
	}

	codeInterpreter := interpreter.NewInterpreter(codeResolver.ExprToDepth)
	for _, stmt := range stmts {
		err := stmt.Accept(codeInterpreter)
		if _, ok := err.(*interpreter.BreakError); ok {
			continue
		}
		if err != nil {
			fmt.Fprintf(os.Stderr, "%v\n", err)
			RUNTIME_ERROR = true
		}
	}

	return nil
}
