## [unreleased]

### 🐛 Bug Fixes

- Linter was reporting a valid variable as illegal
- Incorrect usage of features
- Move some code to test cfg
- Fixed build in release mode
- Incorrect upper bound for max length of string

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

### 💼 Other

- Changed Name::Qualified to use QualifiedName as it used to in the past
- Removed HasQualifier trait
- Renamed TypeDefinition to ExpressionType
- Arguments appearing out of order due to enumerating over HashMap
## [0.3.0] - 2020-07-29

### 💼 Other

- Strings cannot be used as an if condition

### 🚜 Refactor

- Moved numeric casts to separate module
