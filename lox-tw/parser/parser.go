package parser

import (
	"fmt"
	"os"

	"lox-tw/ast"
	"lox-tw/token"
)

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

func ParseTokens(tokens []token.Token) (ast.Expr[any], error) {
	expr, _, err := parseExpression(tokens, 0)
	return expr, err
}

func parseDeclaration(tokens []token.Token, start int) (ast.Stmt[any], int, error) {
	var stmt ast.Stmt[any]
	var end int
	var err error
	if tokens[start].Type == token.VAR {
		stmt, end, err = parseVarDeclaration(tokens, start+1)
	} else {
		stmt, end, err = parseStatement(tokens, start)
	}

	if err != nil {
		end = synchronize(tokens, end)
	}

	return stmt, end, err
}

func parseVarDeclaration(tokens []token.Token, start int) (ast.Stmt[any], int, error) {
	if tokens[start].Type != token.IDENTIFIER {
		return nil, start, &ParserError{
			Token:   tokens[start],
			Message: "Expected variable name",
		}
	}

	varName := tokens[start]
	var initializer ast.Expr[any] = ast.NothingExpr[any]{}
	end := start + 1
	var err error
	if tokens[end].Type == token.EQUAL {
		initializer, end, err = parseExpression(tokens, start+2)
		if err != nil {
			return nil, end, err
		}
	}

	if tokens[end].Type != token.SEMICOLON {
		return nil, start, &ParserError{
			Token:   tokens[start],
			Message: "Expected ';' after variable declaration",
		}
	}

	return ast.VarStmt[any]{Name: varName, Initializer: initializer}, end + 1, nil
}

func parseStatement(tokens []token.Token, start int) (ast.Stmt[any], int, error) {
	if tokens[start].Type == token.PRINT {
		return parsePrintStatement(tokens, start+1)
	}

	return parseExpressionStatement(tokens, start)
}

func parsePrintStatement(tokens []token.Token, start int) (ast.Stmt[any], int, error) {
	value, end, err := parseExpression(tokens, start)
	if err != nil {
		return nil, end, err
	}

	if tokens[end].Type != token.SEMICOLON {
		return nil, end, &ParserError{
			Token:   tokens[end],
			Message: "Expected ';' after expression",
		}
	}

	return ast.PrintStmt[any]{Expression: value}, end + 1, nil
}

func parseExpressionStatement(tokens []token.Token, start int) (ast.Stmt[any], int, error) {
	expr, end, err := parseExpression(tokens, start)
	if err != nil {
		return nil, end, err
	}

	if tokens[end].Type != token.SEMICOLON {
		return nil, end, &ParserError{
			Token:   tokens[end],
			Message: "Expected ';' after expression",
		}
	}

	return ast.ExpressionStmt[any]{Expression: expr}, end + 1, nil
}

func parseExpression(tokens []token.Token, start int) (ast.Expr[any], int, error) {
	return parseComma(tokens, start)
}

func parseComma(tokens []token.Token, start int) (ast.Expr[any], int, error) {
	var end int
	if tokens[start].Type == token.COMMA {
		fmt.Fprintf(os.Stderr, "Error: Unexpected '%s' at the start of comma\n", tokens[start].Lexeme)
		_, end, _ = parseTernary(tokens, start)
		return ast.NothingExpr[any]{}, end, nil
	}

	var expr ast.Expr[any]
	var err error

	expr, end, err = parseTernary(tokens, start)
	if err != nil {
		return expr, end, err
	}

	for tokens[end].Type == token.COMMA {
		var rightExpr ast.Expr[any]
		var rightEnd int
		rightExpr, rightEnd, err = parseTernary(tokens, end+1)
		if err != nil {
			return rightExpr, rightEnd, err
		}

		expr = ast.BinaryExpr[any]{Left: expr, Operator: tokens[end], Right: rightExpr}
		end = rightEnd
	}

	return expr, end, nil
}

func parseTernary(tokens []token.Token, start int) (ast.Expr[any], int, error) {
	expr, end, err := parseEquality(tokens, start)
	if err != nil {
		return expr, end, err
	}

	if tokens[end].Type == token.QUESTION_MARK {
		trueExpr, trueEnd, err := parseExpression(tokens, end+1)
		if err != nil {
			return trueExpr, trueEnd, err
		}

		if tokens[trueEnd].Type != token.COLON {
			return trueExpr, trueEnd, &ParserError{
				Token:   tokens[trueEnd],
				Message: "Expected ':' after true branch of ternary expression",
			}
		}

		falseExpr, falseEnd, err := parseExpression(tokens, trueEnd+1)
		if err != nil {
			return falseExpr, falseEnd, err
		}

		expr = ast.TernaryExpr[any]{Condition: expr, TrueExpr: trueExpr, FalseExpr: falseExpr}
		end = falseEnd
	}

	return expr, end, nil
}

func parseEquality(tokens []token.Token, start int) (ast.Expr[any], int, error) {
	if tokens[start].Type == token.EQUAL_EQUAL || tokens[start].Type == token.BANG_EQUAL {
		fmt.Fprintf(os.Stderr, "Error: Unexpected '%s' at the start of equality\n", tokens[start].Lexeme)
		_, end, _ := parseComparison(tokens, start+1)
		return ast.NothingExpr[any]{}, end, nil
	}

	expr, end, err := parseComparison(tokens, start)
	if err != nil {
		return expr, end, err
	}

	for tokens[end].Type == token.EQUAL_EQUAL || tokens[end].Type == token.BANG_EQUAL {
		righExpr, rightEnd, err := parseComparison(tokens, end+1)
		if err != nil {
			return righExpr, rightEnd, err
		}

		expr = ast.BinaryExpr[any]{Left: expr, Operator: tokens[end], Right: righExpr}
		end = rightEnd
	}

	return expr, end, nil
}

func parseComparison(tokens []token.Token, start int) (ast.Expr[any], int, error) {
	if tokens[start].Type == token.LESS || tokens[start].Type == token.LESS_EQUAL ||
		tokens[start].Type == token.GREATER || tokens[start].Type == token.GREATER_EQUAL {
		fmt.Fprintf(os.Stderr, "Error: Unexpected '%s' at the start of comparison\n", tokens[start].Lexeme)
		_, end, _ := parseTerm(tokens, start+1)
		return ast.NothingExpr[any]{}, end, nil
	}

	expr, end, err := parseTerm(tokens, start)
	if err != nil {
		return expr, end, err
	}

	for tokens[end].Type == token.LESS || tokens[end].Type == token.LESS_EQUAL ||
		tokens[end].Type == token.GREATER || tokens[end].Type == token.GREATER_EQUAL {
		rightExpr, rightEnd, err := parseTerm(tokens, end+1)
		if err != nil {
			return rightExpr, rightEnd, err
		}

		expr = ast.BinaryExpr[any]{Left: expr, Operator: tokens[end], Right: rightExpr}
		end = rightEnd
	}

	return expr, end, nil
}

func parseTerm(tokens []token.Token, start int) (ast.Expr[any], int, error) {
	if tokens[start].Type == token.PLUS {
		fmt.Fprintf(os.Stderr, "Error: Unexpected '%s' at the start of term\n", tokens[start].Lexeme)
		_, end, _ := parseFactor(tokens, start+1)
		return ast.NothingExpr[any]{}, end, nil
	}

	expr, end, err := parseFactor(tokens, start)
	if err != nil {
		return expr, end, err
	}

	for tokens[end].Type == token.PLUS || tokens[end].Type == token.MINUS {
		rightExpr, rightEnd, err := parseFactor(tokens, end+1)
		if err != nil {
			return rightExpr, rightEnd, err
		}

		expr = ast.BinaryExpr[any]{Left: expr, Operator: tokens[end], Right: rightExpr}
		end = rightEnd
	}

	return expr, end, nil
}

func parseFactor(tokens []token.Token, start int) (ast.Expr[any], int, error) {
	if tokens[start].Type == token.STAR || tokens[start].Type == token.SLASH {
		fmt.Fprintf(os.Stderr, "Error: Unexpected '%s' at the start of factor\n", tokens[start].Lexeme)
		_, end, _ := parseUnary(tokens, start+1)
		return ast.NothingExpr[any]{}, end, nil
	}

	expr, end, err := parseUnary(tokens, start)
	if err != nil {
		return expr, end, err
	}

	for tokens[end].Type == token.STAR || tokens[end].Type == token.SLASH {
		rightExpr, rightEnd, err := parseUnary(tokens, end+1)
		if err != nil {
			return rightExpr, rightEnd, err
		}

		expr = ast.BinaryExpr[any]{Left: expr, Operator: tokens[end], Right: rightExpr}
		end = rightEnd
	}

	return expr, end, nil
}

func parseUnary(tokens []token.Token, start int) (ast.Expr[any], int, error) {
	tokenType := tokens[start].Type
	if tokenType != token.MINUS && tokenType != token.BANG {
		return parsePrimary(tokens, start)
	}

	expr, end, err := parseUnary(tokens, start+1)
	if err != nil {
		return expr, end, err
	}

	return ast.UnaryExpr[any]{Operator: tokens[start], Right: expr}, end, nil
}

func parsePrimary(tokens []token.Token, start int) (ast.Expr[any], int, error) {
	switch tokens[start].Type {
	case token.NUMBER, token.STRING, token.NIL:
		return ast.LiteralExpr[any]{Value: tokens[start].Literal}, start + 1, nil
	case token.TRUE:
		return ast.LiteralExpr[any]{Value: true}, start + 1, nil
	case token.FALSE:
		return ast.LiteralExpr[any]{Value: false}, start + 1, nil
	case token.LEFT_PAREN:
		expr, end, err := parseExpression(tokens, start+1)
		if err != nil {
			return expr, end, err
		}

		if tokens[end].Type != token.RIGHT_PAREN {
			return expr, end, &ParserError{
				Token:   tokens[end],
				Message: "Expected ')' after expression",
			}
		}

		return ast.GroupingExpr[any]{Expression: expr}, end + 1, nil
	case token.IDENTIFIER:
		return ast.VarExpr[any]{Name: tokens[start]}, start + 1, nil
	default:
		return ast.LiteralExpr[any]{Value: tokens[start].Literal}, start, &ParserError{
			Token:   tokens[start],
			Message: "Expected expression",
		}
	}
}

func synchronize(tokens []token.Token, start int) int {
	for i := start; i < int(len(tokens)); i++ {
		if tokens[i].Type == token.SEMICOLON {
			return i + 1
		}

		switch tokens[i].Type {
		case token.CLASS, token.FUN, token.VAR, token.FOR, token.IF, token.WHILE, token.PRINT, token.RETURN:
			return i
		}
	}

	return int(len(tokens))
}
