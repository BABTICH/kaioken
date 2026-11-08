#!/usr/bin/env node

/**
 * Kaioken Desktop Studio — CLI Companion
 *
 * Usage:
 *   npx kaioken --help
 *   npx kaioken info
 *   npx kaioken open
 *   npx kaioken sessions
 *   npx kaioken clear-cache
 */

const { execSync, spawn } = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");
const os = require("node:os");

const APP_NAME = "Kaioken Desktop Studio";
const APP_ID = "com.kaioken.desktop-studio";

function getAppDataPath() {
  const platform = os.platform();
  const home = os.homedir();

  if (platform === "win32") {
    return path.join(process.env.APPDATA || path.join(home, "AppData", "Roaming"), APP_NAME);
  } else if (platform === "darwin") {
    return path.join(home, "Library", "Application Support", APP_NAME);
  } else {
    return path.join(process.env.XDG_CONFIG_HOME || path.join(home, ".config"), APP_NAME.toLowerCase().replace(/\s+/g, "-"));
  }
}

function readPackageJson() {
  try {
    const pkgPath = path.resolve(__dirname, "..", "package.json");
    return JSON.parse(fs.readFileSync(pkgPath, "utf8"));
  } catch {
    return { name: "kaioken-desktop", version: "unknown" };
  }
}

function printInfo() {
  const pkg = readPackageJson();
  const appData = getAppDataPath();

  console.log("");
  console.log("  ╔══════════════════════════════════════╗");
  console.log("  ║     KAIOKEN — Desktop Studio CLI     ║");
  console.log("  ╚══════════════════════════════════════╝");
  console.log("");
  console.log(`  Name:        ${pkg.name}`);
  console.log(`  Version:     ${pkg.version}`);
  console.log(`  App ID:      ${APP_ID}`);
  console.log(`  Electron:    ${pkg.devDependencies?.electron || "unknown"}`);
  console.log(`  React:       ${pkg.dependencies?.react || "unknown"}`);
  console.log(`  App Data:    ${appData}`);
  console.log("");
}

function printHelp() {
  console.log("");
  console.log("  Kaioken Desktop Studio — CLI Companion");
  console.log("");
  console.log("  Usage:");
  console.log("    npx kaioken <command>");
  console.log("");
  console.log("  Commands:");
  console.log("    info          Show app and environment info");
  console.log("    open          Launch the desktop app");
  console.log("    sessions      List persisted session data");
  console.log("    clear-cache   Clear cached app data");
  console.log("    help          Show this help message");
  console.log("");
}

function listSessions() {
  const appData = getAppDataPath();
  const sessionsPath = path.join(appData, "kaiopi-sessions");

  if (!fs.existsSync(sessionsPath)) {
    console.log("  No session data found.");
    return;
  }

  const files = fs.readdirSync(sessionsPath).filter(f => f.endsWith(".json"));

  if (files.length === 0) {
    console.log("  No session files found.");
    return;
  }

  console.log("");
  console.log("  Persisted Sessions:");
  console.log("  ─────────────────────────────────────");

  for (const file of files) {
    const filePath = path.join(sessionsPath, file);
    const stats = fs.statSync(filePath);
    const sizeKB = (stats.size / 1024).toFixed(1);
    const modified = stats.mtime.toISOString().split("T")[0];

    console.log(`  ${file.padEnd(30)} ${sizeKB.padStart(8)} KB  ${modified}`);
  }

  console.log("");
}

function clearCache() {
  const appData = getAppDataPath();

  if (!fs.existsSync(appData)) {
    console.log("  No app data found. Nothing to clear.");
    return;
  }

  const cachePaths = [
    path.join(appData, "Cache"),
    path.join(appData, "Code Cache"),
    path.join(appData, "GPUCache"),
  ];

  let cleared = 0;

  for (const cachePath of cachePaths) {
    if (fs.existsSync(cachePath)) {
      try {
        fs.rmSync(cachePath, { recursive: true, force: true });
        cleared++;
        console.log(`  Cleared: ${path.basename(cachePath)}`);
      } catch (err) {
        console.log(`  Failed to clear: ${path.basename(cachePath)}`);
      }
    }
  }

  if (cleared === 0) {
    console.log("  No cache directories found.");
  } else {
    console.log(`  Cleared ${cleared} cache director${cleared === 1 ? "y" : "ies"}.`);
  }
}

function openApp() {
  console.log("  Launching Kaioken Desktop Studio...");

  try {
    const electronPath = require("electron");
    const appPath = path.resolve(__dirname, "..");

    const child = spawn(electronPath, [appPath], {
      detached: true,
      stdio: "ignore"
    });

    child.unref();
    console.log("  App launched.");
  } catch (err) {
    console.error("  Failed to launch app:", err.message);
    console.error("  Make sure you are running this from the project root.");
  }
}

// Main dispatch
const command = process.argv[2] || "help";

switch (command) {
  case "info":
    printInfo();
    break;
  case "open":
    openApp();
    break;
  case "sessions":
    listSessions();
    break;
  case "clear-cache":
    clearCache();
    break;
  case "help":
  case "--help":
  case "-h":
    printHelp();
    break;
  default:
    console.log(`  Unknown command: ${command}`);
    printHelp();
    process.exit(1);
}
