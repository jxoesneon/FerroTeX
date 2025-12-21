#!/usr/bin/env node
/**
 * Validate package.json settings schema
 * Tests all 32 settings for correctness
 */

const fs = require("fs");
const path = require("path");

const packagePath = path.join(__dirname, "package.json");
const pkg = JSON.parse(fs.readFileSync(packagePath, "utf8"));

const config = pkg.contributes.configuration;
const props = config.properties;

console.log("🔍 Validating FerroTeX Settings Schema\n");

let errors = 0;
let warnings = 0;

// Expected settings count
const expectedCount = 32;
const actualCount = Object.keys(props).length;

if (actualCount !== expectedCount) {
  console.error(`❌ Expected ${expectedCount} settings, found ${actualCount}`);
  errors++;
} else {
  console.log(`✅ Correct number of settings: ${actualCount}`);
}

// Validate each setting
const settingsToValidate = [
  { key: "ferrotex.serverPath", type: "string", hasDefault: true },
  { key: "ferrotex.trace.server", type: "string", enum: ["off", "messages", "verbose"] },
  { key: "ferrotex.build.autoBuildOnSave", type: "boolean", hasDefault: true },
  {
    key: "ferrotex.build.engine",
    type: "string",
    enum: ["auto", "tectonic", "latexmk", "pdflatex", "xelatex", "lualatex"],
  },
  { key: "ferrotex.build.outputDirectory", type: "string", hasDefault: true },
  { key: "ferrotex.build.cleanAuxiliaryFiles", type: "boolean", hasDefault: true },
  { key: "ferrotex.build.showOutputPanel", type: "string", enum: ["always", "onError", "never"] },
  { key: "ferrotex.lint.enabled", type: "boolean", hasDefault: true },
  { key: "ferrotex.lint.onType", type: "boolean", hasDefault: true },
  { key: "ferrotex.lint.deprecatedCommands", type: "boolean", hasDefault: true },
  { key: "ferrotex.lint.obsoletePackages", type: "boolean", hasDefault: true },
  { key: "ferrotex.lint.displayMathDelimiters", type: "boolean", hasDefault: true },
  { key: "ferrotex.preview.syncToSource", type: "boolean", hasDefault: true },
  { key: "ferrotex.preview.defaultZoom", type: "number", min: 25, max: 500 },
  {
    key: "ferrotex.preview.scrollMode",
    type: "string",
    enum: ["vertical", "horizontal", "wrapped"],
  },
  { key: "ferrotex.completion.enabled", type: "boolean", hasDefault: true },
  { key: "ferrotex.completion.packages", type: "boolean", hasDefault: true },
  { key: "ferrotex.completion.citations", type: "boolean", hasDefault: true },
  { key: "ferrotex.format.enabled", type: "boolean", hasDefault: true },
  { key: "ferrotex.format.onSave", type: "boolean", hasDefault: true },
  { key: "ferrotex.format.indentSize", type: "number", min: 1, max: 8 },
  { key: "ferrotex.imagePaste.enabled", type: "boolean", hasDefault: true },
  { key: "ferrotex.imagePaste.defaultDirectory", type: "string", hasDefault: true },
  { key: "ferrotex.imagePaste.filenamePattern", type: "string", hasDefault: true },
  { key: "ferrotex.hover.enabled", type: "boolean", hasDefault: true },
  { key: "ferrotex.hover.citations", type: "boolean", hasDefault: true },
  { key: "ferrotex.hover.previewMath", type: "boolean", hasDefault: true },
  { key: "ferrotex.workspace.scanOnStartup", type: "boolean", hasDefault: true },
  { key: "ferrotex.workspace.maxFileSize", type: "number", min: 1, max: 100 },
  { key: "ferrotex.workspace.excludePatterns", type: "array", hasDefault: true },
  { key: "ferrotex.diagnostics.humanReadableErrors", type: "boolean", hasDefault: true },
  { key: "ferrotex.diagnostics.showCode", type: "boolean", hasDefault: true },
];

console.log("\n📋 Validating Individual Settings:\n");

for (const expected of settingsToValidate) {
  const setting = props[expected.key];

  if (!setting) {
    console.error(`❌ Missing setting: ${expected.key}`);
    errors++;
    continue;
  }

  // Check type
  if (setting.type !== expected.type) {
    console.error(
      `❌ ${expected.key}: Wrong type (expected ${expected.type}, got ${setting.type})`,
    );
    errors++;
  } else {
    console.log(`✅ ${expected.key}: Type correct`);
  }

  // Check enum values
  if (expected.enum) {
    if (!setting.enum || !Array.isArray(setting.enum)) {
      console.error(`❌ ${expected.key}: Missing enum values`);
      errors++;
    } else {
      const hasAll = expected.enum.every((v) => setting.enum.includes(v));
      if (!hasAll) {
        console.error(`❌ ${expected.key}: Enum values mismatch`);
        errors++;
      } else {
        console.log(`✅ ${expected.key}: Enum values correct`);
      }
    }
  }

  // Check min/max
  if (expected.min !== undefined) {
    if (setting.minimum !== expected.min) {
      console.error(
        `❌ ${expected.key}: Wrong minimum (expected ${expected.min}, got ${setting.minimum})`,
      );
      errors++;
    }
  }

  if (expected.max !== undefined) {
    if (setting.maximum !== expected.max) {
      console.error(
        `❌ ${expected.key}: Wrong maximum (expected ${expected.max}, got ${setting.maximum})`,
      );
      errors++;
    }
  }

  // Check for default
  if (expected.hasDefault && setting.default === undefined) {
    console.warn(`⚠️  ${expected.key}: Missing default value`);
    warnings++;
  }

  // Check for description
  if (!setting.markdownDescription && !setting.description) {
    console.warn(`⚠️  ${expected.key}: Missing description`);
    warnings++;
  }

  // Check for order (recommended)
  if (setting.order === undefined) {
    console.warn(`⚠️  ${expected.key}: Missing order property (recommended)`);
    warnings++;
  }

  // Check for scope
  if (!setting.scope) {
    console.warn(`⚠️  ${expected.key}: Missing scope`);
    warnings++;
  }
}

console.log(`\n${"=".repeat(60)}`);
console.log(`\n📊 Validation Summary:`);
console.log(`   Settings validated: ${settingsToValidate.length}`);
console.log(`   ✅ Errors: ${errors}`);
console.log(`   ⚠️  Warnings: ${warnings}`);

if (errors === 0) {
  console.log("\n✅ All settings schema validations passed!");
  process.exit(0);
} else {
  console.log("\n❌ Settings validation failed!");
  process.exit(1);
}
