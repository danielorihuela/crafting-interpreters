package parser

import (
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

	for tokens[pos].Type != token.EOF {
		stmt, end, err := parseDeclaration(tokens, pos)
		if err != nil {
			return nil, err
		}

		statements = append(statements, stmt)
		pos = end
	}

	return statements, nil
}
