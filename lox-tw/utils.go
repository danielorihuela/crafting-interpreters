package main

func isComment(a, b byte) bool {
	return a == '/' && b == '/'
}

func allCharactersParsed(source string, position uint) bool {
	return position >= uint(len(source))
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
