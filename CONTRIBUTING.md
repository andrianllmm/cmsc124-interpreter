# Contributing

## Setup

1. Install Rust via [rustup](https://rustup.rs). The toolchain (`stable`) is pinned in `rust-toolchain.toml` and installs automatically.
2. Install [`cargo-commitlint`](https://github.com/) and enable it for this repo:
   ```sh
   cargo install cargo-commitlint
   cargo commitlint install-hook
   ```
   This wires up the `commit-msg` git hook (already present under `.git/hooks/commit-msg`) that lints every commit message against `commitlint.toml`.
3. Add the `rustfmt` and `clippy` components:
   ```sh
   rustup component add rustfmt clippy
   ```

## Build & run

```sh
./build.sh               # cargo build --release
./run <file>             # run a .griz program
./run --tokenize <file>
./run --parse <file>
./run --eval <file>
./run                    # REPL
```

## Testing

```sh
cargo test
python3 run_tests.py tests/...
```

## Commit messages

Commits are validated by commitlint (`commitlint.toml`). Format:

```
<type>(<optional-scope>): <subject>
```

- **type** - one of: `feat`, `fix`, `refactor`, `ux`, `docs`, `test`, `meta`. `chore` is not allowed.
- **scope** - optional, `kebab-case`.
- **subject** - lower-case/sentence-case, no trailing period, no ending whitespace.

## Branching & PRs

- Branch off `main`, one change per branch.
- Name branches `<type>/<short-description>`, matching the commit type used.
- Open a PR into `main`.
- Ensure CI passes.
- Wait for approval.
- Merge via PR.
