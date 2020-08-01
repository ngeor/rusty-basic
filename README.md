# rusty-basic

> An interpreter for QBasic, written in Rust.

[![Build Status](https://travis-ci.org/ngeor/rusty-basic.svg?branch=master)](https://travis-ci.org/ngeor/rusty-basic)


## Goals

- Be able to interpret the sample `TODO.BAS` from the `basic` Docker project.
- Be able to cross-compile to Rust
- Unit tests, with code coverage
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

Characters form lexemes.

Lexemes form parser tokens. At this point parsing is done.

```
input (file or str) -> CharOrEofReader -> Lexer -> BufLexer -> Parser
```

- CharOrEofReader offers peek/read functions over the consumed source, returning
  one `Option<char>` at a time.
- Lexer combines characters together into lexemes (Keyword, Digits, Word, Whitespace, Symbol, etc) and keeps track of their location (row/col).
- BufLexer offers peek/read/undo functions over the Lexer.
- Parser builds the parse tree of declarations, statements, expressions, etc.

### Linting

The next layer is linting, where the parse tree is transformed into a different
tree. In the resulting tree, all types are resolved. Built-in functions and
subs are identified.


### Instruction generation

The instruction generator converts the linted parser tree into a flat list of
instructions (similar to assembly instructions).

### Instruction interpretation

This is the runtime step where the program is being run, interpreted one
instruction at a time.


## Design issues

### Dealing with location

Lexemes, parsed tokens, instructions, all have a location (row / col). The same
for errors. There's the question of how to propagate this information in enums.

- Option 1 - Envelope

  e.g.

  ```rust
  pub enum Whatever {
    Foo(i32),
    Bar
  }

  pub struct WhateverNode(Whatever, Location);
  ```

- Option 2 - Embed

  e.g.

  ```rust
  pub enum Whatever {
    Foo(i32, Location),
    Bar(Location)
  }
  ```

- Option 3 - Neither

  This is applicable only for errors. The location that caused the error can
  be retrieved by the processing class (e.g. `Lexer`).

Regardless of the option, it can get more complicated for nested structs (e.g.
`IF` blocks), where the location information needs to be preserved for inner
elements as well.

Using an envelope leads to more types and a bit more accessing to get to the body.
Embedding on the other hand makes unit tests more difficult as we need to match
on the exact location, which isn't always important.

From a separation of concern principle, the envelope is the better solution,
as we don't define the same member over and over. However, it is cumbersome
to wrap/unwrap the body of the envelope.

**Status:** There is no silver bullet at this point. Enums tend to embed the location,
structs use the envelope approach with a common class `Locatable`.

### Code separation

Classes like lexer, parser, interpreter, etc tend to be organized in multiple
files, but they are still the same `struct` spanning multiple files. The
design is therefore quite monolithic.

### Adding new built-in functions/subs

~~Adding new built-ins involves touching a lot of places in the code. It would be
interesting to see if this can be turned around, modeling the built-in as a
struct implementing all necessary traits needed to make it fit into the puzzle
(e.g. type resolver, linter, implementation logic).~~ Resolved. Built-ins are
now self-contained modules.
