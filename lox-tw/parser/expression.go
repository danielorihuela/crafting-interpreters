package parser

import (
	"fmt"
	"os"

	"lox-tw/ast"
	"lox-tw/token"
)

func parseExpression(tokens []token.Token, start int) (ast.Expr[any], int, error) {
	return parseComma(tokens, start)
}

func parseComma(tokens []token.Token, start int) (ast.Expr[any], int, error) {
	return parseLeftAssociativeRule("comma", parseAssign, tokens, start, []token.TokenType{token.COMMA})
}

func parseAssign(tokens []token.Token, start int) (ast.Expr[any], int, error) {
	comma, endComma, err := parseTernary(tokens, start)
	if err != nil || tokens[endComma].Type != token.EQUAL {
		return comma, endComma, err
	}

	endCommaToken := tokens[endComma]
	assign, endAssign, err := parseAssign(tokens, endComma+1)
	if err != nil {
		return assign, endAssign, err
	}

	if v, ok := comma.(ast.GetExpr[any]); ok {
		return ast.SetExpr[any]{Object: v.Object, Name: v.Name, Value: assign}, endAssign, nil
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

func parseTernary(tokens []token.Token, start int) (ast.Expr[any], int, error) {
	orExpr, endOr, err := parseOr(tokens, start)
	if err != nil || tokens[endOr].Type != token.QUESTION_MARK {
		return orExpr, endOr, err
	}

	trueExpr, endExpression, err := parseExpression(tokens, endOr+1)
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

	return ast.TernaryExpr[any]{Condition: orExpr, TrueExpr: trueExpr, FalseExpr: falseExpr}, endSecondExpression, nil
}

func parseOr(tokens []token.Token, start int) (ast.Expr[any], int, error) {
	return parseLeftAssociativeRule("or", parseAnd, tokens, start, []token.TokenType{token.OR})
}

func parseAnd(tokens []token.Token, start int) (ast.Expr[any], int, error) {
	return parseLeftAssociativeRule("and", parseEquality, tokens, start, []token.TokenType{token.AND})
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
		return parseCall(tokens, start)
	}

	expr, end, err := parseUnary(tokens, start+1)
	if err != nil {
		return expr, end, err
	}

	return ast.UnaryExpr[any]{Operator: tokens[start], Right: expr}, end, nil
}

func parseCall(tokens []token.Token, start int) (ast.Expr[any], int, error) {
	callee, end, err := parsePrimary(tokens, start)
	if err != nil {
		return callee, end, err
	}

	pos := end
	for {
		if tokens[pos].Type == token.LEFT_PAREN {
			callee, end, err = finishCall(callee, tokens, pos+1)
			if err != nil {
				return callee, end, err
			}
		} else if tokens[pos].Type == token.DOT {
			if tokens[pos+1].Type != token.IDENTIFIER {
				return nil, pos + 1, &ParserError{
					Token:   tokens[pos+1],
					Message: "Expect property name after '.'.",
				}
			}
			callee = ast.GetExpr[any]{Object: callee, Name: tokens[pos+1]}
			end = pos + 2
		} else {
			break
		}
		pos = end
	}

	return callee, pos, nil
}

func parsePrimary(tokens []token.Token, start int) (ast.Expr[any], int, error) {
	switch tokens[start].Type {
	case token.NUMBER, token.STRING, token.NIL:
		return ast.LiteralExpr[any]{Value: tokens[start].Literal}, start + 1, nil
	case token.TRUE:
		return ast.LiteralExpr[any]{Value: true}, start + 1, nil
	case token.FALSE:
		return ast.LiteralExpr[any]{Value: false}, start + 1, nil
	case token.THIS:
		return ast.ThisExpr[any]{Keyword: tokens[start]}, start + 1, nil
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
	case token.SUPER:
		if tokens[start+1].Type != token.DOT {
			return nil, start + 1, &ParserError{
				Token:   tokens[start+1],
				Message: "Expect '.' after 'super'.",
			}
		}
		if tokens[start+2].Type != token.IDENTIFIER {
			return nil, start + 2, &ParserError{
				Token:   tokens[start+2],
				Message: "Expect superclass method name.",
			}
		}
		return ast.SuperExpr[any]{Keyword: tokens[start], Method: tokens[start+2]}, start + 3, nil
	default:
		if tokens[start].Type == token.FUN && tokens[start+1].Type != token.IDENTIFIER {
			parameters, body, end, err := parseFunctionHelper("lambda", tokens, start+1)
			if err != nil {
				return nil, end, err
			}

			return ast.LambdaExpr[any]{Parameters: parameters, Body: body}, end, nil
		}
		return ast.LiteralExpr[any]{Value: tokens[start].Literal}, start, &ParserError{
			Token:   tokens[start],
			Message: "Expect expression.",
		}
	}
}

func finishCall(callee ast.Expr[any], tokens []token.Token, start int) (ast.Expr[any], int, error) {
	arguments := []ast.Expr[any]{}
	pos := start
	if tokens[pos].Type != token.RIGHT_PAREN {
		for {
			if len(arguments) >= 255 {
				return nil, pos, &ParserError{
					Token:   tokens[pos],
					Message: "Can't have more than 255 arguments.",
				}
			}

			arg, end, err := parseAssign(tokens, pos)
			if err != nil {
				return arg, end, err
			}

			arguments = append(arguments, arg)
			pos = end

			if tokens[pos].Type != token.COMMA {
				break
			}
			pos += 1
		}
	}

	if tokens[pos].Type != token.RIGHT_PAREN {
		return nil, pos, &ParserError{
			Token:   tokens[pos],
			Message: "Expect ')' after arguments.",
		}
	}

	return ast.CallExpr[any]{Callee: callee, Parenthesis: tokens[pos], Arguments: arguments}, pos + 1, nil
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

		if operation == "or" || operation == "and" {
			leftExpr = ast.LogicalExpr[any]{Left: leftExpr, Operator: tokens[leftEnd], Right: rightExpr}
		} else {
			leftExpr = ast.BinaryExpr[any]{Left: leftExpr, Operator: tokens[leftEnd], Right: rightExpr}
		}
		leftEnd = rightEnd
	}

	return leftExpr, leftEnd, nil
}
