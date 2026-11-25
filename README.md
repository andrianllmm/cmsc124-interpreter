<div align="center">
  <h3>Grizzly</h3>
  <img src="assets/logo.svg" alt="Grizzly logo" width="80">
</div>

## Creators

- Andrian Lloyd Maagma (andrianllmm)
- Julian Hanns Medalla (jtmedalla)

## Overview

Grizzly is a small language for transforming tabular data. It's built for data analysts and engineers who'd otherwise reach for a general-purpose language or a SQL dialect to do the same repetitive work. Tables are first-class values, definable inline or loaded from a file. Every transformation is just an ordinary function, chained with pipes rather than nested calls or query clauses. Code reads top to bottom in the same order it runs, so a pipeline looks like the sequence of steps it performs. Writing Grizzly should feel like describing a data pipeline, not programming one.

## Host language and build

- Host language: Rust (Release)
- Version metadata: `rust-toolchain.toml`
- Build: `./build.sh`, which produces `target/release/grizzly`

## Running it

| Command                   | What it does                                    |
| ------------------------- | ----------------------------------------------- |
| `./run <file>`            | Executes a program.                             |
| `./run --tokenize <file>` | Prints the token stream.                        |
| `./run --parse <file>`    | Prints the parsed tree.                         |
| `./run --eval <file>`     | Evaluates each expression and prints its value. |
| `./run`                   | Starts the REPL.                                |
| `./run --tokenize`        | Starts the REPL, printing only tokens.          |
| `./run --parse`           | Starts the REPL, printing only the parsed tree. |

In the REPL, an entry continues onto the next line (with a `...` prompt) while a `(`, `[`, or `{` is unclosed. An empty line gives up on an incomplete entry and reports its syntax error.

`./run` wraps that binary. After `cargo install --path .`, use `grizzly` in place of `./run`.

Exit codes: 0 on success, 65 on a static error (lexical, syntax), 70 on a runtime error (type mismatch, division by zero, undefined name, arity mismatch).

### Jupyter

To run Grizzly in Jupyter notebooks, install `grizzly` on your PATH, then register the kernel once (needs Jupyter installed):

```sh
cargo install --path .
python3 editors/jupyter/grizzly_kernel.py install
```

Pick the "Grizzly" kernel in JupyterLab. Cells share one interpreter and echo each expression's value, like the REPL. The kernel is a [wrapper kernel](https://jupyter-client.readthedocs.io/en/latest/wrapperkernels.html) that runs cells on `grizzly --cells`.

## File extension

`.griz`

Editor support: [VS Code extension](editors/vscode).

## Lexical structure

### Keywords

| Keyword              | Purpose                        |
| -------------------- | ------------------------------ |
| `return`             | Return a value from a function |
| `if` / `else`        | Conditional                    |
| `match` / `case`     | Multi-branch conditional       |
| `for` / `in`         | Loop over a `List`             |
| `while`              | Loop while a condition holds   |
| `and` / `or` / `not` | Boolean operators              |
| `true` / `false`     | Boolean literals               |
| `null`               | The absence of a value         |
| `table`              | Introduces a table literal     |

### Operators

| Operator            | Category   | Operands | Associativity | Precedence   | Meaning                                                 |
| ------------------- | ---------- | -------- | ------------- | ------------ | ------------------------------------------------------- |
| `=`                 | assignment | binary   | right         | 0            | `x = y`                                                 |
| `+=` `-=` `*=` `/=` | assignment | binary   | right         | 0            | `x OP= y` means `x = x OP y`                            |
| `\|>`               | pipe       | binary   | left          | 0            | `x \|> f` means `f(x)`, `x \|> f(a)` means `f(x, a)`    |
| `\|=`               | assignment | binary   | right         | 0            | `x \|= f1 \|> f2` means `x = x \|> f1 \|> f2`           |
| `or`                | logical    | binary   | left          | 1 (loosest)  |                                                         |
| `and`               | logical    | binary   | left          | 2            |                                                         |
| `not`               | logical    | unary    | right         | 3            |                                                         |
| `==` `!=`           | comparison | binary   | none          | 4            |                                                         |
| `<` `<=` `>` `>=`   | comparison | binary   | none          | 4            |                                                         |
| `+` `-`             | arithmetic | binary   | left          | 5            |                                                         |
| `++`                | string     | binary   | left          | 5            | concatenation                                           |
| `*` `/` `%`         | arithmetic | binary   | left          | 6            |                                                         |
| `-` (unary)         | arithmetic | unary    | right         | 7            |                                                         |
| `^`                 | arithmetic | binary   | right         | 8 (tightest) | exponent                                                |
| `->`                | other      | n/a      | n/a           | n/a          | lambda arrow, introduces a lambda's body                |
| `=>`                | other      | n/a      | n/a           | n/a          | match arrow, introduces a `case`/`else` clause's result |

- Comparisons don't chain, so `1 < 2 < 3` and `a == b == c` are syntax errors. Write `1 < 2 and 2 < 3`, or group explicitly, e.g. `(a == b) == c`.

### Literals

| Kind                | Syntax                                                                                               | Produces  |
| ------------------- | ---------------------------------------------------------------------------------------------------- | --------- |
| Integer             | `42`                                                                                                 | `Int`     |
| Float               | `3.14`                                                                                               | `Float`   |
| String              | `"hello"`, escapes: `\n \t \\ \" \{`                                                                 | `Str`     |
| Interpolated string | `"Hi {name}, you earn {salary}"`                                                                     | `Str`     |
| Boolean             | `true`, `false`                                                                                      | `Bool`    |
| Null                | `null`                                                                                               | `Null`    |
| List                | `[1, 2, 3]`                                                                                          | `List(T)` |
| Table               | `table [ a \| b, 1 \| 2, 3 \| 4 ]` (header row, then rows; cells `\|`-separated, rows `,`-separated) | `Table`   |

- Strings cannot span lines. A newline before the closing `"` is an unterminated string.
- Since newlines carry no meaning, table rows are separated by `,`, like list elements. Lay a literal out one row per line, with the `,` at the end of each row but the last.
- A `.` in a number needs digits on both sides, so `1.`, `.5`, `1.2.3`, and `3.foo` are errors. A stray `.` is reported as a typo.

### Identifiers

- Start characters: ASCII letters (`a-z`, `A-Z`), `_`
- Continue characters: ASCII letters, ASCII digits (`0-9`), `_`
- Case-sensitive: yes
- No length limit

Examples:

```
sales
customer_name
_total
amount2
```

### Comments

- Line comments: `#`, runs to end of line
- Block comments: not supported
- Nesting: n/a
- `comment_prefix` in `tests/lab*/manifest.json` is set to `#`

### String interpolation

- `{expr}` inside a string literal embeds an expression's text value. `"Hi {name}"` desugars at parse time to `"Hi " ++ to_string(name)`, no new evaluator behavior needed.

## Whitespace and termination

- Whitespace significant: no (except inside string literals)
- Statement terminator: semicolon `;`.
  Newlines carry no grammar meaning, so a statement can freely span multiple lines.
- Block delimiters: braces `{ }`.
  Used for `if`/`while`/`for` bodies, function/lambda block bodies, and `match`'s clause list.
- Grouping delimiters: parentheses `( )`

## Token output format

```
Token(type=, lexeme=, literal=, line=)
```

Fields: token type, lexeme, literal value (or empty), line number.

String literals print escaped.

## Grammar

```
program     → statement* EOF

statement   → exprStmt
            | ifStmt
            | whileStmt
            | forStmt
            | returnStmt
            | block

exprStmt    → expression ";"

ifStmt      → "if" expression block ( "else" ( ifStmt | block ) )?

whileStmt   → "while" expression block

forStmt     → "for" IDENTIFIER "in" expression block

returnStmt  → "return" expression? ";"

block       → "{" statement* "}"

expression  → assignment

assignment  → IDENTIFIER ( "=" | "+=" | "-=" | "*=" | "/=" | "|=" ) assignment
            | lambda
            | pipe

lambda      → ( IDENTIFIER | "(" parameters? ")" ) "->" ( expression | block )

parameters  → IDENTIFIER ( "," IDENTIFIER )*

pipe        → logicOr ( "|>" logicOr )*

logicOr     → logicAnd ( "or" logicAnd )*

logicAnd    → logicNot ( "and" logicNot )*

logicNot    → "not" logicNot
            | comparison

comparison  → term ( ( "==" | "!=" | "<" | "<=" | ">" | ">=" ) term )?

term        → factor ( ( "+" | "-" | "++" ) factor )*

factor      → unary ( ( "*" | "/" | "%" ) unary )*

unary       → "-" unary
            | exponent

exponent    → call ( "^" unary )?

call        → primary ( "(" arguments? ")" )*

arguments   → argument ( "," argument )*

argument    → ( IDENTIFIER ":" )? expression

exprList    → expression ( "," expression )*

primary     → INTEGER | FLOAT | STRING | "true" | "false" | "null"
            | IDENTIFIER
            | "(" expression ")"
            | "[" exprList? "]"
            | table

table       → "table" "[" header ( "," row )* "]"

header      → IDENTIFIER ( "|" IDENTIFIER )*

row         → expression ( "|" expression )*
```

- Assignment is an expression, so it can appear anywhere an expression can, e.g. `print(x = 1);`. It produces the assigned value. Its target must be an identifier
- A binding created by assignment belongs to the innermost enclosing block, or to the top level outside any block
- A lambda sits at assignment level, so it can be assigned or passed as an argument directly, but needs parentheses as an operand, e.g. `(x -> x) |> f`. Its expression body extends as far right as possible: `x -> x + 1` is `(lambda (x) (+ x 1))`
- Blocks nest to any depth and can appear anywhere a statement can
- `if`, `while`, and `for` take no parentheses, and their bodies must be blocks: `if x > 1 { ... } else if y { ... } else { ... }`. `else if` needs no extra rule, since `else` may be followed by another `ifStmt`
- A named argument is an identifier, `:`, then an expression: `add_col(t, bonus: salary * 0.1)`. The parser tells it apart by looking one token past the identifier, since an expression never starts with `IDENTIFIER ":"`. Named arguments come after all positional ones
- `|` (`BAR`) isn't an operator, so a table cell's expression ends at the next `|`, `,`, or `]`. `|>` and `|=` are separate tokens, so a cell can still hold a pipe

## Parse output format

```
(+ 2 (* 3 4))
(* (group (+ 2 3)) 4)
(- (- 1 2) 3)
```

- Groupings print as: `(group <expr>)`
- Lists print as: `(list 1 2 3)`, or `(list)` when empty
- Named arguments print as: `(named bonus (* salary 0.1))`, e.g. `(call add_col t (named bonus (* salary 0.1)))`
- Tables print as: `(table (name dept) ("Ana" "eng") ("Bo" "ops"))`, the header first, then one parenthesized list per row
- Blocks print as: `(block <stmt>...)`, or `(block)` when empty
- Lambdas print as: `(lambda (a b) (+ a b))`, or `(lambda () (block ...))` with a block body
- `if` prints as: `(if <cond> <then>)`, or `(if <cond> <then> <else>)`. An `else if` nests as the else branch: `(if a (block) (if b (block)))`
- `while` prints as: `(while <cond> <block>)`. Nesting appears inside `<block>`: `(while cond (block (while cond (block)))`
- `for` prints as: `(for x <list> <block>)`, e.g. `(for x (list 1 2) (block (call print x)))`
- Assignments print as: `(= x 1)`, compound ones with their operator, `(+= x 1)`. Chains nest to the right: `a = b = 2` is `(= a (= b 2))`
- Integers print as written: `2`
- Floats always keep a decimal point: `3.0`, never `3`
- Strings print quoted and escaped: `"hi\n"`
- Booleans and null print as their keyword: `true`, `false`, `null`

## Parser quirks

- `;` ends each expression. Newlines don't, so `1 +\n2;` is one expression.
- An empty or comment-only file is valid and prints nothing.
- `-1` is a unary minus on `1`, not a negative literal: `(- 1)`.
- `^` is right-associative and binds tighter than unary `-`: `-2 ^ 2` is `(- (^ 2 2))`.
- `not` binds looser than comparisons: `not 1 < 2` is `(not (< 1 2))`.
- `|>` binds looser than `or`: `a or b |> f` is `(|> (or a b) f)`.
- The target of `=` or `OP=` must be a bare identifier. Anything else, e.g. `1 = 2;` or `(x) = 2;`, is `Invalid assignment target`, a syntax error (exit 65).
- After a syntax error, parsing skips to the next statement boundary (after a `;`, or before a `}` or a statement keyword), reports every error, and runs nothing.
- A missing `{` after an `if` condition or `else` is `Missing '{'`, a syntax error (exit 65).
- A trailing comma in a call or list, e.g. `[1, ]`, is `Missing expression`, a syntax error (exit 65).
- A `for` needs a name and then `in`, so `for 1 in xs { }` is `Missing loop variable` and `for x [1] { }` is `Missing 'in'`, syntax errors (exit 65).
- A lambda parameter must be an identifier, so `(1) -> 1;` and `(a, ) -> 1;` are `Missing parameter name`, a syntax error (exit 65).
- A lexical error stops the run before parsing, so syntax errors in that file go unreported.
- A positional argument after a named one, e.g. `f(a: 1, 2)`, is `Positional argument after named one`, a syntax error (exit 65).
- A table header cell must be an identifier, so `table [ "a" | b ]` and `table [ ]` are `Missing column name`, a syntax error (exit 65). A repeated header name is `Duplicate column 'a'`, a syntax error (exit 65).
- Every row must have as many cells as the header, checked while parsing: `table [ a | b, 1 ]` is `Expected 2 cells but got 1.`, worded like an arity mismatch, a syntax error (exit 65).
- A trailing `,` after the last row is `Missing expression`, the same as in a list.

## Semantics

### Values and types

| Type       | Host representation             |
| ---------- | ------------------------------- |
| `Int`      | `Value::Int(i64)`               |
| `Float`    | `Value::Float(f64)`             |
| `Str`      | `Value::Str(String)`            |
| `Bool`     | `Value::Bool(bool)`             |
| `Null`     | `Value::Null`                   |
| native     | `Value::Native(Native)`         |
| `Function` | `Value::Function(Rc<Function>)` |
| `List`     | `Value::List(Rc<Vec<Value>>)`   |
| `Table`    | `Value::Table(Rc<Table>)`       |

Values are a Rust `enum` (a tagged union). One type holds any Grizzly value, matching on the variant tells the evaluator what it has, and the compiler flags a `match` that forgets a type.

A `List` is immutable, so it's behind an `Rc` and copies share one `Vec`. `append` returns a new list.

A `Table` is ordered, named columns of equal length, plus the group columns set by `group_by` (none otherwise). Each column is an `Rc<Vec<Value>>`, so verbs that keep a column (`select`, `rename`, `add_col`) share it instead of copying it. Like a `List`, a `Table` is never changed in place.

A `Function` is a lambda turned into a value: its parameters, its body, and the environment it was created in. It's behind an `Rc`, so copying the value shares the same function. Natives and functions both implement one `Callable` trait (`arity` and `call`), so a call checks callability and arity in one place for both.

### Value printing

| Type       | Prints as                              | Example              |
| ---------- | -------------------------------------- | -------------------- |
| `Int`      | as written                             | `5`                  |
| `Float`    | always with a decimal point            | `5.0`, never `5`     |
|            | exponent form when very large or small | `1e20`, `1e-7`       |
|            | non-finite values by name              | `inf`, `-inf`, `NaN` |
| `Str`      | without quotes                         | `"hi"` prints `hi`   |
| `Bool`     | its keyword                            | `true`, `false`      |
| `Null`     | its keyword                            | `null`               |
| native     | its name                               | `<native fn print>`  |
| `Function` | a placeholder                          | `<fn>`               |
| `List`     | elements by these rules, in brackets   | `[1, 2.0, a]`        |
| `Table`    | an aligned grid, see below             |                      |

A `Table` prints as a header line, a rule line, then one line per row:

```
name | dept  | salary
-----+-------+-------
Ana  | eng   | 60000
Cy   | sales | 70000
```

- Cells print by the rules above, so strings are unquoted and `null` prints `null`
- Each column is left-aligned and padded to its widest cell or name, counted in characters. Cells are joined with ` | `, and the rule line uses `-` joined with `-+-`
- The last column isn't padded, so no line has trailing spaces
- A table with no rows prints only the header and rule lines
- A grouped table prints the same as its ungrouped rows; grouping isn't shown
- `to_string` produces the same text, lines joined by `\n`, with no trailing newline

### Truthiness

| Type                 | Falsey        | Truthy                        |
| -------------------- | ------------- | ----------------------------- |
| `Int`                | `0`           | any other `Int`               |
| `Float`              | `0.0`, `-0.0` | any other `Float`, even `NaN` |
| `Str`                | `""`          | any non-empty `Str`           |
| `Bool`               | `false`       | `true`                        |
| `Null`               | `null`        | never                         |
| `List`               | `[]`          | any non-empty `List`          |
| `Table`              | no rows       | one or more rows              |
| native, `Function`   | never         | always                        |

### Operator semantics

Operands are evaluated left to right, and both are evaluated before the operator applies, except for `and` and `or`, which short-circuit.

#### Arithmetic: `+ - * / % ^`, unary `-`

- Operands: `Int` and `Float` only. Anything else is a runtime error
- Mixed types: an `Int` mixed with a `Float` is promoted, so `1 + 2.0` is `3.0`
- `Int` division: `/` truncates toward zero, so `1 / 2` is `0` and `-7 / 2` is `-3`. Write `1.0 / 2` for `0.5`
- `%`: takes the sign of the left operand, so `-7 % 2` is `-1` and `7 % -2` is `1`
- Division by zero: `Int` `/` or `%` by zero is a runtime error, since `Int` has no infinity to return. `Float` follows IEEE 754 and never errors: `1.0 / 0` is `inf`, `-1.0 / 0` is `-inf`, `0.0 / 0` is `NaN`
- Overflow: `Int` arithmetic is checked, and overflow is a runtime error
- Negative exponent: `2 ^ -1` is a runtime error, since an `Int` can't hold a fraction

#### Concatenation: `++`

- Operands: `Str` only, so `"a" ++ 1` is a runtime error. Convert first with `to_string`
- `+` doesn't concatenate: `"a" + "b"` is a runtime error

#### Comparison: `< <= > >=`

- Operands: `Int` and `Float`, mixed included. Strings don't compare, so `"a" < "b"` is a runtime error
- Returns a `Bool`. Any comparison with `NaN` is `false`

#### Equality: `== !=`

- Operands: any types, never an error
- `Int` and `Float` compare by value, so `1 == 1.0` is `true`
- Other values are equal only to the same type with the same value, e.g. `null == null`, `print == print`
- Lists compare element by element by these rules, so `[1, [2]] == [1.0, [2]]` is `true`
- Tables are equal when their column names match in order and every cell is equal by these rules. Grouping is ignored
- Functions are equal only to themselves: after `g = f;`, `g == f` is `true`, but two separately written `x -> x` are unequal
- Mixed types are unequal, so `1 == "1"` is `false`

#### Logical: `and or not`

- Operands: any types, tested with the truthiness rule
- Returns a `Bool`, so `1 and "a"` is `true`
- Short-circuit: the right side is skipped when `and`'s left is falsey or `or`'s left is truthy, so `false and 1 / 0` is `false`

### Scope and bindings

- Declaration: there's no keyword. `x = value` updates the nearest binding named `x`, searching outward from the current block. If none exists, it creates `x` in the current block
- Scope rule: lexical. Lookup starts in the block where the name appears and walks out through the blocks that enclose it in the source. The innermost binding wins
- Shadowing: not possible. Assigning an outer name inside a block updates the outer binding. A name first assigned inside a block is local to that block and is gone once the block ends
- Redeclaration in the same scope: not applicable, since assigning again updates the existing binding
- Uninitialized variable holds: nothing, since every binding is created with a value
- Mutability: every binding is mutable, and both its value and its type can change, e.g. `x = 1; x = "a";`
- Binding time: a name is bound to storage and a value when its first assignment runs. Types are bound at runtime
- Storage lifetime: every binding is implicit heap-dynamic, since its storage is allocated by its first assignment rather than before the program runs. A top-level binding then lasts the rest of the program, or the rest of the REPL session. A block binding stops being visible when the block exits, and its storage is reclaimed then unless a closure created in the block captured it
- Leaving a block: the enclosing scope is restored however the block exits, including on a runtime error, so the REPL keeps working in the outer scope. A block is a statement and produces no value, so the REPL prints nothing for it
- Undefined name: reading a name that has no binding is a runtime error (exit 70), `Undefined variable 'x'`. It's checked at runtime so recursive and mutually recursive functions (Lab 5) work without a separate resolver pass
- A misspelled name on the left of `=` creates a new binding instead of erroring, which is the cost of having no declaration keyword
- Compound assignment: `x OP= v` means `x = x OP v` for `+= -= *= /= |=`, with the same errors as the plain operator. It reads `x` first, so it needs an existing binding and never creates one: `y += 1;` with no `y` is `Undefined variable 'y'` (exit 70). It produces the new value
- Natives: bound as ordinary globals, so assignment can rebind them. After `print = 1;`, `print(2);` is `Can only call functions` (exit 70)

### Control flow and functions

- Conditions: tested with the truthiness rule. Only the chosen branch runs, and each branch is a block with its own scope
- `if` without `else`: does nothing when the condition is falsey
- Dangling else binds to: can't arise, since every branch is a braced block, so each `else` belongs to the `if` whose block just closed
- `while`: tests the condition before every iteration, so a falsey condition runs the body zero times. Each iteration runs the body as a fresh block
- `for x in list`: runs the body once per element, in order. Each iteration runs the body in a fresh block with `x` defined there, so `x` never updates an outer `x` and is gone after the loop, and a lambda created in the body keeps its own `x`. An empty list runs the body zero times
- Looping over a non-list: `Can only loop over a list`, a runtime error (exit 70)
- Functions: created only as lambdas with `->` and named by assigning them, e.g. `tax = (amount, rate) -> amount * rate;`. There's no separate declaration form. A function is a value like any other, so it can be passed, returned, and stored
- Bodies: `x -> expr` produces `expr`. A block body `x -> { ... }` produces its value with `return`, which exits the function from inside any loop or `if`
- Function with no return statement produces: `null`
- Calls: arguments are evaluated left to right at the call site, then bound as parameters in a fresh environment for that call
- Scope of a call: lexical. The call's environment encloses the environment where the lambda was created, not the caller's
- Closures: a lambda captures the environment it was created in and keeps it alive after the creating call returns, so `make_adder = n -> (x -> x + n);` works. Each call of `make_adder` captures its own `n`
- Closure capture of a loop variable: per iteration for names first assigned in the body, since each iteration is a fresh block. Names from outside the loop are shared
- Recursion: a lambda finds its own name at call time through ordinary lookup, so recursion and mutual recursion need nothing special
- Call depth: at most 5000 calls may be active at once. One more is `Stack overflow`, a runtime error (exit 70). The depth unwinds with the error, so the REPL keeps working
- Arity mismatch: `Expected 2 arguments but got 1.`, a runtime error (exit 70). Natives and lambdas are checked the same way
- Calling a non-function: `Can only call functions`, a runtime error (exit 70)
- Deep recursion: calls past a fixed depth limit raise `Stack overflow`, a runtime error (exit 70), instead of crashing the host

Planned for Lab 5, not built yet: the `if ... then ... else` expression form, `match`/`case`, and string interpolation.

### Tables

Planned, not built yet. Table verbs can't use plain call evaluation: in `filter(t, salary > 50000)`, `salary` is a column, not a variable, and the condition must run once per row.

#### Parameter kinds

Each native declares a kind for each parameter. The call path reads the kinds and handles every verb the same way, so no verb parses its own arguments.

| Kind         | The native receives                          | Example                    |
| ------------ | -------------------------------------------- | -------------------------- |
| value        | the evaluated argument, as for any call      | `t` in `filter(t, ...)`    |
| row          | the unevaluated expression, run once per row | `salary > 50000`           |
| column       | a column name, written as a bare identifier  | `salary` in `sort(t, ...)` |
| option       | one of a fixed set of bare words             | `desc`, `left`             |
| named row    | any number of `name: row` pairs              | `bonus: salary * 0.1`      |
| named column | any number of `name: column` pairs           | `dept: department`         |
| named        | fixed names, each taking a column or option  | `on: dept`, `how: left`    |

A parameter can be variadic (`select(t, cols...)`) or optional (`sort`'s direction). Lambdas have only value parameters. The aggregates are the one case whose kind depends on where they're called, see Group context.

#### Row context

- A row expression runs once per row, in a fresh environment whose parent is the call site's environment. Each column is bound there as a scalar holding that row's cell
- Columns win over outer names, so in `filter(t, dept == "eng")` `dept` is the column even if a variable `dept` exists. This is the one place a name shadows an outer binding. Other names resolve as usual, so `for d in depts { t |> filter(dept == d); }` reads the loop's `d`
- A name that is neither a column nor a variable is `Undefined variable 'x'` (exit 70)
- Rows run top to bottom, and the expression is evaluated fully for each row. With no rows it never runs
- Assignment follows the usual rules: it updates an outer binding if one exists, else it creates one in the row's environment, which is discarded after the row. Assigning a column name changes only that row's binding, never the table
- Piping keeps row context: `t |> filter(salary > 50000)` is `filter(t, salary > 50000)` with the condition still unevaluated

#### Column names

- A column or option parameter takes a bare identifier, used as a name and never looked up, e.g. `select(t, name, dept)`, `sort(t, salary, desc)`
- Anything else there, e.g. `select(t, "name")` or `select(t, x + 1)`, is `Expected a column name`, a runtime error (exit 70)
- A name the table doesn't have is `Unknown column 'x'` (exit 70)
- A bad option is `Expected one of: asc, desc` (exit 70)
- `asc`, `desc`, `inner`, and `left` aren't keywords, so they stay usable as variable names
- Dynamic column names (a name held in a variable) aren't supported

#### Named arguments

- Only natives accept them. A named argument to a lambda is `Named arguments only work with built-in functions` (exit 70)
- In a named row parameter (`add_col`, `update_col`, `summarize`) the name is the column to create or replace and the expression is a row expression. `rename(t, dept: department)` takes column names on both sides
- A name the native doesn't take, e.g. `join(t, u, key: id)` or `print(x: 1)`, is `Unknown argument 'key'`, and a missing required one is `Missing argument 'on'` (exit 70)
- A name given twice is `Duplicate argument 'x'` (exit 70)
- Arity counts positional arguments only. A variadic native's mismatch reads `Expected at least 2 arguments but got 1.`

#### Group context

- `group_by(t, cols...)` returns the same rows with those columns marked as group columns. Only `summarize` uses the mark; every other verb returns an ungrouped table
- `summarize(t, name: agg...)` produces one row per group, in order of each group's first row: the group columns, then the named columns in the order written. On an ungrouped table, the whole table is one group, so the result has one row
- A `summarize` expression runs once per group, with the group columns bound. Inside it, an aggregate call runs its argument once per row of the group and aggregates the results, so `sum(if tier == "vip" then revenue else 0)` works and `sum(salary) / count(salary)` combines two aggregates
- Group context covers the `summarize` argument itself, not the body of a lambda it calls. Inside a lambda, and anywhere outside `summarize`, an aggregate takes a `List`: `mean(pull(t, salary))`. So `filter(t, salary > mean(salary))` is `Argument must be a list`; compute `avg = mean(pull(t, salary));` first and filter on `avg`
- A non-group column outside an aggregate, e.g. `summarize(g, x: salary)`, is `Column 'salary' needs an aggregate`, and an aggregate inside an aggregate is `Aggregates can't nest` (exit 70)
- Null handling of the aggregates is in #310

#### Cells and columns

- A cell holds a scalar: `Int`, `Float`, `Str`, `Bool`, or `Null`. Building a table with any other cell, from a literal, `add_col`, or `update_col`, is `Cell must be a scalar, got a list` (exit 70)
- A column's type isn't enforced, so `if x then 0 else 1.5` can mix `Int` and `Float` in one column
- A literal's ragged row or repeated header name is a syntax error, see Parser quirks. The same problems in a CSV are runtime errors (exit 70), naming the line: `Line 3 has 2 cells but header has 3`
- Making a column that exists, with `add_col`, `rename`, or `join`, is `Column 'x' already exists`, and `update_col` on a missing one is `Unknown column 'x'` (exit 70)
- A literal's cells are evaluated left to right, top to bottom, when the literal is evaluated
- `length(t)` is the row count

#### Joins

- `join(t, other, on: col, how: kind)` matches rows where `col` is equal by the equality rules. `on` takes one column, with the same name in both tables; rename first if they differ. `how` is `inner` (the default, matched rows only) or `left` (every row of `t`, with `null` for `other`'s columns when nothing matches)
- Each match produces one row, so a key that appears twice in `other` repeats the row of `t`. Rows keep `t`'s order, then `other`'s order within a key
- The result has `t`'s columns, then `other`'s columns except `col`. Any other shared name is `Column 'x' already exists`; rename one side first
- `col` must exist in both tables, else `Unknown column 'x'`. A `null` key never matches, as in SQL

#### CSV

- Paths are relative to the directory of the running script, so a test can read `data/x.csv` next to itself. The REPL and Jupyter resolve against the working directory. An absolute path is used as is
- The first line is the header, and each name must be an identifier, else `Invalid column name 'first name'` (exit 70). An empty file is `Missing header` (exit 70)
- An unquoted empty cell is `null`, and a quoted empty cell `""` is the empty `Str`
- Types are inferred per column, ignoring `null` cells: `Int` if every cell is an integer (an optional `-`, then digits, fitting an `Int`), else `Float` if every cell is a number (the same, with digits on both sides of any `.`), else `Bool` if every cell is `true` or `false`, else `Str` for the whole column. So an ID column holding `105` and `A12` stays text
- Parsing is hand-rolled, keeping the crate dependency-free: `,` separators, `"`-quoted cells that may hold `,`, `""`, and newlines, and `\n` or `\r\n` line ends. A trailing newline doesn't add a row
- `write_csv(t, path)` writes the header and rows, cells by the value printing rules, `null` as an empty cell, and quotes a cell only when it holds `,`, `"`, a newline, or is the empty `Str`. It overwrites the file
- `write_json(t, path)` writes an array with one object per row, keys in column order, and overwrites the file. JSON has no `inf` or `NaN`, so those `Float`s are written as `null`
- A file that can't be read or written is `Can't read 'x.csv'` or `Can't write 'x.csv'` (exit 70)

#### Verbs

No native changes its arguments; each verb returns a new table. In the signatures, `t` is a table, `col` a column name, `expr` a row expression, and `...` means one or more (zero or more for `distinct`).

**Rows**

| Verb       | Signature                   | Result                                                     |
| ---------- | --------------------------- | ---------------------------------------------------------- |
| `filter`   | `filter(t, expr)`           | Rows where `expr` is truthy                                |
| `sort`     | `sort(t, col, asc \| desc)` | Stable; the direction is optional and defaults to `asc`    |
| `distinct` | `distinct(t, col...)`       | First row of each unique combination; no columns means all |
| `head`     | `head(t, n)`                | First `n` rows, or all of them if `n` is past the end      |
| `tail`     | `tail(t, n)`                | Last `n` rows                                              |
| `slice`    | `slice(t, from, to)`        | Rows `from` up to but not including `to`, counting from 0  |

A negative count or index is a runtime error (exit 70). `slice` clamps both ends to the row count. `sort` orders by the comparison rules and puts `null` cells last in either direction; a column mixing numbers and strings is a runtime error (exit 70). Comparing strings depends on #309.

**Columns**

| Verb         | Signature                        | Result                                    |
| ------------ | -------------------------------- | ----------------------------------------- |
| `select`     | `select(t, col...)`              | Those columns, in the order given         |
| `drop`       | `drop(t, col...)`                | Every other column                        |
| `add_col`    | `add_col(t, name: expr, ...)`    | New columns at the end; each must be new  |
| `update_col` | `update_col(t, name: expr, ...)` | Replaces values in place; each must exist |
| `rename`     | `rename(t, old: new, ...)`       | Columns keep their position               |

**Grouping and combining**

| Verb        | Signature                            | Result                                            |
| ----------- | ------------------------------------ | ------------------------------------------------- |
| `group_by`  | `group_by(t, col...)`                | Marks group columns, see Group context            |
| `summarize` | `summarize(t, name: expr, ...)`      | One row per group, see Group context              |
| `join`      | `join(t, other, on: col, how: kind)` | See Joins                                         |
| `concat`    | `concat(t...)`                       | Rows stacked; same column names in the same order |

**Access and files**

| Native       | Signature             | Result                                                   |
| ------------ | --------------------- | -------------------------------------------------------- |
| `pull`       | `pull(t, col)`        | The column as a `List`                                   |
| `first_row`  | `first_row(t)`        | A one-row table; an empty table gives an empty table     |
| `get`        | `get(t, col)`         | The cell of a one-row table; other row counts are errors |
| `read_csv`   | `read_csv(path)`      | A table, see CSV                                         |
| `write_csv`  | `write_csv(t, path)`  | `null`, see CSV                                          |
| `write_json` | `write_json(t, path)` | `null`, see CSV                                          |

**Aggregates and values**

| Native      | Signature                | Result                                                                  |
| ----------- | ------------------------ | ----------------------------------------------------------------------- |
| `sum`       | `sum(expr)`              | Total of the numbers; `0` when there are none                           |
| `mean`      | `mean(expr)`             | Average as a `Float`; `null` when there are none                        |
| `median`    | `median(expr)`           | Middle number; the mean of the two middle values when the count is even |
| `min`/`max` | `min(expr)`, `max(expr)` | Smallest or largest, by the comparison rules                            |
| `count`     | `count(expr)`            | Number of values                                                        |
| `fill_null` | `fill_null(x, default)`  | `default` when `x` is `null`, else `x`; works on any value              |

Aggregates take a row expression inside `summarize` and a `List` elsewhere, see Group context.

#### Wrapping a verb

A lambda can't forward a row expression. In `my_filter = (t, cond) -> filter(t, cond);`, the call `my_filter(t, salary > 50000)` evaluates `salary > 50000` at the call site, where `salary` is undefined, since lambdas have only value parameters. Wrappers instead capture what varies as values:

```griz
earns_over = (t, amount) -> filter(t, salary > amount);
```

## Native functions

| Name        | Arguments          | Returns | Notes                                                                |
| ----------- | ------------------ | ------- | -------------------------------------------------------------------- |
| `print`     | 1, any             | `Null`  | Writes the value and a newline to stdout                             |
| `to_string` | 1, any             | `Str`   | Converts a value to text using the value printing rules              |
| `length`    | 1, `Str` or `List` | `Int`   | Characters in a `Str` (not bytes), or elements in a `List`           |
| `to_int`    | 1, `Str`           | `Int`   | Parses an integer; unparseable input is `Can't convert 'abc' to Int` |
| `to_float`  | 1, `Str`           | `Float` | Parses a float; unparseable input is `Can't convert 'abc' to Float`  |
| `append`    | 2, `List` and any  | `List`  | A new list with the value at the end; the original is unchanged      |

A wrong argument type is `Argument must be` followed by what the native takes, e.g. `length(1)` is `Argument must be a string or list`, and `append(1, 2)` is `Argument must be a list`. Both errors are runtime errors (exit 70).

Output: in file mode, only `print` writes to stdout, and expression statements discard their values. The REPL also prints the value of each bare expression, except a call that returns `null`, so `print(5);` prints `5` once instead of `5` then `null`. `--eval` follows the same rule.

## Errors and diagnostics

Diagnostics are printed to stderr as `Error at line N: <message>`.

```
Error at line 1: Unexpected character '@'
Error at line 1: Unterminated string
Error at line 1: Invalid escape sequence '\q'
Error at line 1: Invalid number
Error at line 1: Missing '='
```

Syntax errors also name the token where parsing failed, or `end` at end of file.

```
Error at line 1: Missing expression at ';'
Error at line 1: Missing ')' at ';'
Error at line 2: Missing '}' at end
Error at line 1: Missing ';' at end
```

| Failure         | Exit code |
| --------------- | --------- |
| Bad usage       | 64        |
| Lexical error   | 65        |
| Syntax error    | 65        |
| Unreadable file | 66        |
| Runtime error   | 70        |

## Testing conventions

| Folder     | Activity  | Mode    | Flag         |
| ---------- | --------- | ------- | ------------ |
| tests/lab1 | Scanner   | sidecar | `--tokenize` |
| tests/lab2 | Parser    | sidecar | `--parse`    |
| tests/lab3 | Evaluator | inline  | `--eval`     |
| tests/lab4 | Context   | inline  | none         |
| tests/lab5 | Functions | inline  | none         |

Each folder has a `manifest.json` and one `.griz` file per test case, mostly grouped into subfolders by feature (e.g. `tests/lab2/groupings`, `tests/lab3/arithmetic`). Inline folders annotate with `# expect:`, `# expect runtime error:`, and `# expect error:`. Error cases are one per file, prefixed `invalid-` (e.g. `tests/lab3/arithmetic/invalid-divide-by-zero.griz`).

Run locally with:

```bash
curl -sSL https://raw.githubusercontent.com/WhiteLicorice/cmsc-124-harness/v1.1/run_tests.py -o run_tests.py
./build.sh
python3 run_tests.py tests/lab1
```

## Sample code

### Variables, arithmetic, strings, booleans

```griz
age = 25;
name = "Ana";
rate = 0.2;

total = 10 + 5 * 2;
avg = total / 3;
greeting = "Hi, " ++ name ++ "!";
can_vote = age >= 18 and name != "";
```

### Control flow

```griz
if age >= 65 {
  status = "senior";
} else if age >= 18 {
  status = "adult";
} else {
  status = "minor";
}

i = 0;
while i < 5 {
  print(i);
  i = i + 1;
}

for n in [1, 2, 3, 4, 5] {
  print(n * n);
}

band = match age {
  case > 65 => "senior"
  case > 18 => "adult"
  else => "minor"
};
```

### Functions, closures, wrapping a verb

```griz
tax = (amount, rate) -> amount * rate;

make_adder = n -> (x -> x + n);
add5 = make_adder(5);
add5(10);

earns_over = (t, amount) -> filter(t, salary > amount);
```

### Creating a table

```griz
small = table [
  name  | dept    | salary,
  "Ana" | "eng"   | 60000,
  "Bo"  | "eng"   | 45000,
  "Cy"  | "sales" | 70000
];

employees = read_csv("employees.csv");
```

### Filtering and selecting

```griz
employees |> filter(salary > 50000);
employees |> filter(salary > 50000 and dept == "eng");

employees |> select(name, dept, salary);
employees |> drop(ssn, internal_notes);

employees |= select(name, age) |> filter(age > 18);
```

### Adding and computing columns

```griz
employees |> add_col(bonus: salary * 0.10);

employees |> add_col(
  band: if salary > 100000 then "exec" else "standard"
);

employees |> add_col(
  band: match salary {
    case > 150000 => "exec"
    case > 100000 => "senior"
    else => "standard"
  }
);
```

### Grouping and aggregating

```griz
employees
  |> group_by(dept)
  |> summarize(
       total: sum(salary),
       average: mean(salary),
       headcount: count(salary)
     );

employees
  |> group_by(dept)
  |> summarize(
       total: sum(salary),
       exec_total: sum(if salary > 100000 then salary else 0)
     );
```

### Slicing

```griz
employees |> slice(10, 20);
employees |> head(5);
employees |> tail(5);
```

### Sorting, distinct

```griz
employees |> sort(salary, desc);
employees |> distinct(dept);
```

### Joining and combining tables

```griz
merged = employees |> join(departments, on: dept, how: left);
combined = concat(q1_sales, q2_sales, q3_sales);
renamed = employees |> rename(dept: department, salary: pay);
```

### Column and row access

```griz
salaries = employees |> pull(salary);
avg = mean(salaries);

first = employees |> first_row();

first_salary = get(first, salary);
```

### Nulls

```griz
employees |= update_col(quantity: fill_null(quantity, 0));
```

### String interpolation

```griz
name = "Ana";
salary = 60000;

"Hi {name}, you earn {salary}";
```

### Writing output

```griz
write_csv(employees, "out.csv");
write_json(employees, "out.json");
```

### Lists (script-level, not table data)

```griz
files = ["jan.csv", "feb.csv", "mar.csv"];
n = length(files);
more = append(files, "apr.csv");

# same transform, run once per file
for f in files {
  data = read_csv(f);
  summary = data |> group_by(dept) |> summarize(total: sum(revenue));
  write_csv(summary, f ++ "_summary.csv");
}

# one output table per distinct group
depts = employees |> distinct(dept) |> pull(dept);
for d in depts {
  employees |> filter(dept == d) |> write_csv(d ++ "_employees.csv");
}

# sweep a scalar parameter across the same computation
for rate in [0.05, 0.10, 0.15] {
  print(tax(50000, rate));
}
```

### Full example: monthly regional report

```griz
monthly_regional_report = () -> {
  orders = read_csv("orders.csv");
  customers = read_csv("customers.csv");

  orders |= update_col(quantity: fill_null(quantity, 0));

  customers |= add_col(
    tier: match lifetime_spend {
      case > 10000 => "vip"
      case > 1000  => "regular"
      else => "new"
    }
  );

  enriched =
    orders
      |> join(customers, on: customer_id, how: left)
      |> add_col(revenue: quantity * unit_price);

  high_value_orders =
    enriched |> filter(tier == "vip" and revenue > 500);

  regional_totals =
    enriched
      |> group_by(region, month)
      |> summarize(
           total_revenue: sum(revenue),
           order_count: count(order_id),
           vip_revenue: sum(if tier == "vip" then revenue else 0)
         )
      |> sort(total_revenue);

  write_csv(regional_totals, "regional_report.csv");
  write_csv(high_value_orders, "vip_orders_flagged.csv");

  return regional_totals;
};

monthly_regional_report();
```

## Design rationale

The language design is grounded in tabular data transformation. Each decision exists to make pipelines read and write naturally, with the structure of the language following the way data transformations are expressed.

**Read order is execution order.** SQL makes you write queries in an order that doesn't match how they execute or how you reason about them. Grizzly instead uses a top-down sequence of transformations, so the code reads in the same order it runs. This is the central design principle behind the language.

**Mostly one way to do things.** Grizzly deliberately avoids alternative spellings for the same operation. `=>` is for `match`, `->` is for lambdas, `+` is arithmetic, and `++` is concatenation. A small set of distinct primitives keeps code predictable and consistent.

**Functions over keywords.** `filter`, `join`, `sort`, and `sum` are ordinary functions. Keywords are reserved for constructs that genuinely need special syntax, such as `if`, `match`, loops, and `table`. User-defined transformations therefore look and behave like built-ins.

**No object system.** Tables and lists are plain values. Operations are functions, such as `filter(table, condition)`, and can be written with pipes as `table |> filter(condition)`. There are no methods, classes, or dot notation.

**Functions are values.** Functions can be assigned, passed, returned, or written anonymously (`x -> x + 1`). This provides closures naturally and lets users build new transformations from existing functions.

**Everything produces a value.** `if`, `match`, and blocks are expressions. Their results can be assigned, passed to functions, or composed with other expressions. For example, `sum(if tier == "vip" then revenue else 0)` works because `if` produces a value.

**Pipes make sequences readable.** `|>` turns nested calls like `sort(summarize(filter(...)))` into a sequence where each step follows the previous one. The syntax exists to preserve Grizzly's top-down reading order.

**Semicolons terminate statements.** Explicit `;` makes statement boundaries clear across line breaks, so pipelines, calls, and table literals can span multiple lines without special newline rules.

**Braces delimit blocks.** Braces give blocks explicit boundaries without requiring indentation-based parsing. Square brackets hold data, list and table literals, making data visually distinct from code.

**Row-context evaluation, not vectorization.** Table verbs like `filter` and `add_col` evaluate their expression argument once per row, with column names bound as ordinary scalars for that row. Every operator (`+`, `>`, `and`, `or`, ...) is always scalar-to-scalar; there is no vector/column value type and no broadcasting. This keeps operator semantics uniform instead of giving `and`/`or`/comparisons a second, elementwise meaning depending on whether their operands are scalars or columns.

## Known limitations

- **Functions that capture their own binding are never freed.** A function holds its environment through an `Rc`, so `f = x -> x;` stores `f` in the same environment `f` captured, and neither reference count reaches zero. That memory lives until the process exits, which is harmless for a script or a REPL session, but a long-running program creating many such functions would grow.

- **No vectorized or columnar execution.** Table verbs (`filter`, `add_col`, `summarize`, `sort`, ...) are implemented as tree-walking interpretation that re-evaluates the row expression once per row, rather than compiling to a columnar, batched execution plan. This is a scope simplification since it will not scale to real-sized datasets.

- **Lambdas can't forward a row expression.** Lambdas have only value parameters, so a wrapper like `(t, cond) -> filter(t, cond)` evaluates `cond` at its call site, where the columns aren't bound. Wrappers capture what varies as values instead, see Wrapping a verb.

- **Column names are fixed in the source.** A column parameter takes a bare identifier, never a `Str`, so code can't pick a column from a variable, and a CSV header that isn't an identifier (`first name`) can't be loaded.

- **CSV types are inferred, not declared.** A column of zip codes like `00123` loads as `Int`s and loses its leading zeros. There's no way to ask for a `Str` column.

- **Only inner and left joins.** A right join is a left join with the tables swapped. Outer joins aren't supported.

## Changelog

| Activity | What changed in the language                                                                                                                                                                                              |
| -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Lab 1    | Scanner with the full token vocabulary (keywords, operators, punctuation), `--tokenize` flag and line REPL, string escapes `\n \t \\ \" \{`, `Int` and `Float` literals (digits on both sides of `.`), `#` line comments  |
| Lab 2    | Recursive descent parser for expressions, `--parse` flag and parse REPL, `;`-terminated expressions, prefix AST printer with `(group ...)`, `^` right-associative, non-chaining comparisons, syntax error recovery at `;` |
| Lab 3    | Tree-walking evaluator, `--eval` flag and value-printing REPL, `Int`/`Float`/`Str`/`Bool`/`Null` values, checked `Int` arithmetic, IEEE 754 `Float`s, `++` concatenation, truthiness rule, runtime errors with exit 70 |
| Lab 4    | Plain `./run <file>` executes programs and the REPL keeps state across lines, native `print` bound as a global, identifiers and calls, assignment as an expression with `= += -= *= /= \|=`, braced blocks with lexical scope and no declaration keyword, environment restored after runtime errors, syntax error recovery at `;`, `}`, and statement keywords |
