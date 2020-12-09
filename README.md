# rusty-basic

> An interpreter for QBasic, written in Rust.

[![Build Status](https://travis-ci.org/ngeor/rusty-basic.svg?branch=master)](https://travis-ci.org/ngeor/rusty-basic)

## Goals

The primary goal is to have an interpreter compatible with QBasic.

- Be able to interpret the sample
  [TODO.BAS](https://github.com/ngeor/kamino/blob/trunk/dockerfiles/basic/basic/rest-qb/TODO.BAS)
  from my
  [basic Docker project](https://github.com/ngeor/kamino/tree/trunk/dockerfiles/basic).
  ✅
- Be able to interpret `MONEY.BAS` from QBasic.

Secondary goals:

- Be able to cross-compile to Rust and/or JavaScript
- Unit tests for QBasic programs, with code coverage
- VS Code debugging

## Development

Tip: run tests continuously with `make watch` or
`nodemon -e rs -x "cargo test"`.

## Architecture

- Parsing
- Linting
- Instruction generation
- Instruction interpretation

### Parsing

A program is read from a file character by character.

```
input (file or str) -> CharReader -> EolReader -> Parser
```

- CharReader returns one character a time, working with a `BufRead` as its
  source.
- EolReader adds support for row-col position, handling new lines.
- Parsing is done with parser combinators, ending up in a parse tree of
  declarations, statements, expressions, etc.

### Linting

The next layer is linting, where the parse tree is transformed into a different
tree. In the resulting tree, all types are resolved. Built-in functions and subs
are identified.

### Instruction generation

The instruction generator converts the linted parser tree into a flat list of
instructions (similar to assembly instructions).

### Instruction interpretation

This is the runtime step where the program is being run, interpreted one
instruction at a time.

## Names

### Bare and qualified names

In QBasic, you can have a simple variable like this `A = 42`.

You can also specify its type like this `A$ = "Hello, world!"`.

In rusty-basic, the first style is called _bare name_ and the second style is
called _qualified name_. The character that denotes the type is called a _type
qualifier_.

There are five of these characters, matching the five built-in types:

- `%` for integer
- `&` for long
- `!` for single
- `#` for double
- `$` for string

Bare names also have a type. By default, it's single. So typing `A` and `A!`
will point to the same variable.

The default type can be changed to integer with the `DEFINT A-Z` statement.
There's also `DEFLNG`, `DEFSNG`, `DEFDBL` and `DEFSTR`.

This simple name resolution mechanism gets a bit more complicated with the `AS`
keyword.

### Extended names

For the lack of a better name, rusty-basic calls these variables _extended_:

- `DIM A AS INTEGER`
- `DIM A AS SomeUserDefinedType`
- `FUNCTION Add(A AS INTEGER, B AS INTEGER)`

These names:

- cannot have a type qualifier (i.e. you can't say `DIM A$ AS INTEGER`)
- when in scope, you can't have any other qualified name of the same bare name

So it's possible to have this:

```basic
A = 42 ' this is a single by default name resolution
A$ = "hello"
```

But not this:

```basic
DIM A AS INTEGER
A = 42 ' this is an integer because it's explicitly defined as such
A$ = "hello" ' duplicate definition error here
```

## Syntax reference

```
<program> ::= <top-level-token> | <top-level-token><program>

<top-level-token> ::= <comment>
    | <def-type>
    | <declaration>
    | <statement>
    | <function>
    | <sub>
    | <user-defined-type>

<statement> ::= <comment>
    | <dim>
    | <const>
    | <built-in>
    | <label>
    | <sub-call>
    | <assignment>
    | <if-block>
    | <for-loop>
    | <select-case>
    | <while-wend>
    | <go-to>
    | <on-error-go-to>
```

### DIM statement

```
<dim> ::= "DIM"<ws><dim-name>
<dim-name> ::= <bare-dim-name> | <compact-dim-name> | <extended-dim-name> | <user-defined-dim-name>

(* DIM A, DIM A.B.C, DIM A.., DIM A(1 TO 2) *)
<bare-dim-name> ::= <bare-name-with-dots-not-keyword><opt-array-dimensions>

(* DIM A$, DIM A.B.C!, DIM A..% *)
<compact-dim-name> ::= <compact-dim-name-left><opt-array-dimensions>
(* it is allowed to have a keyword qualified by a string qualifier *)
<compact-dim-name-left> ::= <bare-name-with-dots-not-keyword> ( "!" | "#" | "%" | "&")
    | <bare-name-with-dots> "$"

<extended-dim-name> ::= <bare-name-with-dots-not-keyword><opt-array-dimensions> <ws> "AS" <ws> <extended-dim-type>
<extended-dim-type> ::= "INTEGER"
    | "LONG"
    | "SINGLE"
    | "DOUBLE"
    | <extended-dim-string>
<extended-dim-string> ::= "STRING" <opt-ws> "*" <opt-ws> <expression> | "STRING"

(* user defined type variable cannot have dots *)
<user-defined-dim-name> ::= <bare-name-not-keyword><opt-array-dimensions> <ws> "AS" <ws> <user-defined-type>
<user-defined-type> ::= <bare-name-not-keyword>

<opt-array-dimensions> ::= "" | "(" <opt-ws> <array-dimensions> <opt-ws> ")"
<array-dimensions> ::= <array-dimension> | <array-dimension> <opt-ws> "," <opt-ws> <array-dimensions>
<array-dimension> ::= <expression> | <expression> <ws> "TO" <ws> <expression>
```

### Expression

```
<expression> ::= TODO
```

### Names

```
<qualifier> ::= "!" | "#" | "$" | "%" | "&"

<bare-name-with-dots-not-keyword> ::= <bare-name-with-dots> AND NOT keyword
<bare-name-with-dots> ::= <letter> | <letter><letters-or-digits-or-dots>

<bare-name-not-keyword> ::= <bare-name> AND NOT keyword
<bare-name> ::= <letter> | <letter><letters-or-digits>

<letters-or-digits-or-dots> ::= <letter-or-digit-or-dot> | <letter-or-digit-or-dot><letters-or-digits-or-dots>
<letter-or-digit-or-dot> ::= <letter> | <digit> | "."

<letters-or-digits> ::= <letter-or-digit> | <letter-or-digit><letters-or-digits>
<letter-or-digit> ::= <letter> | <digit>
```

### Fundamentals

```
(* zero or more whitespace *)
<opt-ws> ::= "" | <ws>
(* at least one whitespace *)
<ws> ::= " " | " "<ws>
<letter> ::= "A".."Z" | "a".."z"
<digit> ::= "0".."9"
```
