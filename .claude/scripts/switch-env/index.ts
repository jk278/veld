#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const cacheFilePath = path.join(__dirname, "env.local.json");
const settingsFilePath = path.join(__dirname, "..", "..", "settings.local.json");

function switchEnv(selectedEnv: string): void {
  if (!fs.existsSync(cacheFilePath)) {
    console.error("❌ env.local.json not found!");
    process.exit(1);
  }

  const cacheData = JSON.parse(fs.readFileSync(cacheFilePath, "utf8"));
  const envNames = Object.keys(cacheData);

  // Case-insensitive matching: find the actual env name that matches (case-insensitive)
  const actualEnvName = envNames.find((name) => name.toLowerCase() === `env_${selectedEnv.toLowerCase()}`);

  if (!actualEnvName) {
    console.error(`❌ Environment "${selectedEnv}" not found!`);
    console.error("\nAvailable:");
    envNames.forEach((env) => {
      console.error(`  • ${env}`);
    });
    process.exit(1);
  }

  const settings: Record<string, unknown> = fs.existsSync(settingsFilePath)
    ? JSON.parse(fs.readFileSync(settingsFilePath, "utf8"))
    : {};

  settings.env = cacheData[actualEnvName];
  fs.writeFileSync(settingsFilePath, JSON.stringify(settings, null, 2));

  console.log(`✅ Switched to ${actualEnvName}`);
}

const argIndex = process.argv.indexOf("--env");
if (argIndex === -1 || !process.argv[argIndex + 1]) {
  console.error("Usage: node index.ts --env <env_name>");
  process.exit(1);
}

switchEnv(process.argv[argIndex + 1]);
