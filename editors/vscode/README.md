# Grizzly for VS Code

Language support for [Grizzly](https://github.com/andrianllmm/cmsc124-interpreter) `.griz` files: syntax highlighting (including ```` ```griz ```` blocks in Markdown), comment toggling, bracket matching and auto-closing, and a file icon.

## Development

```sh
npm run assets   # copy the logo and license from the repo root
npm test         # check the grammar's keywords against src/keyword.rs
```

Then open this folder in VS Code and press F5 to launch a window with the extension loaded.

When the language changes, update `syntaxes/grizzly.tmLanguage.json` and `language-configuration.json`.

## Publishing

```sh
npx @vscode/vsce package   # builds grizzly-<version>.vsix
npx @vscode/vsce publish
```
