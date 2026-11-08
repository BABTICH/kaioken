// scripts/release.ts
import { execSync } from "node:child_process";

console.log("🚀 Starting Kaioken Desktop Studio release build...");
execSync("npm run build", { stdio: "inherit" });
console.log("📦 Packaging and publishing to GitHub Releases...");
execSync("npx electron-builder --publish always", { stdio: "inherit" });
console.log("✅ Release process complete.");
