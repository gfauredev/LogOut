import { test, expect } from "@playwright/test";

// base_path from Dioxus.toml
const BASE = "/LogOut";

test.describe("Sessions tab (home page)", () => {
  test("renders title and tagline", async ({ page }) => {
    await page.goto(`${BASE}/`);
    await expect(page.locator(".app-title")).toHaveText("💪 LogOut");
    await expect(page.locator(".app-tagline")).toHaveText(
      "Turn off your computer, Log your workOut"
    );
  });

  test("has permanent bottom navigation with 3 tabs", async ({ page }) => {
    await page.goto(`${BASE}/`);
    await expect(page.locator(".bottom-nav")).toBeVisible();
    await expect(
      page.locator(".bottom-nav__tab", { hasText: "Exercises" })
    ).toBeVisible();
    await expect(
      page.locator(".bottom-nav__tab", { hasText: "Sessions" })
    ).toBeVisible();
    await expect(
      page.locator(".bottom-nav__tab", { hasText: "Analytics" })
    ).toBeVisible();
  });

  test("has FAB button to start a new session", async ({ page }) => {
    await page.goto(`${BASE}/`);
    await expect(page.locator(".fab")).toBeVisible();
  });

  test("starting a session shows session view with timer and Cancel button", async ({
    page,
  }) => {
    await page.goto(`${BASE}/`);
    await page.click(".fab");
    await expect(page.locator(".session-header__title")).toContainText(
      "Active Session"
    );
    await expect(
      page.locator("button", { hasText: "Cancel Session" })
    ).toBeVisible();
  });

  test("cancelling an empty session from header returns to sessions list", async ({
    page,
  }) => {
    await page.goto(`${BASE}/`);
    await page.click(".fab");
    await expect(page.locator(".session-header__title")).toContainText(
      "Active Session"
    );
    await page.click("button:has-text('Cancel Session')");
    await expect(page.locator(".app-title")).toHaveText("💪 LogOut");
    await expect(page.locator(".fab")).toBeVisible();
  });

  test("Sessions tab is active on home page", async ({ page }) => {
    await page.goto(`${BASE}/`);
    await expect(
      page.locator(".bottom-nav__tab--active", { hasText: "Sessions" })
    ).toBeVisible();
  });
});

test.describe("Exercise browser (Exercises tab)", () => {
  test("renders with search input and title", async ({ page }) => {
    await page.goto(`${BASE}/exercises`);
    await expect(page.locator(".search-input")).toBeVisible();
    await expect(page.locator("h1")).toHaveText("Exercise Database");
  });

  test("has permanent bottom navigation", async ({ page }) => {
    await page.goto(`${BASE}/exercises`);
    await expect(page.locator(".bottom-nav")).toBeVisible();
  });

  test("Exercises tab is active on exercise browser page", async ({ page }) => {
    await page.goto(`${BASE}/exercises`);
    await expect(
      page.locator(".bottom-nav__tab--active", { hasText: "Exercises" })
    ).toBeVisible();
  });

  test("navigate to exercises from home via bottom nav", async ({ page }) => {
    await page.goto(`${BASE}/`);
    await page.click(".bottom-nav__tab:has-text('Exercises')");
    await expect(page.locator("h1")).toHaveText("Exercise Database");
  });
});

test.describe("Analytics tab", () => {
  test("renders analytics page", async ({ page }) => {
    await page.goto(`${BASE}/analytics`);
    await expect(page.locator(".analytics-panel")).toBeVisible();
  });

  test("has permanent bottom navigation", async ({ page }) => {
    await page.goto(`${BASE}/analytics`);
    await expect(page.locator(".bottom-nav")).toBeVisible();
  });

  test("Analytics tab is active on analytics page", async ({ page }) => {
    await page.goto(`${BASE}/analytics`);
    await expect(
      page.locator(".bottom-nav__tab--active", { hasText: "Analytics" })
    ).toBeVisible();
  });

  test("navigate to analytics from home via bottom nav", async ({ page }) => {
    await page.goto(`${BASE}/`);
    await page.click(".bottom-nav__tab:has-text('Analytics')");
    await expect(page.locator(".analytics-panel")).toBeVisible();
  });
});

test.describe("Active session view", () => {
  test("has exercise search input", async ({ page }) => {
    await page.goto(`${BASE}/`);
    await page.click(".fab");
    await expect(
      page.locator('input[placeholder="Search for an exercise..."]')
    ).toBeVisible();
  });

  test("bottom nav is visible during active session", async ({ page }) => {
    await page.goto(`${BASE}/`);
    await page.click(".fab");
    await expect(page.locator(".bottom-nav")).toBeVisible();
  });

  test("empty session shows Cancel Session button", async ({ page }) => {
    await page.goto(`${BASE}/`);
    await page.click(".fab");
    await expect(
      page.locator("button", { hasText: "Cancel Session" })
    ).toBeVisible();
  });

  test("cancelling an empty session returns to home screen", async ({
    page,
  }) => {
    await page.goto(`${BASE}/`);
    await page.click(".fab");
    await page.click("button:has-text('Cancel Session')");
    await expect(page.locator(".app-title")).toHaveText("💪 LogOut");
  });
});

test.describe("PWA assets", () => {
  test("manifest.json is accessible and valid JSON", async ({ request }) => {
    const response = await request.get(`${BASE}/manifest.json`);
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body.name).toBeTruthy();
    expect(body.icons).toBeInstanceOf(Array);
  });

  test("service worker script sw.js is accessible", async ({ request }) => {
    const response = await request.get(`${BASE}/sw.js`);
    expect(response.ok()).toBeTruthy();
    const contentType = response.headers()["content-type"];
    // Content-type may vary between dev server and production; just verify the file exists and is text
    if (contentType) {
      expect(contentType).toMatch(/javascript|text/);
    }
  });

  test("manifest link is present in page HTML", async ({ page }) => {
    await page.goto(`${BASE}/`);
    const manifestLink = page.locator('link[rel="manifest"]');
    await expect(manifestLink).toHaveAttribute("href", "./manifest.json");
  });
});

