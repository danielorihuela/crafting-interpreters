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

type Expression struct {
	Name   string
	Fields []Field
}

type Config struct {
	BaseType    string
	ReturnType  string
	Expressions []Expression
}

func main() {
	config := Config{
		BaseType:   "Expr",
		ReturnType: "T, error",
		Expressions: []Expression{
			{"Grouping", []Field{{"Expression", "Expr[T]"}}},
			{"Ternary", []Field{{"Condition", "Expr[T]"}, {"TrueExpr", "Expr[T]"}, {"FalseExpr", "Expr[T]"}}},
			{"Binary", []Field{{"Left", "Expr[T]"}, {"Operator", "token.Token"}, {"Right", "Expr[T]"}}},
			{"Unary", []Field{{"Operator", "token.Token"}, {"Right", "Expr[T]"}}},
			{"Call", []Field{{"Callee", "Expr[T]"}, {"Parenthesis", "token.Token"}, {"Arguments", "[]Expr[T]"}}},
			{"Logical", []Field{{"Left", "Expr[T]"}, {"Operator", "token.Token"}, {"Right", "Expr[T]"}}},
			{"Literal", []Field{{"Value", "any"}}},
			{"Nothing", nil},

			{"Var", []Field{{"Name", "token.Token"}}},
			{"Assign", []Field{{"Name", "token.Token"}, {"Value", "Expr[T]"}}},
			{"Lambda", []Field{{"Parameters", "[]token.Token"}, {"Body", "[]Stmt[T]"}}},
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

	file, err := os.Create("ast/expression.go")
	if err != nil {
		panic(err)
	}
	defer file.Close()

	_, err = file.Write(formattedCode)
	if err != nil {
		panic(err)
	}
}
