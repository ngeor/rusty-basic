# rusty-basic

> An interpreter for QBasic, written in Rust.



## Goals

The primary goal is to have an interpreter compatible with QBasic.

- Be able to interpret the sample
  [TODO.BAS](https://github.com/ngeor/kamino/blob/trunk/dockerfiles/basic/basic/rest-qb/TODO.BAS)
  from my
  [basic Docker project](https://github.com/ngeor/kamino/tree/trunk/dockerfiles/basic)
  ✅
- Be able to interpret `MONEY.BAS` (an original demo program from QBasic) ✅

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
