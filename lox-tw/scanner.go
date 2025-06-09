package main

func scanTokens(source string) ([]Token, error) {
	tokens := []Token{}

	line, position := uint(1), uint(0)
	for !allCharactersParsed(source, position) {
		scannedToken, err := scanToken(source, position, line)
		if err != nil {
			return nil, err
		}

		if scannedToken.Type != NOTHING {
			tokens = append(tokens, scannedToken)
		}

		position, line = scannedToken.Position, scannedToken.Line
	}

	return append(tokens, EofToken(position, line)), nil
}

func scanToken(source string, start uint, line uint) (Token, error) {
	position := start
	currentCharacter := source[position]

	nextCharacter := byte(0)
	if !allCharactersParsed(source, position+1) {
		nextCharacter = source[position+1]
	}

	if isSingleLineComment(currentCharacter, nextCharacter) {
		return scanSingleLineComment(source, position, line)
	}

	if MULTILINE_COMMENTS {
		if isMultiLineCommentStart(currentCharacter, nextCharacter) {
			return scanMultiLineComment(source, position, line)
		}
	}

	tokenType := TrySingleCharTokenType(currentCharacter)
	if tokenType != NOTHING {
		return Token{
			Type:     tokenType,
			Lexeme:   string(currentCharacter),
			Literal:  nil,
			Line:     line,
			Position: position + 1,
		}, nil
	}

	tokenType, length := TryComparisonOperatorTokenType(currentCharacter, nextCharacter)
	if tokenType != NOTHING {
		return Token{
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
	case ' ':
	case '\r':
	case '\t':
		return NilToken(position+1, line), nil
	case '\n':
		return NilToken(position+1, line+1), nil
	case '"':
		return scanString(source, position, line)
	default:
		return NilToken(position, line), &ScannerError{
			Line:    line,
			Where:   "",
			Message: "Unexpected character: " + string(source[position]),
		}
	}

	return NilToken(position+1, line), nil
}

func scanSingleLineComment(source string, start uint, line uint) (Token, error) {
	position := start + 2
	for !allCharactersParsed(source, position) && source[position] != '\n' {
		position += 1
	}

	return NilToken(position, line), nil
}

func scanMultiLineComment(source string, start uint, line uint) (Token, error) {
	position := start + 2
	for !allCharactersParsed(source, position+1) {
		if isMultiLineCommentEnd(source[position], source[position+1]) {
			return NilToken(position+2, line), nil
		}

		if source[position] == '\n' {
			line += 1
		}
		position += 1
	}

	return NilToken(position, line), &ScannerError{
		Line:    line,
		Where:   "",
		Message: "Unterminated multi-line comment",
	}
}

func scanString(source string, start uint, line uint) (Token, error) {
	position := start + 1
	for !allCharactersParsed(source, position) && source[position] != '"' {
		if source[position] == '\n' {
			line += 1
		}
		position += 1
	}

	if allCharactersParsed(source, position) {
		return NilToken(position, line), &ScannerError{
			Line:    line,
			Where:   "",
			Message: "Unterminated string",
		}
	}

	position += 1
	return StringToken(source[start:position], position, line), nil
}

func scanDecimal(source string, start uint, line uint) Token {
	position := start

	for !allCharactersParsed(source, position) && isDigit(source[position]) {
		position += 1
	}

	if !allCharactersParsed(source, position) && source[position] == '.' {
		if !allCharactersParsed(source, position) && isDigit(source[position+1]) {
			position += 1
		}
	}

	for !allCharactersParsed(source, position) && isDigit(source[position]) {
		position += 1
	}

	return NumberToken(source[start:position], position, line)
}

func scanIdentifier(source string, start uint, line uint) Token {
	position := start
	for !allCharactersParsed(source, position) && isAlphaNumeric(source[position]) {
		position += 1
	}

	tokenType := TryKeywordTokenType(source[start:position])
	if tokenType == NOTHING {
		tokenType = IDENTIFIER
	}

	return Token{
		Type:     tokenType,
		Lexeme:   source[start:position],
		Literal:  nil,
		Line:     line,
		Position: position,
	}
}
