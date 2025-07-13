package scanner

import (
	"lox-tw/token"
	"strconv"
)

func ScanTokens(source string) ([]token.Token, error) {
	tokens := []token.Token{}

	position, line := uint(0), uint(1)
	for !allCharactersParsed(source, position) {
		scannedToken, err := scanToken(source, position, line)
		if err != nil {
			return nil, err
		}

		if scannedToken.Type != token.NOTHING {
			tokens = append(tokens, scannedToken)
		}

		position, line = scannedToken.Position, scannedToken.Line
	}

	return append(tokens, token.EofToken(position+1, line)), nil
}

func scanToken(source string, start uint, line uint) (token.Token, error) {
	position := start
	currentCharacter := source[position]

	nextCharacter := byte(0)
	if !allCharactersParsed(source, position+1) {
		nextCharacter = source[position+1]
	}

	if isSingleLineComment(currentCharacter, nextCharacter) {
		return scanSingleLineComment(source, position, line)
	}

	if isMultiLineCommentStart(currentCharacter, nextCharacter) {
		return scanMultiLineComment(source, position, line)
	}

	tokenType := token.TrySingleCharTokenType(currentCharacter)
	if tokenType != token.NOTHING {
		return token.Token{
			Type:     tokenType,
			Lexeme:   string(currentCharacter),
			Literal:  nil,
			Line:     line,
			Position: position + 1,
		}, nil
	}

	tokenType, length := token.TryComparisonOperatorTokenType(currentCharacter, nextCharacter)
	if tokenType != token.NOTHING {
		return token.Token{
			Type:     tokenType,
			Lexeme:   source[position : position+length],
			Literal:  nil,
			Line:     line,
			Position: position + length,
		}, nil
	}

	if isDigit(currentCharacter) {
		return scanDecimal(source, position, line), nil
	}

	if isAlpha(currentCharacter) {
		return scanIdentifier(source, position, line), nil
	}

	switch currentCharacter {
	case ' ', '\r', '\t':
		return token.NilToken(position+1, line), nil
	case '\n':
		return token.NilToken(position+1, line+1), nil
	case '"':
		return scanString(source, position, line)
	default:
		return token.NilToken(position, line), &ScannerError{
			Line:    line,
			Message: "Unexpected character: " + string(source[position]) + ".",
		}
	}
}

func scanSingleLineComment(source string, start uint, line uint) (token.Token, error) {
	position := start + 2
	for !allCharactersParsed(source, position) && source[position] != '\n' {
		position += 1
	}

	return token.NilToken(position, line), nil
}

func scanMultiLineComment(source string, start uint, line uint) (token.Token, error) {
	position := start + 2
	for !allCharactersParsed(source, position+1) {
		if isMultiLineCommentEnd(source[position], source[position+1]) {
			return token.NilToken(position+2, line), nil
		}

		if source[position] == '\n' {
			line += 1
		}
		position += 1
	}

	return token.NilToken(position, line), &ScannerError{
		Line:    line,
		Message: "Unterminated multi-line comment.",
	}
}

func scanString(source string, start uint, line uint) (token.Token, error) {
	position := start + 1
	for !allCharactersParsed(source, position) && source[position] != '"' {
		if source[position] == '\n' {
			line += 1
		}
		position += 1
	}

	if allCharactersParsed(source, position) {
		return token.NilToken(position, line), &ScannerError{
			Line:    line,
			Message: "Unterminated string.",
		}
	}

	position += 1
	lexeme := source[start:position]
	return token.Token{
		Type:     token.STRING,
		Lexeme:   lexeme,
		Literal:  lexeme[1 : len(lexeme)-1], // Remove quotes
		Line:     line,
		Position: position,
	}, nil
}

func scanDecimal(source string, start uint, line uint) token.Token {
	position := start

	for !allCharactersParsed(source, position) && isDigit(source[position]) {
		position += 1
	}

	if !allCharactersParsed(source, position) && source[position] == '.' {
		if !allCharactersParsed(source, position) && isDigit(source[position+1]) {
			position += 1
		}

		for !allCharactersParsed(source, position) && isDigit(source[position]) {
			position += 1
		}
	}

	lexeme := source[start:position]
	value, _ := strconv.ParseFloat(lexeme, 64)
	return token.Token{
		Type:     token.NUMBER,
		Lexeme:   lexeme,
		Literal:  value,
		Line:     line,
		Position: position,
	}
}

func scanIdentifier(source string, start uint, line uint) token.Token {
	position := start
	for !allCharactersParsed(source, position) && isAlphaNumeric(source[position]) {
		position += 1
	}

	tokenType := token.TryKeywordTokenType(source[start:position])
	if tokenType == token.NOTHING {
		tokenType = token.IDENTIFIER
	}

	return token.Token{
		Type:     tokenType,
		Lexeme:   source[start:position],
		Literal:  nil,
		Line:     line,
		Position: position,
	}
}
