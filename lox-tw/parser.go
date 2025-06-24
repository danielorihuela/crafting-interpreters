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

with ternary operator
expression     → ternary ;
ternary        → equality "?" ternary ":" ternary ;

with comma and ternary operator
expression     → comma ;
comma          → ternary ( "," ternary )* ;
ternary        → equality "?" ternary ":" ternary ;
*/
package main

func parseTokens(tokens []Token) (Expr[string], error) {
	expr, _, err := parseExpression(tokens, 0)
	return expr, err
}

func parseExpression(tokens []Token, start int) (Expr[string], int, error) {
	if COMMA_OPERATOR && TERNARY_OPERATOR {
		return parseComma(tokens, start)
	} else if COMMA_OPERATOR {
		return parseComma(tokens, start)
	} else if TERNARY_OPERATOR {
		return parseTernary(tokens, start)
	}

	return parseEquality(tokens, start)
}

func parseComma(tokens []Token, start int) (Expr[string], int, error) {
	var expr Expr[string]
	var end int
	var err error

	if TERNARY_OPERATOR {
		expr, end, err = parseTernary(tokens, start)
	} else {
		expr, end, err = parseEquality(tokens, start)
	}
	if err != nil || end >= len(tokens) {
		return expr, end, err
	}

	for end < len(tokens) && tokens[end].Type == COMMA {
		var right Expr[string]
		var right_end int
		if TERNARY_OPERATOR {
			right, right_end, err = parseTernary(tokens, end+1)
		} else {
			right, right_end, err = parseEquality(tokens, end+1)
		}
		if err != nil {
			return right, right_end, err
		}

		expr = BinaryExpr[string]{Left: expr, Operator: tokens[end], Right: right}
		end = right_end
	}

	return expr, end, nil
}

func parseTernary(tokens []Token, start int) (Expr[string], int, error) {
	expr, end, err := parseEquality(tokens, start)
	if err != nil || end >= len(tokens) {
		return expr, end, err
	}

	if end < len(tokens) && tokens[end].Type == QUESTION_MARK {
		true_expr, true_expr_end, err := parseTernary(tokens, end+1)
		if err != nil {
			return true_expr, true_expr_end, err
		}

		if true_expr_end < len(tokens) && tokens[true_expr_end].Type != COLON {
			return true_expr, true_expr_end, &ParserError{
				Token:   tokens[true_expr_end],
				Message: "Expected ':' after true branch of ternary expression",
			}
		}

		false_expr, false_expr_end, err := parseTernary(tokens, true_expr_end+1)
		if err != nil {
			return false_expr, false_expr_end, err
		}

		expr = TernaryExpr[string]{Condition: expr, TrueExpr: true_expr, FalseExpr: false_expr}
		end = false_expr_end
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
