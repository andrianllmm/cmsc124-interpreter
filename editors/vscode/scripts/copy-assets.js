// Copies shared repo files into the extension, since vsce only packages this folder.

const fs = require("fs");
const path = require("path");

const root = path.join(__dirname, "..", "..", "..");
const ext = path.join(__dirname, "..");

const files = {
  "assets/logo.svg": "icons/grizzly.svg",
  "assets/logo.png": "icons/grizzly.png",
  LICENSE: "LICENSE",
};

for (const [from, to] of Object.entries(files)) {
  fs.mkdirSync(path.dirname(path.join(ext, to)), { recursive: true });
  fs.copyFileSync(path.join(root, from), path.join(ext, to));
}
