import { test, expect } from "@playwright/test";

// base_path from Dioxus.toml
const BASE = "/LogOut";

test.describe("Home page", () => {
  test("renders title and tagline", async ({ page }) => {
    await page.goto(`${BASE}/`);
    await expect(page.locator("h1")).toHaveText("💪 LogOut");
    await expect(page.locator(".home-tagline")).toHaveText(
      "Turn off your computer, Log your workOut"
    );
  });

  test("has navigation links", async ({ page }) => {
    await page.goto(`${BASE}/`);
    await expect(
      page.locator("a", { hasText: "Start New Workout" })
    ).toBeVisible();
    await expect(
      page.locator("a", { hasText: "Browse Exercises" })
    ).toBeVisible();
  });

  test("navigates to active session page", async ({ page }) => {
    await page.goto(`${BASE}/`);
    await page.click("a:has-text('Start New Workout')");
    await expect(page.locator(".session-header__title")).toContainText(
      "Active Session"
    );
  });

  test("navigates to exercise list page", async ({ page }) => {
    await page.goto(`${BASE}/`);
    await page.click("a:has-text('Browse Exercises')");
    await expect(page.locator("h1")).toHaveText("Exercise Database");
  });
});

test.describe("Exercise list page", () => {
  test("renders with search input", async ({ page }) => {
    await page.goto(`${BASE}/exercises`);
    await expect(page.locator(".search-input")).toBeVisible();
    await expect(page.locator("h1")).toHaveText("Exercise Database");
  });

  test("has back link to home", async ({ page }) => {
    await page.goto(`${BASE}/exercises`);
    await page.click("a:has-text('Back')");
    await expect(page.locator("h1")).toHaveText("💪 LogOut");
  });
});

test.describe("Active session page", () => {
  test("renders session header with timer", async ({ page }) => {
    await page.goto(`${BASE}/session`);
    await expect(page.locator(".session-header__title")).toContainText(
      "Active Session"
    );
    await expect(
      page.locator("button", { hasText: "Finish Session" })
    ).toBeVisible();
  });

  test("has exercise search input", async ({ page }) => {
    await page.goto(`${BASE}/session`);
    await expect(
      page.locator('input[placeholder="Search for an exercise..."]')
    ).toBeVisible();
  });

  test("has analytics button", async ({ page }) => {
    await page.goto(`${BASE}/session`);
    await expect(
      page.locator("button", { hasText: "Analytics" })
    ).toBeVisible();
  });

  test("finish session navigates to home", async ({ page }) => {
    await page.goto(`${BASE}/session`);
    await page.click("button:has-text('Finish Session')");
    await expect(page.locator("h1")).toHaveText("💪 LogOut");
  });
});
