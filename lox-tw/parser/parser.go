package parser

import (
	"fmt"
	"os"

	"lox-tw/ast"
	"lox-tw/token"
)

func ParseTokensToExpression(tokens []token.Token) (ast.Expr[any], error) {
	expr, _, err := parseExpression(tokens, 0)
	return expr, err
}

func ParseTokensToStmts(tokens []token.Token) ([]ast.Stmt[any], error) {
	var statements []ast.Stmt[any]
	pos := 0

	var finalError error
	for pos < len(tokens) && tokens[pos].Type != token.EOF {
		stmt, end, err := parseDeclaration(tokens, pos)
		if end == pos {
			break
		}
		pos = end

		if err != nil {
			fmt.Fprintf(os.Stderr, "%v\n", err)
			finalError = err
			continue
		}

		statements = append(statements, stmt)
	}

	return statements, finalError
}
