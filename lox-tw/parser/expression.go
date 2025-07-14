package parser

import (
	"fmt"
	"os"

	"lox-tw/ast"
	"lox-tw/token"
)

func parseExpression(tokens []token.Token, start int) (ast.Expr[any], int, error) {
	return parseAssign(tokens, start)
}

func parseAssign(tokens []token.Token, start int) (ast.Expr[any], int, error) {
	comma, endComma, err := parseComma(tokens, start)
	if err != nil || tokens[endComma].Type != token.EQUAL {
		return comma, endComma, err
	}

	endCommaToken := tokens[endComma]
	assign, endAssign, err := parseAssign(tokens, endComma+1)
	if err != nil {
		return assign, endAssign, err
	}

	v, ok := comma.(ast.VarExpr[any])
	if !ok {
		return comma, endAssign, &ParserError{
			Token:   endCommaToken,
			Message: "Invalid assignment target.",
		}
	}

	return ast.AssignExpr[any]{Name: v.Name, Value: assign}, endAssign, nil
}

func parseComma(tokens []token.Token, start int) (ast.Expr[any], int, error) {
	return parseLeftAssociativeRule("comma", parseTernary, tokens, start, []token.TokenType{token.COMMA})
}

func parseTernary(tokens []token.Token, start int) (ast.Expr[any], int, error) {
	conditionExpr, endEquality, err := parseEquality(tokens, start)
	if err != nil || tokens[endEquality].Type != token.QUESTION_MARK {
		return conditionExpr, endEquality, err
	}

	trueExpr, endExpression, err := parseExpression(tokens, endEquality+1)
	if err != nil {
		return trueExpr, endExpression, err
	}

	if tokens[endExpression].Type != token.COLON {
		return trueExpr, endExpression, &ParserError{
			Token:   tokens[endExpression],
			Message: "Expected ':' after true branch of ternary expression.",
		}
	}

	falseExpr, endSecondExpression, err := parseExpression(tokens, endExpression+1)
	if err != nil {
		return falseExpr, endSecondExpression, err
	}

	return ast.TernaryExpr[any]{Condition: conditionExpr, TrueExpr: trueExpr, FalseExpr: falseExpr}, endSecondExpression, nil
}

func parseEquality(tokens []token.Token, start int) (ast.Expr[any], int, error) {
	return parseLeftAssociativeRule("equality", parseComparison, tokens, start, []token.TokenType{token.EQUAL_EQUAL, token.BANG_EQUAL})
}

func parseComparison(tokens []token.Token, start int) (ast.Expr[any], int, error) {
	return parseLeftAssociativeRule("comparison", parseTerm, tokens, start, []token.TokenType{token.LESS, token.LESS_EQUAL, token.GREATER, token.GREATER_EQUAL})
}

func parseTerm(tokens []token.Token, start int) (ast.Expr[any], int, error) {
	return parseLeftAssociativeRule("term", parseFactor, tokens, start, []token.TokenType{token.PLUS, token.MINUS}, token.PLUS)
}

func parseFactor(tokens []token.Token, start int) (ast.Expr[any], int, error) {
	return parseLeftAssociativeRule("factor", parseUnary, tokens, start, []token.TokenType{token.STAR, token.SLASH})
}

func parseUnary(tokens []token.Token, start int) (ast.Expr[any], int, error) {
	if tokens[start].Type.NotIn(token.MINUS, token.BANG) {
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
				Message: "Expected ')' after expression.",
			}
		}

		return ast.GroupingExpr[any]{Expression: expr}, end + 1, nil
	case token.IDENTIFIER:
		return ast.VarExpr[any]{Name: tokens[start]}, start + 1, nil
	default:
		return ast.LiteralExpr[any]{Value: tokens[start].Literal}, start, &ParserError{
			Token:   tokens[start],
			Message: "Expect expression.",
		}
	}
}

func parseLeftAssociativeRule(
	operation string,
	parse func(tokens []token.Token, pos int) (ast.Expr[any], int, error),
	tokens []token.Token,
	start int,
	types []token.TokenType,
	otherTypes ...token.TokenType,
) (ast.Expr[any], int, error) {
	if otherTypes == nil {
		otherTypes = types
	}

	if tokens[start].Type.In(otherTypes...) {
		fmt.Fprintf(os.Stderr, "Error: Unexpected '%s' at the start of %s\n", tokens[start].Lexeme, operation)
		_, end, _ := parse(tokens, start+1)
		return ast.NothingExpr[any]{}, end, nil
	}

	leftExpr, leftEnd, err := parse(tokens, start)
	if err != nil {
		return leftExpr, leftEnd, err
	}

	for tokens[leftEnd].Type.In(types...) {
		rightExpr, rightEnd, err := parse(tokens, leftEnd+1)
		if err != nil {
			return rightExpr, rightEnd, err
		}

		leftExpr = ast.BinaryExpr[any]{Left: leftExpr, Operator: tokens[leftEnd], Right: rightExpr}
		leftEnd = rightEnd
	}

	return leftExpr, leftEnd, nil
}
