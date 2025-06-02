package main

import (
	"strconv"
)

func scanTokens(source string) []Token {
	tokens := []Token{}

	currentCharacterPosition := uint(0)
	for !allCharactersParsed(source, currentCharacterPosition) {
		scannedToken, position := scanToken(source, currentCharacterPosition, 1)

		if scannedToken.Type != NOTHING {
			tokens = append(tokens, scannedToken)
		}

		currentCharacterPosition = position
	}

	tokens = append(tokens, Token{
		Type:    EOF,
		Lexeme:  "",
		Literal: nil,
		Line:    0,
	})

	return tokens
}

func scanToken(source string, start uint, line uint) (Token, uint) {
	position := start
	currentCharacter := source[position]

	nextCharacter := byte(0)
	if position+1 < uint(len(source)) {
		nextCharacter = source[position+1]
	}

	if isComment(currentCharacter, nextCharacter) {
		for !allCharactersParsed(source, position) && source[position] != '\n' {
			position += 1
		}

		return NilToken(), position + 1
	}

	tokenType := TrySingleCharTokenType(currentCharacter)
	if tokenType != 0 {
		return Token{
			Type:    tokenType,
			Lexeme:  string(currentCharacter),
			Literal: nil,
			Line:    line,
		}, position + 1
	}

	tokenType, length := TryComparisonOperatorTokenType(currentCharacter, nextCharacter)
	if tokenType != 0 {
		return Token{
			Type:    tokenType,
			Lexeme:  source[position : position+length],
			Literal: nil,
			Line:    line,
		}, position + length
	}

	if isDigit(currentCharacter) {
		return scanDecimal(source, position, line)
	}

	if isAlpha(currentCharacter) {
		return scanIdentifier(source, position, line)
	}

	switch currentCharacter {
	case ' ':
	case '\r':
	case '\t':
		return NilToken(), position + 1
	case '\n':
		return NilToken(), position + 1
	case '"':
		return scanString(source, position, line)
	default:
		report(0, "", "Unexpected character: "+string(source[position]))
	}

	return Token{
		Type:    NOTHING,
		Lexeme:  "",
		Literal: nil,
		Line:    line,
	}, position + 1
}

func isComment(a, b byte) bool {
	return a == '/' && b == '/'
}

func allCharactersParsed(source string, position uint) bool {
	return position >= uint(len(source))
}

func scanString(source string, start uint, line uint) (Token, uint) {
	position := start + 1
	for !allCharactersParsed(source, position) && source[position] != '"' {
		if source[position] == '\n' {
			line++
		}
		position++
	}

	if allCharactersParsed(source, position) {
		report(line, "", "Unterminated string")
		return NilToken(), position
	}

	position += 1
	return Token{
		Type:    STRING,
		Lexeme:  source[start:position],
		Literal: source[start+1 : position-1],
		Line:    line,
	}, position
}

func scanDecimal(source string, start uint, line uint) (Token, uint) {
	position := start
	for !allCharactersParsed(source, position) && isDigit(source[position]) {
		position += 1
	}

	if allCharactersParsed(source, position) {
		value, _ := strconv.ParseFloat(source[start:position], 64)
		return Token{
			Type:    NUMBER,
			Lexeme:  source[start:position],
			Literal: value,
			Line:    line,
		}, position
	}

	if source[position] == '.' {
		if !allCharactersParsed(source, position+1) && isDigit(source[position+1]) {
			position += 1
		}
	}

	for !allCharactersParsed(source, position) && isDigit(source[position]) {
		position += 1
	}

	if allCharactersParsed(source, position) {
		value, _ := strconv.ParseFloat(source[start:position], 64)
		return Token{
			Type:    NUMBER,
			Lexeme:  source[start:position],
			Literal: value,
			Line:    line,
		}, position
	}

	value, _ := strconv.ParseFloat(source[start:position], 64)
	return Token{
		Type:    NUMBER,
		Lexeme:  source[start:position],
		Literal: value,
		Line:    line,
	}, position
}

func isDigit(c byte) bool {
	return c >= '0' && c <= '9'
}

func isAlpha(c byte) bool {
	return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
}

func isAlphaNumeric(c byte) bool {
	return isAlpha(c) || isDigit(c)
}

func scanIdentifier(source string, start uint, line uint) (Token, uint) {
	position := start
	for !allCharactersParsed(source, position) && isAlphaNumeric(source[position]) {
		position += 1
	}

	tokenType := TryKeywordTokenType(source[start:position])
	if tokenType == 0 {
		tokenType = IDENTIFIER
	}

	return Token{
		Type:    tokenType,
		Lexeme:  source[start:position],
		Literal: nil,
		Line:    line,
	}, position
}
