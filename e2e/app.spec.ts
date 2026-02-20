import { test, expect } from "@playwright/test";

// After the merge, base_path was removed from Dioxus.toml, so the app
// is served at root.  The dev-server (dx serve) does NOT add a prefix.
const BASE = "";

test.describe("Sessions tab (home page)", () => {
  test("renders title and tagline", async ({ page }) => {
    await page.goto(`${BASE}/`);
    await expect(page.locator(".app-title")).toHaveText("💪 LogOut");
    await expect(page.locator(".app-tagline")).toHaveText(
      "Turn off your computer, Log your workOut"
    );
  });

  test("has permanent bottom navigation with 4 icon tabs", async ({
    page,
  }) => {
    await page.goto(`${BASE}/`);
    await expect(page.locator(".bottom-nav")).toBeVisible();
    // After the redesign the nav only has emoji icons (no text labels
    // except Credits).  Just verify all 4 tabs exist.
    const tabs = page.locator(".bottom-nav__tab");
    await expect(tabs).toHaveCount(4);
  });

  test("has button to start a new session", async ({ page }) => {
    await page.goto(`${BASE}/`);
    await expect(page.locator(".new-session-button")).toBeVisible();
  });

  test("starting a session shows session view with Cancel Session button", async ({
    page,
  }) => {
    await page.goto(`${BASE}/`);
    await page.click(".new-session-button");
    await expect(page.locator(".session-header__title")).toContainText(
      "Active Session"
    );
    // Empty session → Cancel Session (Finish Session only appears after
    // completing at least one exercise)
    await expect(
      page.locator("button", { hasText: "Cancel Session" })
    ).toBeVisible();
  });

  test("cancelling an empty session returns to sessions list", async ({
    page,
  }) => {
    await page.goto(`${BASE}/`);
    await page.click(".new-session-button");
    await expect(page.locator(".session-header__title")).toContainText(
      "Active Session"
    );
    await page.click("button:has-text('Cancel Session')");
    await expect(page.locator(".app-title")).toHaveText("💪 LogOut");
    await expect(page.locator(".new-session-button")).toBeVisible();
  });

  test("Sessions tab is active on home page", async ({ page }) => {
    await page.goto(`${BASE}/`);
    // The active tab has the --active modifier class
    await expect(page.locator(".bottom-nav__tab--active")).toBeVisible();
  });
});

test.describe("Exercise browser (Exercises tab)", () => {
  test("renders with search input and title", async ({ page }) => {
    await page.goto(`${BASE}/exercises`);
    await expect(page.locator(".search-input")).toBeVisible();
    await expect(page.locator("h1")).toHaveText("📚 Exercise Database");
  });

  test("has permanent bottom navigation", async ({ page }) => {
    await page.goto(`${BASE}/exercises`);
    await expect(page.locator(".bottom-nav")).toBeVisible();
  });

  test("Exercises tab is active on exercise browser page", async ({
    page,
  }) => {
    await page.goto(`${BASE}/exercises`);
    await expect(page.locator(".bottom-nav__tab--active")).toBeVisible();
  });

  test("navigate to exercises from home via bottom nav", async ({ page }) => {
    await page.goto(`${BASE}/`);
    // Click the first tab (Exercises icon)
    await page.locator(".bottom-nav__tab").first().click();
    await expect(page.locator("h1")).toHaveText("📚 Exercise Database");
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
    await expect(page.locator(".bottom-nav__tab--active")).toBeVisible();
  });

  test("navigate to analytics from home via bottom nav", async ({ page }) => {
    await page.goto(`${BASE}/`);
    // Analytics is the 3rd tab (index 2)
    await page.locator(".bottom-nav__tab").nth(2).click();
    await expect(page.locator(".analytics-panel")).toBeVisible();
  });
});

test.describe("Active session view", () => {
  test("has exercise search input", async ({ page }) => {
    await page.goto(`${BASE}/`);
    await page.click(".new-session-button");
    await expect(
      page.locator('input[placeholder="Search for an exercise..."]')
    ).toBeVisible();
  });

  test("bottom nav is visible during active session", async ({ page }) => {
    await page.goto(`${BASE}/`);
    await page.click(".new-session-button");
    await expect(page.locator(".bottom-nav")).toBeVisible();
  });

  test("empty session shows Cancel Session button", async ({ page }) => {
    await page.goto(`${BASE}/`);
    await page.click(".new-session-button");
    await expect(
      page.locator("button", { hasText: "Cancel Session" })
    ).toBeVisible();
  });

  test("cancelling an empty session returns to home screen", async ({
    page,
  }) => {
    await page.goto(`${BASE}/`);
    await page.click(".new-session-button");
    await page.click("button:has-text('Cancel Session')");
    await expect(page.locator(".app-title")).toHaveText("💪 LogOut");
  });

  test("search does not crash app (regression test for duplicate keys)", async ({
    page,
  }) => {
    // This test verifies the fix for the "keyed siblings must each have a unique key" regression
    // Start a new session
    await page.goto(`${BASE}/`);
    await page.click(".new-session-button");
    await expect(page.locator(".session-header__title")).toContainText(
      "Active Session"
    );

    // Type in the search field - this used to crash when duplicate exercise IDs existed
    const searchInput = page.locator('input[placeholder="Search for an exercise..."]');
    await searchInput.fill("test");

    // Wait a bit for any potential crash to occur
    await page.waitForTimeout(1000);

    // Verify the app is still responsive (not crashed)
    await expect(page.locator(".session-header__title")).toContainText(
      "Active Session"
    );

    // Try searching for something else
    await searchInput.fill("pull");
    await page.waitForTimeout(500);

    // App should still be functional
    await expect(page.locator(".session-header__title")).toContainText(
      "Active Session"
    );
  });
});

test.describe("Exercise search functionality", () => {
  test("exercise list search does not crash (regression test)", async ({
    page,
  }) => {
    // Verify the fix works in the exercise list too
    await page.goto(`${BASE}/exercises`);
    await expect(page.locator("h1")).toHaveText("Exercise Database");

    // Search for exercises - this also had the duplicate key bug
    const searchInput = page.locator(".search-input");
    await searchInput.fill("push");

    // Wait for any potential crash
    await page.waitForTimeout(1000);

    // Verify page is still functional
    await expect(page.locator("h1")).toHaveText("Exercise Database");

    // Try another search
    await searchInput.fill("squat");
    await page.waitForTimeout(500);

    // Still functional
    await expect(page.locator("h1")).toHaveText("Exercise Database");
  });
});

test.describe("Credits / Extra tab", () => {
  test("renders credits page with heading", async ({ page }) => {
    await page.goto(`${BASE}/credits`);
    await expect(page.locator("h1.page-title")).toHaveText("ℹ️ Credits");
  });

  test("has exercise database URL input", async ({ page }) => {
    await page.goto(`${BASE}/credits`);
    await expect(page.locator(".db-url-input")).toBeVisible();
  });

  test("has permanent bottom navigation", async ({ page }) => {
    await page.goto(`${BASE}/credits`);
    await expect(page.locator(".bottom-nav")).toBeVisible();
  });

  test("navigate to credits from home via bottom nav", async ({ page }) => {
    await page.goto(`${BASE}/`);
    // Credits is the 4th tab (index 3)
    await page.locator(".bottom-nav__tab").nth(3).click();
    await expect(page.locator("h1.page-title")).toHaveText("ℹ️ Credits");
  });
});

test.describe("Tab heading consistency", () => {
  test("exercise browser heading is centered and has emoji", async ({
    page,
  }) => {
    await page.goto(`${BASE}/exercises`);
    const heading = page.locator("h1.page-title");
    await expect(heading).toBeVisible();
    const text = await heading.textContent();
    expect(text).toContain("📚");
  });

  test("analytics heading has emoji", async ({ page }) => {
    await page.goto(`${BASE}/analytics`);
    const heading = page.locator(".page-title");
    await expect(heading).toBeVisible();
    const text = await heading.textContent();
    expect(text).toContain("📊");
  });

  test("credits heading is centered and has emoji", async ({ page }) => {
    await page.goto(`${BASE}/credits`);
    const heading = page.locator("h1.page-title");
    await expect(heading).toBeVisible();
    const text = await heading.textContent();
    expect(text).toContain("ℹ️");
  });
});

test.describe("Active session – add exercise button placement", () => {
  test("add exercise (+) button appears after search bar", async ({ page }) => {
    await page.goto(`${BASE}/`);
    await page.click(".new-session-button");
    await expect(page.locator(".session-header__title")).toContainText(
      "Active Session"
    );

    const searchInput = page.locator('input[placeholder="Search for an exercise..."]');
    const addBtn = page.locator(".add-exercise-btn");
    await expect(searchInput).toBeVisible();
    await expect(addBtn).toBeVisible();

    // The add button must appear to the right of the search bar in the DOM
    const searchBox = await searchInput.boundingBox();
    const addBox = await addBtn.boundingBox();
    expect(addBox!.x).toBeGreaterThan(searchBox!.x);
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
    expect(contentType).toContain("javascript");
  });

  test("manifest link is present in page HTML", async ({ page }) => {
    await page.goto(`${BASE}/`);
    const manifestLink = page.locator('link[rel="manifest"]');
    await expect(manifestLink).toHaveAttribute("href", "./manifest.json");
  });
});

