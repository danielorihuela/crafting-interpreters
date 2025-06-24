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
	if err != nil {
		return expr, end, err
	}

	for tokens[end].Type == COMMA {
		var rightExpr Expr[string]
		var rightEnd int
		if TERNARY_OPERATOR {
			rightExpr, rightEnd, err = parseTernary(tokens, end+1)
		} else {
			rightExpr, rightEnd, err = parseEquality(tokens, end+1)
		}
		if err != nil {
			return rightExpr, rightEnd, err
		}

		expr = BinaryExpr[string]{Left: expr, Operator: tokens[end], Right: rightExpr}
		end = rightEnd
	}

	return expr, end, nil
}

func parseTernary(tokens []Token, start int) (Expr[string], int, error) {
	expr, end, err := parseEquality(tokens, start)
	if err != nil {
		return expr, end, err
	}

	if tokens[end].Type == QUESTION_MARK {
		trueExpr, trueEnd, err := parseTernary(tokens, end+1)
		if err != nil {
			return trueExpr, trueEnd, err
		}

		if tokens[trueEnd].Type != COLON {
			return trueExpr, trueEnd, &ParserError{
				Token:   tokens[trueEnd],
				Message: "Expected ':' after true branch of ternary expression",
			}
		}

		falseExpr, falseEnd, err := parseTernary(tokens, trueEnd+1)
		if err != nil {
			return falseExpr, falseEnd, err
		}

		expr = TernaryExpr[string]{Condition: expr, TrueExpr: trueExpr, FalseExpr: falseExpr}
		end = falseEnd
	}

	return expr, end, nil
}

func parseEquality(tokens []Token, start int) (Expr[string], int, error) {
	expr, end, err := parseComparison(tokens, start)
	if err != nil {
		return expr, end, err
	}

	for tokens[end].Type == EQUAL_EQUAL || tokens[end].Type == BANG_EQUAL {
		righExpr, rightEnd, err := parseComparison(tokens, end+1)
		if err != nil {
			return righExpr, rightEnd, err
		}

		expr = BinaryExpr[string]{Left: expr, Operator: tokens[end], Right: righExpr}
		end = rightEnd
	}

	return expr, end, nil
}

func parseComparison(tokens []Token, start int) (Expr[string], int, error) {
	expr, end, err := parseTerm(tokens, start)
	if err != nil {
		return expr, end, err
	}

	for tokens[end].Type == LESS || tokens[end].Type == LESS_EQUAL ||
		tokens[end].Type == GREATER || tokens[end].Type == GREATER_EQUAL {
		rightExpr, rightEnd, err := parseTerm(tokens, end+1)
		if err != nil {
			return rightExpr, rightEnd, err
		}

		expr = BinaryExpr[string]{Left: expr, Operator: tokens[end], Right: rightExpr}
		end = rightEnd
	}

	return expr, end, nil
}

func parseTerm(tokens []Token, start int) (Expr[string], int, error) {
	expr, end, err := parseFactor(tokens, start)
	if err != nil {
		return expr, end, err
	}

	for tokens[end].Type == PLUS || tokens[end].Type == MINUS {
		rightExpr, rightEnd, err := parseFactor(tokens, end+1)
		if err != nil {
			return rightExpr, rightEnd, err
		}

		expr = BinaryExpr[string]{Left: expr, Operator: tokens[end], Right: rightExpr}
		end = rightEnd
	}

	return expr, end, nil
}

func parseFactor(tokens []Token, start int) (Expr[string], int, error) {
	expr, end, err := parseUnary(tokens, start)
	if err != nil {
		return expr, end, err
	}

	for tokens[end].Type == STAR || tokens[end].Type == SLASH {
		rightExpr, rightEnd, err := parseUnary(tokens, end+1)
		if err != nil {
			return rightExpr, rightEnd, err
		}

		expr = BinaryExpr[string]{Left: expr, Operator: tokens[end], Right: rightExpr}
		end = rightEnd
	}

	return expr, end, nil
}

func parseUnary(tokens []Token, start int) (Expr[string], int, error) {
	tokenType := tokens[start].Type
	if tokenType != MINUS && tokenType != BANG {
		return parsePrimary(tokens, start)
	}

	expr, end, err := parseUnary(tokens, start+1)
	if err != nil {
		return expr, end, err
	}

	return UnaryExpr[string]{Operator: tokens[start], Right: expr}, end, nil
}

func parsePrimary(tokens []Token, start int) (Expr[string], int, error) {
	switch tokens[start].Type {
	case NUMBER, STRING, NIL:
		return LiteralExpr[string]{Value: tokens[start].Literal}, start + 1, nil
	case TRUE:
		return LiteralExpr[string]{Value: true}, start + 1, nil
	case FALSE:
		return LiteralExpr[string]{Value: false}, start + 1, nil
	case LEFT_PAREN:
		expr, end, err := parseExpression(tokens, start+1)
		if err != nil {
			return nil, 0, err
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
