import fs from "node:fs/promises";
import path from "node:path";
import YAML from "yaml";

const repoRoot = process.cwd();

async function* walk(dir) {
  const entries = await fs.readdir(dir, { withFileTypes: true });
  for (const entry of entries) {
    if (
      entry.name === "node_modules" ||
      entry.name === "_site" ||
      entry.name === "build" ||
      entry.name === ".git"
    )
      continue;
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      yield* walk(fullPath);
    } else if (entry.isFile()) {
      const lower = entry.name.toLowerCase();
      if (lower.endsWith(".yml") || lower.endsWith(".yaml")) {
        yield fullPath;
      }
    }
  }
}

const yamlFiles = [];
for await (const f of walk(repoRoot)) yamlFiles.push(f);
yamlFiles.sort();

let failed = false;
for (const file of yamlFiles) {
  const text = await fs.readFile(file, "utf8");
  try {
    YAML.parseAllDocuments(text);
  } catch (err) {
    failed = true;
    console.error(`YAML parse failed: ${path.relative(repoRoot, file)}`);
    console.error(err?.message ?? String(err));
  }
}

if (failed) process.exit(1);
