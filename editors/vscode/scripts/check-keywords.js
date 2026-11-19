// Fails if the grammar's keywords drift from the interpreter's (src/keyword.rs).

const fs = require("fs");
const path = require("path");

const root = path.join(__dirname, "..", "..", "..");
const source = fs.readFileSync(path.join(root, "src", "keyword.rs"), "utf8");
const grammar = require("../syntaxes/grizzly.tmLanguage.json");

const expected = new Set([...source.matchAll(/"(\w+)"\s*=>/g)].map((m) => m[1]));
const actual = new Set(
  grammar.repository.keywords.patterns.flatMap((p) => p.match.match(/\(([^)]*)\)/)[1].split("|")),
);

const missing = [...expected].filter((k) => !actual.has(k));
const extra = [...actual].filter((k) => !expected.has(k));

if (missing.length || extra.length) {
  if (missing.length) console.error(`Missing from grammar: ${missing.join(", ")}`);
  if (extra.length) console.error(`Not keywords in src/keyword.rs: ${extra.join(", ")}`);
  process.exit(1);
}
console.log(`Grammar keywords match src/keyword.rs (${expected.size}).`);
