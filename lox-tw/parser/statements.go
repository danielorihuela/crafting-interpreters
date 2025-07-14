package parser

import (
	"lox-tw/ast"
	"lox-tw/token"
)

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
			Message: "Expect variable name.",
		}
	}

	end := start + 1
	var initializer ast.Expr[any] = ast.NothingExpr[any]{}
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
			Message: "Expected ';' after variable declaration.",
		}
	}

	return ast.VarStmt[any]{Name: tokens[start], Initializer: initializer}, end + 1, nil
}

func parseStatement(tokens []token.Token, start int) (ast.Stmt[any], int, error) {
	if tokens[start].Type == token.PRINT {
		return parsePrintStatement(tokens, start+1)
	} else if tokens[start].Type == token.LEFT_BRACE {
		return parseBlockStatement(tokens, start+1)
	}

	return parseExpressionStatement(tokens, start)
}

func parseBlockStatement(tokens []token.Token, start int) (ast.Stmt[any], int, error) {
	var statements []ast.Stmt[any]
	pos := start

	for tokens[pos].Type != token.RIGHT_BRACE && tokens[pos].Type != token.EOF {
		declaration, end, err := parseDeclaration(tokens, pos)
		if err != nil {
			return nil, end, err
		}

		statements = append(statements, declaration)
		pos = end
	}

	if tokens[pos].Type != token.RIGHT_BRACE {
		return nil, pos, &ParserError{
			Token:   tokens[pos],
			Message: "Expected '}' to close block.",
		}
	}

	return ast.BlockStmt[any]{Statements: statements}, pos + 1, nil
}

func parseExpressionStatement(tokens []token.Token, start int) (ast.Stmt[any], int, error) {
	expr, end, err := parseExpression(tokens, start)
	if err != nil {
		return nil, end, err
	}

	if tokens[end].Type != token.SEMICOLON {
		return nil, end, &ParserError{
			Token:   tokens[end],
			Message: "Expected ';' after expression.",
		}
	}

	return ast.ExpressionStmt[any]{Expression: expr}, end + 1, nil
}

func parsePrintStatement(tokens []token.Token, start int) (ast.Stmt[any], int, error) {
	value, end, err := parseExpression(tokens, start)
	if err != nil {
		return nil, end, err
	}

	if tokens[end].Type != token.SEMICOLON {
		return nil, end, &ParserError{
			Token:   tokens[end],
			Message: "Expected ';' after expression.",
		}
	}

	return ast.PrintStmt[any]{Expression: value}, end + 1, nil
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
