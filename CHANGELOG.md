# Changelog

All notable changes to this project will be documented in this file.

## [0.9.1] - 2025-12-30

### 🐛 Bug Fixes

- *(ci)* Do not run the build workflow for tags
- *(ci)* Correct artifact paths for release workflow

## [0.9.0] - 2025-12-30

### 🚀 Features

- Add RowColView to map a string to row/col taking newlines into account
- Added RcStringView as the basic input unit that is lightweight and clone-able, and suppors row-col information
- Added Opt parser
- Added surround parser
- Added `flat_map_negate_none`
- Decouple `Parser` from `ParseError`
- Move `no_incomplete` to the `Parser` level

### 🐛 Bug Fixes

- Linter was reporting a valid variable as illegal
- Incorrect usage of features
- Move some code to test cfg
- Fixed build in release mode
- Incorrect upper bound for max length of string
- AndThenOkErr should never re-map errors, only okay and incomplete results
- Use syntax error when entire program can't be parsed
- Use Box dyn for to_option and or_default to fix the build on Mac
- Auto-fixed some issues with clippy
- `seq` parsers should convert errors to fatal
- Removed macro_export from visitor macros
- Remove the build warning about the `resolver`

### 🚜 Refactor

- Split and_pc.rs into smaller files
- Removed _pc suffix from modules
- Reworked csv implementation, added loop_while and opt_zip traits
- Removed unused traits
- Various refactorings (#113)
- Do not use expression to parse param_name.rs
- Unify parsing of dim_name and param_name
- Linter improvements
- Replaced methods of TypeQualifier with new traits IntoTypeQualifier and IntoQualified
- Dropped usage of QualifiedName in Name
- Promoted pre_linter.rs to directory
- Using inner mutability in pre_linter
- Adding convertible.rs for converting function/sub parameters in pre-linter
- Removed lifeline from converter.rs objects for simplicity
- Simplified implicits in linter
- Introducing Stateful trait
- Removed SameTypeConverter trait
- Removed SameTypeConverterInContext trait and ConverterImpl
- Reverted Stateful implementation
- Implemented child state for expression conversion
- Using a different child state for ExpressionNode and Expression
- Promoted files to directory modules
- Reworked dim_rules, removed conversion dance from param_type to dim_type
- Removed custom PartialEq implementation of ParamType
- Trying a structure where small types and traits live in modules named types and traits respectively
- Implemented Convertible for ProgramNode and TopLevelToken
- Remove Rc from Context
- Introducing ResolvedParamType
- Dropped position from the name of UserDefinedType
- Simplify pre-linter
- Drop inner mutability for interpreter Input trait
- Removed all usages of Rc and RefCell
- Split collecting phase of LabelLinter into a LabelCollector
- Dropped top level built_ins module
- Using Cargo workspaces
- Decided on public fields for Locatable and VarName, dropped AsRef implementation and accessor methods
- Removed CaseInsensitiveString's Add operations and StringUtils pad_left
- Added rusty_variant crate
- Moved file_constants.rs to rusty_parser
- Introducing CaseInsensitiveStr
- Moved string_utils.rs to interpreter
- Moved to_str_unchecked to interpreter
- Rename Location and Node
- Separate error type per crate
- Moved error_envelope to interpreter
- Moved indexed_map to interpreter
- Refactor Parser trait to generic, so that it can support `dyn`
- Use `dyn` for `OrParser`
- Use new parsers to build Tokenizer
- Use parser combinator in `comment.rs`
- Use parser combinator for parsing FileHandle
- Use parser combinator for parsing the main program
- Use parser combinator in statement_separator
- Use parser combinator in keyword_choice and keyword_map
- Use parser combinator in select_case
- Remove negate parser
- Remove peek parser
- Remove token from unread method as it is no longer used
- Removed ParserOnce trait
- Removed NonOptParser marker trait
- Introduce ParseResult for old parsers instead of standard Result
- Introducing ParseResult::None enum member
- Use ParseResult in AndThen mapping functions
- Use ParseResult in AndThenOkErr mapping functions
- Reduce usages of ParseResult::Incomplete
- Replaced MapIncompleteError with WithExpectedMessage parser
- Removed ParseError::Incomplete
- Removed incomplete errors
- Port  enum member to new 
- Removed  method
- Added combiner function to AddWithoutUndo
- Remove GuardPC
- Remove concrete KeepLeft and KeepRight structs
- Renamed AndThen to FlatMap
- Deleted AllowDefault and AllowNone
- Expose the combiner function in allow_opt
- Expose the combiner function in and
- Remove the PC suffix from parsers
- Divide Parser into smaller traits
- Redesigned pc_ng to use the standard Result type and not return the parsed input in the Err
- Use the new ParseResult everywhere
- Use Box dyn for or and seq
- Moved surround method to And trait
- Moved loop_while to its own trait LoopWhile
- Moved error related parsers to 
- Move logging method to its own trait
- Move allow_none_if method to its own trait
- Moved one_or_more and zero_or_more to Many trait
- Moved  method to its own  trait
- Removed deprecated  parser in favor of 
- Removed deprecated  in favor of 
- Renamed `rusty_parser/types` to `rusty_parser/specific`
- Move `Implicits` and `Names` under a new module
- Added new type `CompactsInfo`
- Added wrapper struct `NamesInner`
- Delegating implementation to new structs in `names` module
- Expose traits of `names` outside module
- Store names of all scopes in `Names` without dropping them
- Expose linter context to interpreter
- Restructure `rusty_linter` modules
- Restructure `rusty_parser` modules
- Flatten public api of `rusty_parser` crate
- Remove complex nested context objects in linter converter
- Simplify converter for statements
- Not allowing 'zero' `Position`
- Use `VariableInfo` from linter `Context` in generator
- Use a struct for FunctionDeclaration and SubDeclaration
- Introducing new `Visitor` pattern in linter
- Simplify `pre_linter` types
- Implement visitor pattern for `constant_map`
- Converted function `validate_string_length` into a trait
- Converted `user_defined_type_rules` to the visitor pattern
- Use new `Assignment` struct in `Statement`
- Adapt `for_next_counter_match_linter` and `print_linter` to the Visitor pattern
- Add a `bi_tuple` macro to reduce some boilerplate code
- Changed `Statement::SubCall` to use a struct instead of two fields
- Changed `Statement::BuiltInSubCall` to use a struct instead of two fields
- Improve encapsulation of `CaseBlock` struct
- Make `bi_tuple` macro more expressive
- Migrated `UserDefinedNamesCollector` to the new Visitor pattern
- Combined all `VarType*` traits into one `VarType` trait
- Made fields of `TypedName` private
- Introducing `ref_to_value_visitor`
- Converted `Name` into a struct with private fields
- Add `AsBareName` and `ToBareName` traits
- Remove `QualifiedName` type
- Renamed `chain` to `then_with`
- Removed `and_without_undo`
- Moved specific types away from `pc` module
- Use small functions to improve `constant_map` readability
- Implement `ConstLookup` trait for `names_inner`
- Removed `Position` from `Token`, made fields private
- Moved the `pc` framework to the `rusty_pc` package

### 🎨 Styling

- Styling changelog according to default options

### ⚙️ Miscellaneous Tasks

- Ensure no code lives in `mod.rs` files
- Applied auto-fixes from cargo clippy
- Implementing cargo clippy suggestions
- Removed unnecessary boxing of `&dyn Printer`
- Implementing cargo clippy suggestions
- Implementing cargo clippy suggestions
- Reduced module visibility
- Extracted rusty_parser crate
- Extracted rusty_linter crate
- Removed funding file
- Update dependencies
- Extract test function in name parser's unit test
- Deprecated some error related methods
- Fix warning about lifeline
- Uniform coding style of parser implementation
- Reverted deprecation of `or_fail` and `no_incomplete`
- Removed method `Variant::is_array`
- Renamed idx variable to index for more clarity
- Removed deprecated method in `MockInterpreterTrait`
- Applied a few clippy suggestions
- Fixed `use_self` clippy rule
- Fixed clippy finding `branches_sharing_code`
- Derive default with annotation in linter converter types
- MainContext of PreLinter does not need to be public
- Marked function as test only
- Renamed `implicits` to `implicit_vars`
- Moved `NameInfo` to its own module
- Improved docs
- Use single field in `Names` for all one-level data
- Removed mut methods of `Position`
- Add comments
- Instantiate `pre_linter` context with `Default`
- Removed `unwrap` custom functions in favor of `From` trait
- Change `ConstantMap` `type` into a `struct` for encapsulation 
- Move `bi_tuple` macro to a new module
- Implement `AsRef` for `TypedName<T>`
- Removed `From<Token>` implementation of `BareName`
- Removed unused code in `type_qualifier`
- Improve `keyword_enum` macro
- Removed `AccumulateParser`
- Removed `AllowNoneIf` parser
- Removed obscure `ExtractExpression` trait
- Code readability improvements to `opt_second_expression`
- Improve naming of `ConstValueResolver` for clarity
- Reformat code with nightly rustfmt
- Upgrade packages edition to 2024
- Upgraded `rusty_parser` to 2024 edition
- Re-arrange packages in `rusty_parser`
- Update `README`
- Split Windows build on a separate GitHub workflow
- Migrating GitHub workflows to `dtolnay/rust-toolchain`
- Unifiy build and build-windows GitHub actions
- Trying a `windows-latest` GitHub runner instead of cross compilation
- Experimenting with sharing artifacts between jobs
- Align build and release GitHub workflows

## [0.8.0] - 2022-09-25

### 🚜 Refactor

- Using tokenizers (#110)

### ⚙️ Miscellaneous Tasks

- Updated copyright year in LICENSE
- Use cliff.toml from instarepo
- *(changelog)* Updated changelog
- *(changelog)* Updated changelog

## [0.7.0] - 2021-12-11

### 🚀 Features

- [**breaking**] Use standard PATH_TRANSLATED variable for Apache cgi-script instead of BLR_PROGRAM
- Introducing git-cliff and changelog

## [0.5.0] - 2020-11-27

### Bugfix

- Arguments appearing out of order due to enumerating over HashMap

### Refactoring

- Changed Name::Qualified to use QualifiedName as it used to in the past
- Removed HasQualifier trait
- Renamed TypeDefinition to ExpressionType

## [0.3.0] - 2020-07-29

### 🚜 Refactor

- Moved numeric casts to separate module

### Bugfix

- Strings cannot be used as an if condition

<!-- generated by git-cliff -->
