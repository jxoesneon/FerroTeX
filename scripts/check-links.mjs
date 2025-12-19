import fs from "node:fs/promises";
import path from "node:path";
import { spawn } from "node:child_process";

const repoRoot = process.cwd();

async function* walk(dir) {
  const entries = await fs.readdir(dir, { withFileTypes: true });
  for (const entry of entries) {
    if (entry.name === "node_modules" || entry.name === "_site" || entry.name === "build") continue;
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      yield* walk(fullPath);
    } else if (entry.isFile() && entry.name.toLowerCase().endsWith(".md")) {
      yield fullPath;
    }
  }
}

function run(cmd, args) {
  return new Promise((resolve, reject) => {
    const child = spawn(cmd, args, { stdio: "inherit" });
    child.on("exit", (code) => {
      if (code === 0) resolve();
      else reject(new Error(`${cmd} exited with code ${code}`));
    });
    child.on("error", reject);
  });
}

function getArgValue(flag) {
  const idx = process.argv.indexOf(flag);
  if (idx === -1) return undefined;
  return process.argv[idx + 1];
}

const configArg = getArgValue("--config");
const configPath = configArg
  ? path.resolve(repoRoot, configArg)
  : path.join(repoRoot, "scripts", "markdown-link-check.internal.json");

const mdFiles = [];
for await (const f of walk(repoRoot)) mdFiles.push(f);
mdFiles.sort();

for (const file of mdFiles) {
  const npmCmd = process.platform === "win32" ? "npm.cmd" : "npm";
  await run(npmCmd, ["exec", "--", "markdown-link-check", "-q", "-c", configPath, file]);
}
