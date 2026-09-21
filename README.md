# Grizzly

## Creators

- Andrian Lloyd Maagma (andrianllmm)
- Julian Hanns Medalla (jtmedalla)

## Overview

Grizzly is a small language for transforming tabular data. It's built for data analysts and engineers who'd otherwise reach for a general-purpose language or a SQL dialect to do the same repetitive work. Tables are first-class values, definable inline or loaded from a file. Every transformation is just an ordinary function, chained with pipes rather than nested calls or query clauses. Code reads top to bottom in the same order it runs, so a pipeline looks like the sequence of steps it performs. Writing Grizzly should feel like describing a data pipeline, not programming one.

## Host language and build

- Host language: Rust (Release)
- Version metadata: `rust-toolchain.toml`
- Build: `./build.sh`

## Running it

| Command                   | What it does                                    |
| ------------------------- | ----------------------------------------------- |
| `./run <file>`            | Executes a program.                             |
| `./run --tokenize <file>` | Prints the token stream.                        |
| `./run --parse <file>`    | Prints the parsed tree.                         |
| `./run --eval <file>`     | Evaluates each expression and prints its value. |
| `./run`                   | Starts the REPL.                                |

Exit codes: 0 on success, 65 on a static error (lexical, syntax, undefined name), 70 on a runtime error (type mismatch, division by zero, arity mismatch).

## File extension

`.griz`

# Lexical structure

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
| `\|>`               | pipe       | binary   | left          | 0            | `x \|> f` means `f(x)`                                  |
| `\|=`               | assignment | binary   | right         | 0            | `x \|= f1 \|> f2` means `x = x \|> f1 \|> f2`           |
| `or`                | logical    | binary   | left          | 1 (loosest)  |                                                         |
| `and`               | logical    | binary   | left          | 2            |                                                         |
| `not`               | logical    | unary    | right         | 3            |                                                         |
| `==` `!=`           | comparison | binary   | left          | 4            |                                                         |
| `<` `<=` `>` `>=`   | comparison | binary   | left          | 4            |                                                         |
| `+` `-`             | arithmetic | binary   | left          | 5            |                                                         |
| `++`                | string     | binary   | left          | 5            | concatenation                                           |
| `*` `/` `%`         | arithmetic | binary   | left          | 6            |                                                         |
| `-` (unary)         | arithmetic | unary    | right         | 7            |                                                         |
| `^`                 | arithmetic | binary   | right         | 8 (tightest) | exponent                                                |
| `->`                | other      | n/a      | n/a           | n/a          | lambda arrow, introduces a lambda's body                |
| `=>`                | other      | n/a      | n/a           | n/a          | match arrow, introduces a `case`/`else` clause's result |

### Literals

| Kind                | Syntax                                                                                   | Produces  |
| ------------------- | ---------------------------------------------------------------------------------------- | --------- |
| Integer             | `42`                                                                                     | `Int`     |
| Float               | `3.14`                                                                                   | `Float`   |
| String              | `"hello"`, escapes: `\n \t \\ \" \{`                                                     | `String`  |
| Interpolated string | `"Hi {name}, you earn {salary}"`                                                         | `String`  |
| Boolean             | `true`, `false`                                                                          | `Bool`    |
| Null                | `null`                                                                                   | `Null`    |
| List                | `[1, 2, 3]`                                                                              | `List(T)` |
| Table               | `table [ col1 \| col2 \| col3 ... ]` (header row, then one row per line, `\|`-separated) | `Table`   |

### Identifiers

- Start characters: letters, `_`
- Continue characters: letters, digits, `_`
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

## Grammar

```
program     → exprStmt* EOF

exprStmt    → expression ";"

assignment  → IDENTIFIER ( "=" | "+=" | "-=" | "*=" | "/=" | "|=" ) 
assignment
            | pipe

pipe        → logicOr ( "|>" logicOr )*

logicOr     → logicAnd ( "or" logicAnd )*

logicAnd    → logicNot ( "and" logicNot )*

logicNot    → "not" logicNot
            | comparison

comparison  → term ( ( "==" | "!=" | "<" | "<=" | ">" | ">=" ) term )*

term        → factor ( ( "+" | "-" | "++" ) factor )*

factor      → unary ( ( "*" | "/" | "%" ) unary )*

unary       → "-" unary
            | exponent

exponent    → primary ( "^" exponent )?

primary     → INTEGER | FLOAT | STRING | "true" | "false" | "null"
            | IDENTIFIER
            | "(" expression ")"
```

## Parse output format

```
(+ 2 (* 3 4))
(* (group (+ 2 3)) 4)
(- (- 1 2) 3)
```

- Groupings print as:  `(group <expr>)`

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

### Variables, arithmetic, strings, booleans

```
age = 25;
name = "Ana";
rate = 0.2;

total = 10 + 5 * 2;
avg = total / 3;
greeting = "Hi, " ++ name ++ "!";
can_vote = age >= 18 and name != "";
```

### Control flow

```
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

### Functions, closures, forwarding to a verb

```
tax = (amount, rate) -> amount * rate;

make_adder = n -> (x -> x + n);
add5 = make_adder(5);
add5(10);

my_filter = (t, cond) -> filter(t, cond);
```

### Creating a table

```

small = table [
  name  | dept   | salary
  "Ana" | "eng"  | 60000
  "Bo"  | "eng"  | 45000
  "Cy"  | "sales"| 70000
];

employees = read_csv("employees.csv");
```

### Filtering and selecting

```
employees |> filter(salary > 50000);
employees |> filter(salary > 50000 and dept == "eng");

employees |> select(name, dept, salary);
employees |> drop(ssn, internal_notes);


employees |= select(name, age) |> filter(age > 18);
```

### Adding and computing columns

```
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

```
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

### Sorting, slicing, distinct

```
employees |> slice(10, 20);
employees |> head(5);
employees |> tail(5);
```

### Sorting, distinct

```
employees |> sort(salary, desc);
employees |> distinct(dept);
```

### Joining and combining tables

```
merged = employees |> join(departments, on: dept, how: left);
combined = concat(q1_sales, q2_sales, q3_sales);
renamed = employees |> rename(dept: department, salary: pay);
```

### Column and Record access

```
salaries = employees |> pull(salary)
avg = mean(salaries);

first = employees |> first_row();

first_salary = get(first, "salary");
```

### Nulls

```
employees |> update_col(quantity, fill_null(quantity, 0));
```

### String interpolation

```
name = "Ana";
salary = 60000;

"Hi {name}, you earn {salary}";
```

### Writing output

```
write_csv(employees, "out.csv");
write_json(employees, "out.json");
```

### Lists (script-level, not table data)

```
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

```
monthly_regional_report = () -> {
  orders = read_csv("orders.csv")
  customers = read_csv("customers.csv")

  orders |= update_col(quantity, fill_null(quantity, 0));

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

**No object system.** Tables, columns, and records are plain values. Operations are functions, such as `filter(table, condition)`, and can be written with pipes as `table |> filter(condition)`. There are no methods, classes, or dot notation.

**Functions are values.** Functions can be assigned, passed, returned, or written anonymously (`x -> x + 1`). This provides closures naturally and lets users build new transformations from existing functions.

**Everything produces a value.** `if`, `match`, and blocks are expressions. Their results can be assigned, passed to functions, or composed with other expressions. For example, `sum(if tier == "vip" then revenue else 0)` works because `if` produces a value.

**Pipes make sequences readable.** `|>` turns nested calls like `sort(summarize(filter(...)))` into a sequence where each step follows the previous one. The syntax exists to preserve Grizzly's top-down reading order.

**Semicolons terminate statements.** Explicit `;` makes statement boundaries clear across line breaks, so pipelines, calls, and table literals can span multiple lines without special newline rules.

**Braces delimit blocks.** Braces give blocks explicit boundaries without requiring indentation-based parsing. Square brackets are reserved for table literals, making data visually distinct from code.

**Row-context evaluation, not vectorization.** Table verbs like `filter` and `add_col` evaluate their expression argument once per row, with column names bound as ordinary scalars for that row. Every operator (`+`, `>`, `and`, `or`, ...) is always scalar-to-scalar; there is no vector/column value type and no broadcasting. This keeps operator semantics uniform instead of giving `and`/`or`/comparisons a second, elementwise meaning depending on whether their operands are scalars or columns.

## Known limitations

- **No vectorized or columnar execution.** Table verbs (`filter`, `add_col`, `summarize`, `sort`, ...) are implemented as tree-walking interpretation that re-evaluates the row expression once per row, rather than compiling to a columnar, batched execution plan. This is a scope simplification since it will not scale to real-sized datasets.

## Changelog

| Activity | What changed in the language |
| -------- | ---------------------------- |
| Lab 1    | [entry]                      |
