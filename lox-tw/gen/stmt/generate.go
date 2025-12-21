package main

import (
	"bytes"
	"go/format"
	"os"
	"strings"
	"text/template"
)

type Field struct {
	Name string
	Type string
}

type Statement struct {
	Name   string
	Fields []Field
}

type Config struct {
	BaseType    string
	ReturnType  string
	Expressions []Statement
}

func main() {
	config := Config{
		BaseType:   "Stmt",
		ReturnType: "error",
		Expressions: []Statement{
			{"Var", []Field{{"Name", "token.Token"}, {"Initializer", "Expr[T]"}}},
			{"Expression", []Field{{"Expression", "Expr[T]"}}},
			{"If", []Field{{"Condition", "Expr[T]"}, {"ThenBranch", "Stmt[T]"}, {"ElseBranch", "Stmt[T]"}}},
			{"While", []Field{{"Condition", "Expr[T]"}, {"Body", "Stmt[T]"}}},
			{"Print", []Field{{"Expression", "Expr[T]"}}},
			{"Class", []Field{{"Name", "token.Token"}, {"Methods", "[]FunctionStmt[T]"}, {"GlobalMethods", "[]FunctionStmt[T]"}}},
			{"Block", []Field{{"Statements", "[]Stmt[T]"}}},
			{"Break", []Field{}},
			{"Function", []Field{{"Name", "token.Token"}, {"Parameters", "[]token.Token"}, {"Body", "[]Stmt[T]"}}},
			{"Return", []Field{{"Keyword", "token.Token"}, {"Value", "Expr[T]"}}},
		},
	}

	funcMap := template.FuncMap{
		"ToLower": strings.ToLower,
	}

	tmpl, err := template.New("visitor_gen.tmpl").Funcs(funcMap).ParseFiles("gen/visitor_gen.tmpl")
	if err != nil {
		panic(err)
	}

	var buf bytes.Buffer
	err = tmpl.Execute(&buf, config)
	if err != nil {
		panic(err)
	}

	formattedCode, err := format.Source(buf.Bytes())
	if err != nil {
		panic(err)
	}

	file, err := os.Create("ast/statement.go")
	if err != nil {
		panic(err)
	}
	defer file.Close()

	_, err = file.Write(formattedCode)
	if err != nil {
		panic(err)
	}
}
