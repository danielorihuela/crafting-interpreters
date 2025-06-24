/*
grammar without ambiguity

expression     → equality ;
equality       → comparison ( ( "!=" | "==" ) comparison )* ;
comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
term           → factor ( ( "-" | "+" ) factor )* ;
factor         → unary ( ( "/" | "*" ) unary )* ;
unary          → ( "!" | "-" ) unary | primary ;
primary        → NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" ;

with comma operator

expression     → comma ;
comma          → equality ( "," equality )* ;
*/
package main

func parseTokens(tokens []Token) (Expr[string], error) {
	expr, _, err := parseExpression(tokens, 0)
	return expr, err
}

func parseExpression(tokens []Token, start int) (Expr[string], int, error) {
	if COMMA_OPERATOR {
		return parseComma(tokens, start)
	}

	return parseEquality(tokens, start)
}

func parseComma(tokens []Token, start int) (Expr[string], int, error) {
	expr, end, err := parseEquality(tokens, start)
	if err != nil || end >= len(tokens) {
		return expr, end, err
	}

	for end < len(tokens) && tokens[end].Type == COMMA {
		right, right_end, err := parseEquality(tokens, end+1)
		if err != nil {
			return right, right_end, err
		}

		expr = BinaryExpr[string]{Left: expr, Operator: tokens[end], Right: right}
		end = right_end
	}

	return expr, end, nil
}

func parseEquality(tokens []Token, start int) (Expr[string], int, error) {
	expr, end, err := parseComparison(tokens, start)
	if err != nil || end >= len(tokens) {
		return expr, end, err
	}

	for end < len(tokens) && (tokens[end].Type == EQUAL_EQUAL || tokens[end].Type == BANG_EQUAL) {
		right, right_end, err := parseComparison(tokens, end+1)
		if err != nil {
			return right, right_end, err
		}

		expr = BinaryExpr[string]{Left: expr, Operator: tokens[end], Right: right}
		end = right_end
	}

	return expr, end, nil
}

func parseComparison(tokens []Token, start int) (Expr[string], int, error) {
	expr, end, err := parseTerm(tokens, start)
	if err != nil || end >= len(tokens) {
		return expr, end, err
	}

	for end < len(tokens) && (tokens[end].Type == LESS || tokens[end].Type == LESS_EQUAL ||
		tokens[end].Type == GREATER || tokens[end].Type == GREATER_EQUAL) {
		right, right_end, err := parseTerm(tokens, end+1)
		if err != nil {
			return right, right_end, err
		}

		expr = BinaryExpr[string]{Left: expr, Operator: tokens[end], Right: right}
		end = right_end
	}

	return expr, end, nil
}

func parseTerm(tokens []Token, start int) (Expr[string], int, error) {
	expr, end, err := parseFactor(tokens, start)
	if err != nil || end >= len(tokens) {
		return expr, end, err
	}

	for end < len(tokens) && (tokens[end].Type == PLUS || tokens[end].Type == MINUS) {
		right, right_end, err := parseFactor(tokens, end+1)
		if err != nil {
			return right, right_end, err
		}

		expr = BinaryExpr[string]{Left: expr, Operator: tokens[end], Right: right}
		end = right_end
	}

	return expr, end, nil
}

func parseFactor(tokens []Token, start int) (Expr[string], int, error) {
	expr, end, err := parseUnary(tokens, start)
	if err != nil || end >= len(tokens) {
		return expr, end, err
	}

	for end < len(tokens) && (tokens[end].Type == STAR || tokens[end].Type == SLASH) {
		right, right_end, err := parseUnary(tokens, end+1)
		if err != nil {
			return right, right_end, err
		}

		expr = BinaryExpr[string]{Left: expr, Operator: tokens[end], Right: right}
		end = right_end
	}

	return expr, end, nil
}

func parseUnary(tokens []Token, start int) (Expr[string], int, error) {
	if start >= len(tokens) {
		return parsePrimary(tokens, start)
	}

	tokenType := tokens[start].Type
	if tokenType != MINUS && tokenType != BANG {
		return parsePrimary(tokens, start)
	}

	right, end, err := parseUnary(tokens, start+1)
	if err != nil {
		return nil, 0, err
	}

	return UnaryExpr[string]{Operator: tokens[start], Right: right}, end, nil
}

func parsePrimary(tokens []Token, start int) (Expr[string], int, error) {
	if len(tokens) <= int(start) {
		return nil, 0, &ParserError{
			Token:   EofToken(tokens[start-1].Position, tokens[start-1].Line),
			Message: "Expected expression",
		}
	}

	switch tokens[start].Type {
	case NUMBER:
		return LiteralExpr[string]{Value: tokens[start].Literal}, start + 1, nil
	case STRING:
		return LiteralExpr[string]{Value: tokens[start].Literal}, start + 1, nil
	case TRUE:
		return LiteralExpr[string]{Value: true}, start + 1, nil
	case FALSE:
		return LiteralExpr[string]{Value: false}, start + 1, nil
	case NIL:
		return LiteralExpr[string]{Value: nil}, start + 1, nil
	case LEFT_PAREN:
		expr, end, err := parseExpression(tokens, start+1)
		if err != nil {
			return nil, 0, err
		}
		if end >= int(len(tokens)) {
			return nil, 0, &ParserError{
				Token:   EofToken(tokens[end-1].Position, tokens[end-1].Line),
				Message: "Expected ')' after expression",
			}
		}

		if tokens[end].Type != RIGHT_PAREN {
			return nil, 0, &ParserError{
				Token:   tokens[end],
				Message: "Expected ')' after expression",
			}
		}
		return GroupingExpr[string]{Expression: expr}, end + 1, nil
	default:
		return nil, 0, &ParserError{
			Token:   tokens[start],
			Message: "Expected expression",
		}
	}
}

func synchronize(tokens []Token, start int) int {
	for i := start; i < int(len(tokens)); i++ {
		if tokens[i].Type == SEMICOLON {
			return i + 1
		}

		switch tokens[i].Type {
		case CLASS, FUN, VAR, FOR, IF, WHILE, PRINT, RETURN:
			return i
		}
	}

	return int(len(tokens))
}
