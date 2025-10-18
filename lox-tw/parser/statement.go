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
	if tokens[start].Type == token.IF {
		return parseIfStatement(tokens, start+1)
	} else if tokens[start].Type == token.WHILE {
		return parseWhileStatement(tokens, start+1)
	} else if tokens[start].Type == token.FOR {
		return parseForStatement(tokens, start+1)
	} else if tokens[start].Type == token.PRINT {
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
			Message: "Expect ';' after expression.",
		}
	}

	return ast.ExpressionStmt[any]{Expression: expr}, end + 1, nil
}

func parseIfStatement(tokens []token.Token, start int) (ast.Stmt[any], int, error) {
	if tokens[start].Type != token.LEFT_PAREN {
		return nil, start, &ParserError{
			Token:   tokens[start],
			Message: "Expected '(' after 'if'.",
		}
	}

	condition, end, err := parseExpression(tokens, start+1)
	if err != nil {
		return nil, end, err
	}

	if tokens[end].Type != token.RIGHT_PAREN {
		return nil, end, &ParserError{
			Token:   tokens[end],
			Message: "Expected ')' after if condition.",
		}
	}

	thenBranch, end, err := parseStatement(tokens, end+1)
	if err != nil {
		return nil, end, err
	}

	var elseBranch ast.Stmt[any] = nil
	if tokens[end].Type == token.ELSE {
		elseBranch, end, err = parseStatement(tokens, end+1)
		if err != nil {
			return nil, end, err
		}
	}

	return ast.IfStmt[any]{Condition: condition, ThenBranch: thenBranch, ElseBranch: elseBranch}, end, nil
}

func parseWhileStatement(tokens []token.Token, start int) (ast.Stmt[any], int, error) {
	if tokens[start].Type != token.LEFT_PAREN {
		return nil, start, &ParserError{
			Token:   tokens[start],
			Message: "Expected '(' after 'while'.",
		}
	}

	condition, end, err := parseExpression(tokens, start+1)
	if err != nil {
		return nil, end, err
	}

	if tokens[end].Type != token.RIGHT_PAREN {
		return nil, end, &ParserError{
			Token:   tokens[end],
			Message: "Expected ')' after while condition.",
		}
	}

	body, end, err := parseStatement(tokens, end+1)
	if err != nil {
		return nil, end, err
	}

	return ast.WhileStmt[any]{Condition: condition, Body: body}, end, nil
}

func parseForStatement(tokens []token.Token, start int) (ast.Stmt[any], int, error) {
	if tokens[start].Type != token.LEFT_PAREN {
		return nil, start, &ParserError{
			Token:   tokens[start],
			Message: "Expected '(' after 'for'.",
		}
	}

	var initializer ast.Stmt[any]
	var end int
	var err error
	if tokens[start+1].Type == token.SEMICOLON {
		initializer = nil
		end = start + 2
	} else if tokens[start+1].Type == token.VAR {
		initializer, end, err = parseVarDeclaration(tokens, start+2)
		if err != nil {
			return nil, end, err
		}
	} else {
		initializer, end, err = parseExpressionStatement(tokens, start+1)
		if err != nil {
			return nil, end, err
		}
	}

	var condition ast.Expr[any] = nil
	if tokens[end].Type != token.SEMICOLON {
		condition, end, err = parseExpression(tokens, end)
		if err != nil {
			return nil, end, err
		}
	}
	if tokens[end].Type != token.SEMICOLON {
		return nil, end, &ParserError{
			Token:   tokens[end],
			Message: "Expected ';' after loop condition.",
		}
	}
	if tokens[end].Type == token.SEMICOLON {
		end += 1
	}

	var increment ast.Expr[any] = nil
	if tokens[end].Type != token.RIGHT_PAREN {
		increment, end, err = parseExpression(tokens, end)
		if err != nil {
			return nil, end, err
		}
	}
	if tokens[end].Type != token.RIGHT_PAREN {
		return nil, end, &ParserError{
			Token:   tokens[end],
			Message: "Expected ')' after for clauses.",
		}
	}
	if tokens[end].Type == token.RIGHT_PAREN {
		end += 1
	}

	body, end, err := parseStatement(tokens, end)
	if err != nil {
		return nil, end, err
	}

	// Desugar the for loop into a while loop
	if increment != nil {
		body = ast.BlockStmt[any]{Statements: []ast.Stmt[any]{
			body,
			ast.ExpressionStmt[any]{Expression: increment},
		}}
	}

	if condition == nil {
		condition = ast.LiteralExpr[any]{Value: true}
	}
	body = ast.WhileStmt[any]{Condition: condition, Body: body}

	if initializer != nil {
		body = ast.BlockStmt[any]{Statements: []ast.Stmt[any]{
			initializer,
			body,
		}}
	}

	return body, end, nil
}

func parsePrintStatement(tokens []token.Token, start int) (ast.Stmt[any], int, error) {
	value, end, err := parseExpression(tokens, start)
	if err != nil {
		return nil, end, err
	}

	if tokens[end].Type != token.SEMICOLON {
		return nil, end, &ParserError{
			Token:   tokens[end],
			Message: "Expect ';' after expression.",
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
