package main

func scanTokens(source string) []Token {
	tokens := []Token{}

	line, position := uint(1), uint(0)
	for !allCharactersParsed(source, position) {
		scannedToken := scanToken(source, position, line)
		if scannedToken.Type != NOTHING {
			tokens = append(tokens, scannedToken)
		}

		position, line = scannedToken.Position, scannedToken.Line
	}

	return append(tokens, EofToken(position, line))
}

func scanToken(source string, start uint, line uint) Token {
	position := start
	currentCharacter := source[position]

	nextCharacter := byte(0)
	if !allCharactersParsed(source, position+1) {
		nextCharacter = source[position+1]
	}

	if isComment(currentCharacter, nextCharacter) {
		for !allCharactersParsed(source, position) && source[position] != '\n' {
			position += 1
		}

		return NilToken(position, line)
	}

	tokenType := TrySingleCharTokenType(currentCharacter)
	if tokenType != NOTHING {
		return Token{
			Type:     tokenType,
			Lexeme:   string(currentCharacter),
			Literal:  nil,
			Line:     line,
			Position: position + 1,
		}
	}

	tokenType, length := TryComparisonOperatorTokenType(currentCharacter, nextCharacter)
	if tokenType != NOTHING {
		return Token{
			Type:     tokenType,
			Lexeme:   source[position : position+length],
			Literal:  nil,
			Line:     line,
			Position: position + length,
		}
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
		return NilToken(position+1, line)
	case '\n':
		return NilToken(position+1, line+1)
	case '"':
		return scanString(source, position, line)
	default:
		report(0, "", "Unexpected character: "+string(source[position]))
	}

	return NilToken(position+1, line)
}

func scanString(source string, start uint, line uint) Token {
	position := start + 1
	for !allCharactersParsed(source, position) && source[position] != '"' {
		if source[position] == '\n' {
			line += 1
		}
		position += 1
	}

	if allCharactersParsed(source, position) {
		report(line, "", "Unterminated string")
		return NilToken(position, line)
	}

	position += 1
	return StringToken(source[start:position], position, line)
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
