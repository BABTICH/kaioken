// tests/e2e/app.spec.ts
import {
  _electron as electron,
  expect,
  test
} from "@playwright/test";
import type {
  ElectronApplication,
  Page
} from "@playwright/test";

import os from "node:os";
import path from "node:path";
import fs from "node:fs";

let app: ElectronApplication;
let page: Page;
let testUserDataDir: string;

test.describe.configure({
  mode: "serial"
});

test.beforeAll(async () => {
  const appPath =
    process.env.E2E_ELECTRON_APP_PATH ?? process.cwd();

  testUserDataDir = path.join(os.tmpdir(), `kaioken-e2e-${Date.now()}`);

  const extraFlags = process.env.CI
    ? ["--no-sandbox", "--disable-gpu", `--user-data-dir=${testUserDataDir}`]
    : ["--disable-gpu", `--user-data-dir=${testUserDataDir}`];

  const env: Record<string, string> = {};

  for (const [key, value] of Object.entries(process.env)) {
    if (typeof value === "string") {
      env[key] = value;
    }
  }

  env.NODE_ENV = "test";
  env.ELECTRON_DISABLE_SECURITY_WARNINGS = "true";

  app = await electron.launch({
    args: [...extraFlags, appPath],
    env
  });

  page = await app.firstWindow();

  // If a previous crash triggered the recovery surface, start fresh.
  const startFresh = page.getByRole("button", {
    name: "Start Fresh"
  });

  if (await startFresh.isVisible().catch(() => false)) {
    await startFresh.click();
  }

  await page.waitForSelector('[data-app-shell="true"]', {
    timeout: 90_000
  });
});

test.afterAll(async () => {
  await app?.close();
  try {
    if (testUserDataDir && fs.existsSync(testUserDataDir)) {
      fs.rmSync(testUserDataDir, { recursive: true, force: true });
    }
  } catch {
    // ignore cleanup errors
  }
});

test("boots into Kaioken app shell", async () => {
  await expect(
    page.locator('[data-app-shell="true"]')
  ).toBeVisible();

  await expect(
    page.locator(".welcome-surface")
  ).toBeVisible();

  await expect(
    page.getByAltText("Kaioken Desktop App")
  ).toBeVisible();
});

test("navigates all primary surfaces", async () => {
  await page.keyboard.press("Control+Digit1");
  await expect(page.locator(".chat-surface")).toBeVisible();

  await page.keyboard.press("Control+Digit2");
  await expect(page.locator(".research-surface")).toBeVisible();

  await page.keyboard.press("Control+Digit3");
  await expect(page.locator(".wiki-surface")).toBeVisible();

  await page.keyboard.press("Control+Digit4");
  await expect(page.locator(".codemap-surface")).toBeVisible();

  await page.keyboard.press("Control+Digit5");
  await expect(page.locator(".cards-surface")).toBeVisible();

  await page.keyboard.press("Control+Digit6");
  await expect(page.locator(".cost-surface")).toBeVisible();

  await page.keyboard.press("Control+Digit7");

  const certification = page
    .getByText("Release Certification")
    .first();

  await certification.scrollIntoViewIfNeeded();
  await expect(certification).toBeVisible();
});

test("research surface can run example query", async () => {
  await page.keyboard.press("Control+Digit2");

  const exampleButton = page.getByRole("button", {
    name: "Example"
  });

  await exampleButton.click();

  await expect(
    page.locator(".research-result").first()
  ).toBeVisible({
    timeout: 20_000
  });
});

test("wiki surface loads mock pages", async () => {
  await page.keyboard.press("Control+Digit3");

  await expect(
    page.locator(".wiki-page-item").first()
  ).toBeVisible();
});

test("codemap surface loads graph", async () => {
  await page.keyboard.press("Control+Digit4");

  await expect(page.locator(".graph-svg")).toBeVisible();
  await expect(page.locator(".graph-node").first()).toBeVisible();
});

test("cards surface loads mock cards", async () => {
  await page.keyboard.press("Control+Digit5");

  await expect(page.locator(".card").first()).toBeVisible();
});

test("cost surface loads spend chart", async () => {
  await page.keyboard.press("Control+Digit6");

  await expect(page.locator(".spend-chart-svg")).toBeVisible();
});

test("settings surface contains release certification", async () => {
  await page.keyboard.press("Control+Digit7");

  const certification = page
    .getByText("Release Certification")
    .first();

  await certification.scrollIntoViewIfNeeded();
  await expect(certification).toBeVisible();
});

test("toggles light mode and updates data-theme attribute", async () => {
  const html = page.locator("html");
  await expect(html).toHaveAttribute("data-theme", "dark");

  // Verify theme toggle is removed from NavRail (left panel)
  const navRailThemeBtn = page.getByRole("navigation").getByRole("button", { name: "Toggle theme" });
  await expect(navRailThemeBtn).toHaveCount(0);

  // Toggle light mode using top-right TitleBar button
  const titleBarThemeBtn = page.getByRole("banner").getByRole("button", { name: "Toggle theme" });
  await titleBarThemeBtn.click();
  await expect(html).toHaveAttribute("data-theme", "light");

  // Toggle back to dark mode using top-right TitleBar button
  await titleBarThemeBtn.click();
  await expect(html).toHaveAttribute("data-theme", "dark");
});

test("inspector panel is resizable horizontally", async () => {
  const inspector = page.locator("aside");
  await expect(inspector).toBeVisible();

  const initialBox = await inspector.boundingBox();
  expect(initialBox).not.toBeNull();
  const initialWidth = initialBox!.width;

  const resizer = page.locator('[title*="resize Inspector"]');
  await expect(resizer).toBeVisible();
  const resizerBox = await resizer.boundingBox();
  expect(resizerBox).not.toBeNull();

  // Drag the resizer to the left by 60px to expand inspector width
  await page.mouse.move(resizerBox!.x + resizerBox!.width / 2, resizerBox!.y + 50);
  await page.mouse.down();
  await page.mouse.move(resizerBox!.x - 60, resizerBox!.y + 50, { steps: 5 });
  await page.mouse.up();

  const newBox = await inspector.boundingBox();
  expect(newBox).not.toBeNull();
  expect(newBox!.width).toBeGreaterThan(initialWidth + 30);
});

test("terminal drawer toggles and auto-launches terminal session", async () => {
  // Toggle terminal drawer open
  await page.keyboard.press("Control+Backquote");

  const termDrawer = page.locator(".term");
  await expect(termDrawer).toBeVisible();

  // Expect New terminal button to be enabled (not disabled by missing workspace)
  const newTermBtn = page.getByRole("button", { name: "New terminal" });
  await expect(newTermBtn).toBeVisible();
  await expect(newTermBtn).toBeEnabled();

  // Expect active terminal session or running status to appear
  await expect(page.locator(".term-bar .tab").first()).toBeVisible({ timeout: 10_000 });
  await expect(page.locator(".term-status")).toHaveText(/RUNNING/, { timeout: 10_000 });

  // Create another tab using the New button
  const newBtn = page.getByRole("button", { name: "New", exact: true });
  await newBtn.click();
  await expect(page.locator(".term-bar .tab")).toHaveCount(3, { timeout: 5_000 });

  // Close the active tab
  const closeBtn = page.getByRole("button", { name: "Close", exact: true });
  await closeBtn.click();
  await expect(page.locator(".term-bar .tab")).toHaveCount(2, { timeout: 5_000 });
});

