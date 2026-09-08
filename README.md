# [Language name]

## Creators

- [Full name] ([github-username])
- [Full name] ([github-username])

## Overview

Grizzly is a small programming language for manipulating tabular data. It is designed for researchers and others who regularly perform repetitive data-cleaning and transformation tasks without wanting to use a general-purpose language or database query language. Tables are first-class values in Grizzly. Users can define tables directly or load them from files, then apply operations for filtering, transforming, and organizing data. Inspired by Python’s pandas and SQL, Grizzly uses a simple, operation-oriented syntax designed specifically for tabular data. Its goal is to make data transformations concise and readable. Grizzly is not intended to replace Python, pandas, or SQL. Instead, it explores how a small domain-specific language can make common tabular-data tasks easier to express.

## Host language and build

- Host language: Rust (Release)
- Version metadata: rust-toolchain.toml
- Build: `./build.sh`

## Running it

| Command                   | What it does                                      |
| ------------------------- | ------------------------------------------------- |
| `./run <file>`            | [Executes a program. Available from Lab 4.]       |
| `./run --tokenize <file>` | [Prints the token stream.]                        |
| `./run --parse <file>`    | [Prints the parsed tree.]                         |
| `./run --eval <file>`     | [Evaluates each expression and prints its value.] |
| `./run`                   | [Starts the REPL.]                                |

Exit codes: 0 [when], 65 [when], 70 [when].

## File extension

Grizzly source files use .grz
Examples include hello.grz

## Lexical structure

### Keywords

| Keyword | Purpose        |
| ------- | -------------- |
| if      | Begins a conditional statement. |
| else    | Begins the alternative branch of a conditional. |
| true    | Boolean literal representing true |
| false   | Boolean literal representing false. |
| null    | Represents absence of a value. |
| and     | Logical AND. |
| or      | Logical OR. |
| not     | Logical negation. |
| table   | Begins a table literal. |
| load    | Loads a table from a file. |
| save    | Saves a table to a file. |
| show    | Displays a value. |
| where   | Filters table rows using a condition. |
| select  | Selects table columns by name. |
| at      | Performs positional table selection. |
| rows    | Specifies rows for positional table selection. |
| columns | Specifies columns for positional table selection. |
| add     | Adds a column. |
| take    | Limits the number of rows returned. |
| sort    | Sorts a table. |
| descending | Sorts values from greatest to least. |
| by      | Specifies the column used for sorting or grouping. |
| group   | Begins a grouping operation. |
| calculate| Defines an aggregation. |

### Operators

| Operator | Category                                             | Operands          | Associativity       | Precedence    |
| -------- | ---------------------------------------------------- | ----------------- | ------------------- | ------------- |
| =     | Assignment | Binary | Left |  |
| *     | Arithmetic | Binary | Left |  |
| /     | Arithmetic | Binary | Left |  |
| +     | Arithmetic | Binary | Left |  |
| -     | Arithmetic | Binary | Left |  |
| not     | Logical | Unary | Left |  |
| and     | Logical | Binary | Left |  |
| or     | Logical | Binary | Left |  |
| ==     | Comparison | Binary | Left |  |
| >     | Comparison | Binary | Left |  |
| >=     | Comparison | Binary | Left |  |
| <     | Comparison | Binary | Left |  |
| <=     | Comparison | Binary | Left |  |

### Literals

| Kind      | Syntax                            | Produces             |
| --------- | --------------------------------- | -------------------- |
| Number  | 42, 3.14                   | Number Value |
| String  | "Hello" (Double quotes always) | String Value |
| Boolean | true, false                     | Boolean Value |
| Null   | Null                        | Null Value |

### Identifiers

- Start characters: ASCII letters (A-Z, a-z) and _
- Continue characters: ASCII letters (A-Z, a-z) and _
- Case-sensitive: Yes
- May not be reserved keywords.

### Comments

- Line comments: //
- Block comments: Not supported
- Nesting: [supported or not]
- [Harness note: comment_prefix in tests/lab*/manifest.json is set to the
  token above.]

## Whitespace and termination

- Whitespace significant: Separates tokens but have no semantic meaning.
- Statement terminator: Newline
- Block delimiters: Indentation
- Grouping delimiters: Parenthesis

## Token output format

```
Token(type=IDENTIFIER, lexeme=sales, literal=null, line=1)
```

[What each field means. Frozen as of Lab 1; changes are recorded in the
changelog.]

## Grammar

```
[Your complete context-free grammar, current as of the latest activity.
Unambiguous, with precedence and associativity encoded in rule structure.]
```

## Parse output format

```
[one line of real --parse output, e.g. (+ 1.0 (* 2.0 3.0))]
```

- Groupings print as: [form]
- Numbers print as: [form]

## Semantics

### Values and types

[What runtime values exist, and how they are represented in the host
language.]

### Value printing

- Numbers: [e.g. 5 rather than 5.0]
- Nil: [spelling]
- Strings: [with or without quotes]

### Truthiness

[The complete rule. Which values are false in a condition; everything else is
true.]

### Operator semantics

- Arithmetic: [accepted operand types]
- `+` on strings: [concatenation, error, or coercion]
- Mixed types: [what happens]
- Comparison: [accepted operand types]
- Equality across types: [false, or an error]
- Division by zero: [value produced, or runtime error]

### Scope and bindings

- Redeclaration in the same scope: [allowed or an error]
- Uninitialized variable holds: [value]
- Shadowing: [behavior]
- Undefined name: [static error with exit 65, or runtime error with exit 70]

### Control flow and functions

- Logical operators return: [booleans, or the operand]
- Dangling else binds to: [which if]
- Closure capture of a loop variable: [per iteration, or shared]
- Function with no return statement produces: [value]
- Arity mismatch: [message and exit code]

## Native functions

| Name   | Arguments         | Returns | Notes     |
| ------ | ----------------- | ------- | --------- |
| [name] | [count and types] | [type]  | [caveats] |

## Errors and diagnostics

Message format:

```
[one real static error]
[one real runtime error]
```

| Failure         | Exit code |
| --------------- | --------- |
| [lexical error] | 65        |
| [syntax error]  | 65        |
| [runtime error] | 70        |

## Testing conventions

| Folder     | Activity  | Mode    | Flag         |
| ---------- | --------- | ------- | ------------ |
| tests/lab1 | Scanner   | sidecar | `--tokenize` |
| tests/lab2 | Parser    | sidecar | `--parse`    |
| tests/lab3 | Evaluator | inline  | `--eval`     |
| tests/lab4 | Context   | inline  | none         |
| tests/lab5 | Functions | inline  | none         |

```
[specific tests]...
```

Run locally with:

```bash
curl -sSL https://raw.githubusercontent.com/WhiteLicorice/cmsc-124-harness/v1.1/run_tests.py -o run_tests.py
./build.sh
python3 run_tests.py tests/lab1
```

## Sample code

```
[a short program]
```

Output:

```
[its output]
```

## Design rationale

[Why the language is the way it is. Cover the choices that surprised you, the
features you cut, and the decisions you reversed. Specific reasons, not
approval of your own work.]

## Known limitations

- [What doesn't work, what is unimplemented, where behavior is worse than you
  would like.]

## Changelog

| Activity | What changed in the language |
| -------- | ---------------------------- |
| Lab 1    | [entry]                      |
